pub trait IntoLines {
    fn into_lines(self) -> Vec<String>;
}

impl IntoLines for &str {
    fn into_lines(self) -> Vec<String> {
        self.split('\n').map(|s| s.to_string()).collect()
    }
}

impl IntoLines for String {
    fn into_lines(self) -> Vec<String> {
        self.split('\n').map(|s| s.to_string()).collect()
    }
}

impl IntoLines for &[String] {
    fn into_lines(self) -> Vec<String> {
        let mut lines = Vec::new();
        for line in self {
            if line.contains('\n') {
                for sub_line in line.split('\n') {
                    lines.push(sub_line.to_string());
                }
            } else {
                lines.push(line.clone());
            }
        }
        lines
    }
}

impl IntoLines for Vec<String> {
    fn into_lines(self) -> Vec<String> {
        let mut lines = Vec::new();
        for line in self {
            if line.contains('\n') {
                for sub_line in line.split('\n') {
                    lines.push(sub_line.to_string());
                }
            } else {
                lines.push(line);
            }
        }
        lines
    }
}

impl IntoLines for &Vec<String> {
    fn into_lines(self) -> Vec<String> {
        let mut lines = Vec::new();
        for line in self {
            if line.contains('\n') {
                for sub_line in line.split('\n') {
                    lines.push(sub_line.to_string());
                }
            } else {
                lines.push(line.clone());
            }
        }
        lines
    }
}

/// 创建信息框，自动计算大小并居中文字
pub fn create_info_box<T: IntoLines>(input: T, width: Option<usize>, center_align: bool) -> String {
    let lines = input.into_lines();
    
    if lines.is_empty() {
        return String::new();
    }
    
    let max_width = lines.iter()
        .map(|line| calculate_display_width(line))
        .max()
        .unwrap_or(0);
    
    let min_width = 30;
    let padding = 2;

    let box_width = match width {
        Some(w) => if w < max_width { max_width + padding * 2 } else { w },
        None => (max_width + padding * 2).max(min_width),
    };

    
    let mut result = String::new();
    
    // 顶部边框
    result.push('\n');
    result.push_str("        ┏");
    result.push_str(&"━".repeat(box_width));
    result.push_str("┓\n");
    
    for line in &lines {
        result.push_str("        ┃");
        
        if line.is_empty() {
            // 空行
            result.push_str(&" ".repeat(box_width));
        } else {
            let line_width = calculate_display_width(line);
            if center_align {
                // 居中对齐
                let left_padding = (box_width - line_width) / 2;
                let right_padding = box_width - line_width - left_padding;
                
                result.push_str(&" ".repeat(left_padding));
                result.push_str(line);
                result.push_str(&" ".repeat(right_padding));
            } else {
                // 左对齐
                let left_padding = 2;
                let right_padding = box_width - line_width - left_padding;
                
                result.push_str(&" ".repeat(left_padding));
                result.push_str(line);
                result.push_str(&" ".repeat(right_padding));
            }
        }
        
        result.push_str("┃\n");
    }
    
    // 底部边框
    result.push_str("        ┗");
    result.push_str(&"━".repeat(box_width));
    result.push_str("┛\n");
    result.push_str("        ");
    
    result
}

/// 计算字符串的显示宽度（忽略ANSI转义序列）
fn calculate_display_width(s: &str) -> usize {
    // 移除ANSI转义序列后再计算宽度
    let clean_str = strip_ansi_codes(s);
    clean_str.chars().map(|c| {
        if is_cjk_character(c) {
            2
        } else {
            1
        }
    }).sum()
}

/// 移除字符串中的ANSI转义序列
fn strip_ansi_codes(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars();
    
    while let Some(ch) = chars.next() {
        if ch == '\x1b' {  // ESC字符
            // 跳过整个ANSI转义序列
            if let Some('[') = chars.next() {
                // 跳过参数部分（数字、分号、冒号等）
                while let Some(c) = chars.next() {
                    if c.is_ascii_alphabetic() {
                        // 遇到字母表示序列结束
                        break;
                    }
                }
            }
        } else {
            result.push(ch);
        }
    }
    
    result
}

/// 判断是否为中日韩(CJK)字符
pub fn is_cjk_character(c: char) -> bool {
    let code = c as u32;
    (0x4E00..=0x9FFF).contains(&code)   ||   // CJK统一汉字
    (0x3400..=0x4DBF).contains(&code)   ||   // CJK扩展A
    (0x20000..=0x2A6DF).contains(&code) ||   // CJK扩展B
    (0x2A700..=0x2B73F).contains(&code) ||   // CJK扩展C
    (0x2B740..=0x2B81F).contains(&code) ||   // CJK扩展D
    (0x2B820..=0x2CEAF).contains(&code) ||   // CJK扩展E
    (0x3000..=0x303F).contains(&code)   ||   // CJK符号和标点
    (0xFF00..=0xFFEF).contains(&code)        // 全角ASCII
}