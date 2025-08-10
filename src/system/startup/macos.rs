// macOS 启动管理模块
// 支持 LaunchAgent、LaunchDaemon 和 LoginItems

use crate::system::common::error::{SystemResult, SystemError};
use super::types::{StartupEntry, StartupType};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::collections::HashMap;

/// plist 文件内容结构
#[derive(Debug)]
struct LaunchPlist {
    label: String,
    program: String,
    program_arguments: Vec<String>,
    run_at_load: bool,
    keep_alive: bool,
    working_directory: Option<String>,
    environment_variables: Option<HashMap<String, String>>,
    start_interval: Option<u32>,
}

impl LaunchPlist {
    fn to_plist_xml(&self) -> String {
        let mut xml = String::new();
        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str("<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n");
        xml.push_str("<plist version=\"1.0\">\n<dict>\n");
        
        // Label
        xml.push_str(&format!("\t<key>Label</key>\n\t<string>{}</string>\n", self.label));
        
        // Program
        xml.push_str(&format!("\t<key>Program</key>\n\t<string>{}</string>\n", self.program));
        
        // ProgramArguments
        if !self.program_arguments.is_empty() {
            xml.push_str("\t<key>ProgramArguments</key>\n\t<array>\n");
            xml.push_str(&format!("\t\t<string>{}</string>\n", self.program));
            for arg in &self.program_arguments {
                xml.push_str(&format!("\t\t<string>{}</string>\n", arg));
            }
            xml.push_str("\t</array>\n");
        }
        
        // RunAtLoad
        if self.run_at_load {
            xml.push_str("\t<key>RunAtLoad</key>\n\t<true/>\n");
        }
        
        // KeepAlive
        if self.keep_alive {
            xml.push_str("\t<key>KeepAlive</key>\n\t<true/>\n");
        }
        
        // WorkingDirectory
        if let Some(ref wd) = self.working_directory {
            xml.push_str(&format!("\t<key>WorkingDirectory</key>\n\t<string>{}</string>\n", wd));
        }
        
        // EnvironmentVariables
        if let Some(ref env_vars) = self.environment_variables {
            if !env_vars.is_empty() {
                xml.push_str("\t<key>EnvironmentVariables</key>\n\t<dict>\n");
                for (key, value) in env_vars {
                    xml.push_str(&format!("\t\t<key>{}</key>\n\t\t<string>{}</string>\n", key, value));
                }
                xml.push_str("\t</dict>\n");
            }
        }
        
        // StartInterval (for delayed start)
        if let Some(interval) = self.start_interval {
            xml.push_str(&format!("\t<key>StartInterval</key>\n\t<integer>{}</integer>\n", interval));
        }
        
        xml.push_str("</dict>\n</plist>\n");
        xml
    }
}

/// 获取用户 LaunchAgents 目录
fn get_user_launch_agents_dir() -> SystemResult<PathBuf> {
    let home_dir = std::env::var("HOME")
        .map_err(|_| SystemError::Configuration("Cannot get user home directory".to_string()))?;
    let launch_agents_dir = Path::new(&home_dir).join("Library/LaunchAgents");
    
    // 确保目录存在
    if !launch_agents_dir.exists() {
        fs::create_dir_all(&launch_agents_dir)?;
    }
    
    Ok(launch_agents_dir)
}

/// 获取系统 LaunchDaemons 目录
fn get_system_launch_daemons_dir() -> PathBuf {
    PathBuf::from("/Library/LaunchDaemons")
}

