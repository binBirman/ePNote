use crate::util::path::error::SanitizeError;

/// Windows 下的一些保留文件名，需要拒绝
const WINDOWS_RESERVED: &[&str] = &[
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// 验证并清理文件名，返回安全的文件名字符串或 `SanitizeError`。
///
/// 检查项：
/// - 非空
/// - 不包含路径分隔符或 `..`（防止路径穿越）
/// - 不包含非法字符（冒号、星号等）
/// - 不以点开头或点结尾
/// - 不为 Windows 保留名
pub(crate) fn sanitize_filename(input: &str) -> Result<String, SanitizeError> {
    // 1 空
    if input.is_empty() {
        return Err(SanitizeError::Empty);
    }

    // 2 路径分隔符
    if input.contains('/') || input.contains('\\') {
        return Err(SanitizeError::PathTraversal);
    }

    // 3 相对路径
    if input.contains("..") {
        return Err(SanitizeError::PathTraversal);
    }

    // 4 非法字符
    let illegal = [':', '*', '?', '<', '>', '|', '"', '\t', '\n'];

    if input.chars().any(|c| illegal.contains(&c)) {
        return Err(SanitizeError::IllegalChar);
    }

    // 5 隐藏文件
    if input.starts_with('.') {
        return Err(SanitizeError::HiddenFile);
    }

    // 6 尾点
    if input.ends_with('.') {
        return Err(SanitizeError::TrailingDot);
    }

    // 7 Windows 保留名
    let upper = input.to_ascii_uppercase();
    if WINDOWS_RESERVED.contains(&upper.as_str()) {
        return Err(SanitizeError::ReservedName);
    }

    Ok(input.to_string())
}
