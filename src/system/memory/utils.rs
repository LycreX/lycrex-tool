/// 将字节数组转换为 u32
pub fn bytes_to_u32(bytes: &[u8]) -> u32 {
    if bytes.len() >= 4 {
        u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    } else {
        0
    }
}

/// 将字节数组转换为 u64
pub fn bytes_to_u64(bytes: &[u8]) -> u64 {
    if bytes.len() >= 8 {
        u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5], bytes[6], bytes[7]
        ])
    } else {
        0
    }
}

/// 将字节数组转换为 UTF-8 字符串
pub fn bytes_to_utf8_string(bytes: &[u8]) -> String {
    // 找到第一个空字节，截断字符串
    let end_pos = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    let trimmed_bytes = &bytes[..end_pos];
    
    match std::str::from_utf8(trimmed_bytes) {
        Ok(s) => s.to_string(),
        Err(_) => String::new(),
    }
}

/// 将字节数组转换为十六进制字符串
pub fn bytes_to_hex_string(bytes: &[u8]) -> String {
    bytes.iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(" ")
}

/// 打印字节数组的十六进制表示
pub fn print_bytes_hex(bytes: &[u8]) {
    println!("Hex: {}", bytes_to_hex_string(bytes));
}

/// 打印 u32 值（如果字节数组足够长）
pub fn print_u32(bytes: &[u8]) {
    let val = bytes_to_u32(bytes);
    if bytes.len() >= 4 && val != 0 {
        println!("u32: {}", val);
    } else {
        println!("Byte length less than 4 or value is 0, cannot parse as u32");
    }
}

/// 打印 u64 值（如果字节数组足够长）
pub fn print_u64(bytes: &[u8]) {
    let val = bytes_to_u64(bytes);
    if bytes.len() >= 8 && val != 0 {
        println!("u64: {}", val);
    } else {
        println!("Byte length less than 8 or value is 0, cannot parse as u64");
    }
}

/// 打印 UTF-8 字符串
pub fn print_utf8_string(bytes: &[u8]) {
    let s = bytes_to_utf8_string(bytes);
    if !s.is_empty() {
        println!("UTF-8: {}", s);
    } else {
        println!("Not a valid UTF-8 string or empty");
    }
}

/// 检查字节数组是否全为零
pub fn is_zero_bytes(bytes: &[u8]) -> bool {
    bytes.iter().all(|&b| b == 0)
}

/// 查找字节模式在数组中的位置
pub fn find_pattern(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    
    for i in 0..=(haystack.len() - needle.len()) {
        if &haystack[i..i + needle.len()] == needle {
            return Some(i);
        }
    }
    None
}

/// 将地址格式化为十六进制字符串
pub fn format_address(address: usize) -> String {
    format!("0x{:X}", address)
}

/// 将大小格式化为人类可读的字符串
pub fn format_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size_f = size as f64;
    let mut unit_index = 0;
    
    while size_f >= 1024.0 && unit_index < UNITS.len() - 1 {
        size_f /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", size, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size_f, UNITS[unit_index])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytes_to_u32() {
        let bytes = [0x12, 0x34, 0x56, 0x78];
        assert_eq!(bytes_to_u32(&bytes), 0x78563412);
        
        let short_bytes = [0x12, 0x34];
        assert_eq!(bytes_to_u32(&short_bytes), 0);
    }

    #[test]
    fn test_bytes_to_u64() {
        let bytes = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0];
        assert_eq!(bytes_to_u64(&bytes), 0xF0DEBC9A78563412);
        
        let short_bytes = [0x12, 0x34, 0x56];
        assert_eq!(bytes_to_u64(&short_bytes), 0);
    }

    #[test]
    fn test_bytes_to_utf8_string() {
        let bytes = b"Hello\0World";
        assert_eq!(bytes_to_utf8_string(bytes), "Hello");
        
        let bytes_no_null = b"Hello";
        assert_eq!(bytes_to_utf8_string(bytes_no_null), "Hello");
        
        let invalid_utf8 = [0xFF, 0xFE, 0xFD];
        assert_eq!(bytes_to_utf8_string(&invalid_utf8), "");
    }

    #[test]
    fn test_find_pattern() {
        let haystack = b"Hello World, Hello Universe";
        let needle = b"Hello";
        
        assert_eq!(find_pattern(haystack, needle), Some(0));
        
        let needle2 = b"World";
        assert_eq!(find_pattern(haystack, needle2), Some(6));
        
        let needle3 = b"NotFound";
        assert_eq!(find_pattern(haystack, needle3), None);
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1536), "1.50 KB");
        assert_eq!(format_size(1048576), "1.00 MB");
    }
}