/// 解析 plist 文件获取启动项信息
fn parse_plist_file(file_path: &Path, startup_type: StartupType) -> SystemResult<StartupEntry> {
    let content = fs::read_to_string(file_path)?;
    
    if content.contains("<dict/>") {
        return Err(SystemError::Parse("plist file is empty".to_string()));
    }
    
    let label = extract_plist_string(&content, "Label")
        .unwrap_or_else(|| file_path.file_stem().unwrap().to_string_lossy().to_string());
    
    let (program, program_arguments) = if let Some(program) = extract_plist_string(&content, "Program") {
        let args = extract_plist_array(&content, "ProgramArguments");
        (program, args)
    } else {
        let program_args = extract_plist_array(&content, "ProgramArguments");
        if program_args.is_empty() {
            return Err(SystemError::Parse("Cannot find Program or ProgramArguments fields".to_string()));
        }
        let program = program_args[0].clone();
        let args = if program_args.len() > 1 {
            program_args[1..].to_vec()
        } else {
            Vec::new()
        };
        (program, args)
    };
    
    let run_at_load = extract_plist_bool(&content, "RunAtLoad").unwrap_or(false);
    let working_directory = extract_plist_string(&content, "WorkingDirectory");
    
    let metadata = fs::metadata(file_path)?;
    let modified_time = metadata.modified().ok();
    
    let is_daemon = matches!(startup_type, StartupType::LaunchDaemon);
    
    let mut entry = StartupEntry {
        id: label.clone(),
        name: label.clone(),
        command: program,
        arguments: program_arguments,
        description: Some(format!("macOS Launch{} Startup", 
            match &startup_type {
                StartupType::LaunchAgent => "Agent",
                StartupType::LaunchDaemon => "Daemon",
                _ => "Item"
            }
        )),
        startup_type,
        enabled: run_at_load,
        run_as_admin: is_daemon,
        delay_seconds: None,
        working_directory,
        environment_variables: None,
        created_time: None,
        last_modified: modified_time,
    };
    
    if let Some(env_vars) = extract_plist_dict(&content, "EnvironmentVariables") {
        entry.environment_variables = Some(env_vars);
    }
    
    Ok(entry)
}

fn extract_plist_string(content: &str, key: &str) -> Option<String> {
    let key_pattern = format!("<key>{}</key>", key);
    if let Some(key_pos) = content.find(&key_pattern) {
        let after_key = &content[key_pos + key_pattern.len()..];
        if let Some(string_start) = after_key.find("<string>") {
            let string_content = &after_key[string_start + 8..];
            if let Some(string_end) = string_content.find("</string>") {
                return Some(string_content[..string_end].to_string());
            }
        }
    }
    None
}

fn extract_plist_array(content: &str, key: &str) -> Vec<String> {
    let key_pattern = format!("<key>{}</key>", key);
    if let Some(key_pos) = content.find(&key_pattern) {
        let after_key = &content[key_pos + key_pattern.len()..];
        if let Some(array_start) = after_key.find("<array>") {
            let array_content = &after_key[array_start + 7..];
            if let Some(array_end) = array_content.find("</array>") {
                let array_inner = &array_content[..array_end];
                let mut items = Vec::new();
                let mut pos = 0;
                
                while let Some(string_start) = array_inner[pos..].find("<string>") {
                    let abs_start = pos + string_start + 8;
                    if let Some(string_end) = array_inner[abs_start..].find("</string>") {
                        items.push(array_inner[abs_start..abs_start + string_end].to_string());
                        pos = abs_start + string_end + 9;
                    } else {
                        break;
                    }
                }
                
                return items;
            }
        }
    }
    Vec::new()
}

fn extract_plist_bool(content: &str, key: &str) -> Option<bool> {
    let key_pattern = format!("<key>{}</key>", key);
    if let Some(key_pos) = content.find(&key_pattern) {
        let after_key = &content[key_pos + key_pattern.len()..];
        if after_key.trim_start().starts_with("<true/>") {
            return Some(true);
        } else if after_key.trim_start().starts_with("<false/>") {
            return Some(false);
        }
    }
    None
}

