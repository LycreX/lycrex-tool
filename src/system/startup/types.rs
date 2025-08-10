// 启动管理相关的类型定义

use std::time::SystemTime;
use std::collections::HashMap;

/// 启动项类型
#[derive(Debug, Clone, PartialEq)]
pub enum StartupType {
    // Windows
    RegistryCurrentUser,    // HKCU\Software\Microsoft\Windows\CurrentVersion\Run
    RegistryLocalMachine,   // HKLM\Software\Microsoft\Windows\CurrentVersion\Run
    StartupFolder,          // 启动文件夹
    WindowsService,         // Windows服务
    TaskScheduler,          // 任务计划程序
    
    // Linux
    SystemdUser,            // systemd用户服务
    SystemdSystem,          // systemd系统服务
    DesktopAutostart,       // ~/.config/autostart/
    
    // macOS
    LaunchAgent,            // ~/Library/LaunchAgents/ (用户级)
    LaunchDaemon,           // /Library/LaunchDaemons/ (系统级)
    LoginItems,             // 系统偏好设置 -> 用户与群组 -> 登录项
}

/// 启动项信息
#[derive(Debug, Clone)]
pub struct StartupEntry {
    pub id: String,              // 唯一标识符
    pub name: String,            // 显示名称
    pub command: String,         // 执行命令/路径
    pub arguments: Vec<String>,  // 命令参数数组
    pub description: Option<String>, // 描述
    pub startup_type: StartupType,
    pub enabled: bool,           // 是否启用
    pub run_as_admin: bool,      // 是否需要管理员权限
    pub delay_seconds: Option<u32>, // 延迟启动秒数
    pub working_directory: Option<String>, // 工作目录
    pub environment_variables: Option<HashMap<String, String>>, // 环境变量
    pub created_time: Option<SystemTime>, // 创建时间
    pub last_modified: Option<SystemTime>, // 最后修改时间
}

impl StartupEntry {
    /// 创建新的启动项
    pub fn new(name: String, command: String, startup_type: StartupType) -> Self {
        let id = format!("{}_{}", name.replace(' ', "_"), 
                        SystemTime::now().duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default().as_nanos());
        
        Self {
            id,
            name,
            command,
            arguments: Vec::new(),
            description: None,
            startup_type,
            enabled: true,
            run_as_admin: false,
            delay_seconds: None,
            working_directory: None,
            environment_variables: None,
            created_time: Some(SystemTime::now()),
            last_modified: Some(SystemTime::now()),
        }
    }
    
    /// 设置参数
    pub fn with_arguments(mut self, args: Vec<String>) -> Self {
        self.arguments = args;
        self
    }
    
    /// 设置描述
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
    
    /// 设置是否需要管理员权限
    pub fn with_admin_privileges(mut self, require_admin: bool) -> Self {
        self.run_as_admin = require_admin;
        self
    }
    
    /// 设置延迟启动
    pub fn with_delay(mut self, delay_seconds: u32) -> Self {
        self.delay_seconds = Some(delay_seconds);
        self
    }
    
    /// 设置工作目录
    pub fn with_working_directory(mut self, working_dir: String) -> Self {
        self.working_directory = Some(working_dir);
        self
    }
    
    /// 设置环境变量
    pub fn with_environment_variables(mut self, env_vars: HashMap<String, String>) -> Self {
        self.environment_variables = Some(env_vars);
        self
    }
    
    /// 获取完整的命令行
    pub fn get_full_command_line(&self) -> String {
        if self.arguments.is_empty() {
            self.command.clone()
        } else {
            format!("{} {}", self.command, self.arguments.join(" "))
        }
    }
    
    /// 验证启动项配置
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        
        if self.command.trim().is_empty() {
            return Err("Command cannot be empty".to_string());
        }
        
        // 检查命令是否存在 TODO: 可选
        let path = std::path::Path::new(&self.command);
        if !path.exists() && !path.is_absolute() {
            // 如果不是绝对路径，可能是系统命令，这里不做严格检查
        }
        
        Ok(())
    }
} 