fn extract_plist_dict(content: &str, key: &str) -> Option<HashMap<String, String>> {
    let key_pattern = format!("<key>{}</key>", key);
    if let Some(key_pos) = content.find(&key_pattern) {
        let after_key = &content[key_pos + key_pattern.len()..];
        if let Some(dict_start) = after_key.find("<dict>") {
            let dict_content = &after_key[dict_start + 6..];
            if let Some(dict_end) = dict_content.find("</dict>") {
                let dict_inner = &dict_content[..dict_end];
                let mut result = HashMap::new();
                let mut pos = 0;
                
                while let Some(key_start) = dict_inner[pos..].find("<key>") {
                    let abs_key_start = pos + key_start + 5;
                    if let Some(key_end) = dict_inner[abs_key_start..].find("</key>") {
                        let dict_key = dict_inner[abs_key_start..abs_key_start + key_end].to_string();
                        let after_key_end = abs_key_start + key_end + 6;
                        
                        if let Some(string_start) = dict_inner[after_key_end..].find("<string>") {
                            let abs_string_start = after_key_end + string_start + 8;
                            if let Some(string_end) = dict_inner[abs_string_start..].find("</string>") {
                                let dict_value = dict_inner[abs_string_start..abs_string_start + string_end].to_string();
                                result.insert(dict_key, dict_value);
                                pos = abs_string_start + string_end + 9;
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                
                return Some(result);
            }
        }
    }
    None
}

/// 列出指定目录中的所有 plist 文件
fn list_plist_files_in_dir(dir: &Path, startup_type: StartupType) -> SystemResult<Vec<StartupEntry>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    
    let mut entries = Vec::new();
    
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() && path.extension().map_or(false, |ext| ext == "plist") {
            match parse_plist_file(&path, startup_type.clone()) {
                Ok(startup_entry) => entries.push(startup_entry),
                Err(e) => {
                    eprintln!("Warning: Failed to parse plist file {:?}: {}", path, e);
                }
            }
        }
    }
    
    Ok(entries)
}

/// 获取登录项 (LoginItems)
fn list_login_items() -> SystemResult<Vec<StartupEntry>> {
    let output = Command::new("osascript")
        .arg("-e")
        .arg("tell application \"System Events\" to get the name of every login item")
        .output();
    
    match output {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let items: Vec<&str> = stdout.trim().split(", ").collect();
                let mut entries = Vec::new();
                
                for item in items {
                    if !item.is_empty() && item != "missing value" {
                        let entry = StartupEntry {
                            id: format!("loginitem_{}", item),
                            name: item.to_string(),
                            command: item.to_string(), // 对于登录项，通常是应用程序名
                            arguments: Vec::new(),
                            description: Some("macOS Login Item".to_string()),
                            startup_type: StartupType::LoginItems,
                            enabled: true,
                            run_as_admin: false,
                            delay_seconds: None,
                            working_directory: None,
                            environment_variables: None,
                            created_time: None,
                            last_modified: None,
                        };
                        entries.push(entry);
                    }
                }
                
                Ok(entries)
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(SystemError::SystemCall(format!("Failed to get login items: {}", stderr), None))
            }
        }
        Err(e) => Err(SystemError::SystemCall(format!("Failed to execute osascript: {}", e), None))
    }
}

/// 列出所有启动项
pub fn list_all_startup_entries() -> SystemResult<Vec<StartupEntry>> {
    let mut all_entries = Vec::new();
    
    match get_user_launch_agents_dir() {
        Ok(agents_dir) => {
            match list_plist_files_in_dir(&agents_dir, StartupType::LaunchAgent) {
                Ok(mut entries) => all_entries.append(&mut entries),
                Err(e) => eprintln!("Warning: Failed to get user LaunchAgents: {}", e),
            }
        }
        Err(e) => eprintln!("Warning: Failed to get user LaunchAgents directory: {}", e),
    }
    
    let daemons_dir = get_system_launch_daemons_dir();
    match list_plist_files_in_dir(&daemons_dir, StartupType::LaunchDaemon) {
        Ok(mut entries) => all_entries.append(&mut entries),
        Err(e) => eprintln!("Warning: Failed to get system LaunchDaemons: {}", e),
    }
    
    match list_login_items() {
        Ok(mut entries) => all_entries.append(&mut entries),
        Err(e) => eprintln!("Warning: Failed to get login items: {}", e),
    }
    
    Ok(all_entries)
}

/// 添加启动项
pub fn add_startup_entry(entry: &StartupEntry) -> SystemResult<()> {
    // 验证启动项配置
    entry.validate().map_err(|e| SystemError::InvalidArgument(e))?;
    
    match entry.startup_type {
        StartupType::LaunchAgent => add_launch_agent(entry),
        StartupType::LaunchDaemon => add_launch_daemon(entry),
        StartupType::LoginItems => add_login_item(entry),
        _ => Err(SystemError::NotSupported(format!("Unsupported startup type: {:?}", entry.startup_type))),
    }
}

/// 添加 LaunchAgent
fn add_launch_agent(entry: &StartupEntry) -> SystemResult<()> {
    let agents_dir = get_user_launch_agents_dir()?;
    let plist_path = agents_dir.join(format!("{}.plist", entry.id));
    
    let plist = LaunchPlist {
        label: entry.id.clone(),
        program: entry.command.clone(),
        program_arguments: entry.arguments.clone(),
        run_at_load: entry.enabled,
        keep_alive: false, // 默认不保持活跃
        working_directory: entry.working_directory.clone(),
        environment_variables: entry.environment_variables.clone(),
        start_interval: entry.delay_seconds,
    };
    
    fs::write(&plist_path, plist.to_plist_xml())?;
    
    if entry.enabled {
        let output = Command::new("launchctl")
            .args(&["load", plist_path.to_string_lossy().as_ref()])
            .output()?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SystemError::SystemCall(format!("加载 LaunchAgent 失败: {}", stderr), None));
        }
    }
    
    Ok(())
}

fn add_launch_daemon(entry: &StartupEntry) -> SystemResult<()> {
    let daemons_dir = get_system_launch_daemons_dir();
    let plist_path = daemons_dir.join(format!("{}.plist", entry.id));
    
    if std::env::var("USER").unwrap_or_default() != "root" {
        return Err(SystemError::PermissionDenied("Adding system-level startup items requires root privileges".to_string()));
    }
    
    let plist = LaunchPlist {
        label: entry.id.clone(),
        program: entry.command.clone(),
        program_arguments: entry.arguments.clone(),
        run_at_load: entry.enabled,
        keep_alive: false,
        working_directory: entry.working_directory.clone(),
        environment_variables: entry.environment_variables.clone(),
        start_interval: entry.delay_seconds,
    };
    
    fs::write(&plist_path, plist.to_plist_xml())?;
    
    Command::new("chmod")
        .args(&["644", plist_path.to_string_lossy().as_ref()])
        .output()?;
    
    Command::new("chown")
        .args(&["root:wheel", plist_path.to_string_lossy().as_ref()])
        .output()?;
    
    if entry.enabled {
        let output = Command::new("launchctl")
            .args(&["load", plist_path.to_string_lossy().as_ref()])
            .output()?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SystemError::SystemCall(format!("加载 LaunchDaemon 失败: {}", stderr), None));
        }
    }
    
    Ok(())
}

/// 添加登录项
fn add_login_item(entry: &StartupEntry) -> SystemResult<()> {
    let script = format!(
        "tell application \"System Events\" to make login item at end with properties {{name:\"{}\", path:\"{}\", hidden:false}}",
        entry.name, entry.command
    );
    
    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SystemError::SystemCall(format!("添加登录项失败: {}", stderr), None));
    }
    
    Ok(())
}

/// 移除启动项
pub fn remove_startup_entry(id: &str, startup_type: StartupType) -> SystemResult<()> {
    match startup_type {
        StartupType::LaunchAgent => remove_launch_agent(id),
        StartupType::LaunchDaemon => remove_launch_daemon(id),
        StartupType::LoginItems => remove_login_item(id),
        _ => Err(SystemError::NotSupported(format!("不支持的启动类型: {:?}", startup_type))),
    }
}

/// 移除 LaunchAgent
fn remove_launch_agent(id: &str) -> SystemResult<()> {
    let agents_dir = get_user_launch_agents_dir()?;
    let plist_path = agents_dir.join(format!("{}.plist", id));
    
    if plist_path.exists() {
        let _ = Command::new("launchctl")
            .args(&["unload", plist_path.to_string_lossy().as_ref()])
            .output();
        
        fs::remove_file(&plist_path)?;
    }
    
    Ok(())
}

/// 移除 LaunchDaemon
fn remove_launch_daemon(id: &str) -> SystemResult<()> {
    let daemons_dir = get_system_launch_daemons_dir();
    let plist_path = daemons_dir.join(format!("{}.plist", id));
    
    if std::env::var("USER").unwrap_or_default() != "root" {
        return Err(SystemError::PermissionDenied("Removing system-level startup items requires root privileges".to_string()));
    }
    
    if plist_path.exists() {
        let _ = Command::new("launchctl")
            .args(&["unload", plist_path.to_string_lossy().as_ref()])
            .output();
        
        fs::remove_file(&plist_path)?;
    }
    
    Ok(())
}

/// 移除登录项
fn remove_login_item(id: &str) -> SystemResult<()> {
    let app_name = id.strip_prefix("loginitem_").unwrap_or(id);
    
    let script = format!(
        "tell application \"System Events\" to delete login item \"{}\"",
        app_name
    );
    
    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.contains("doesn't exist") {
            return Err(SystemError::SystemCall(format!("Failed to remove login item: {}", stderr), None));
        }
    }
    
    Ok(())
} 