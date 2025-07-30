pub mod logo;
pub mod logger;

static mut IS_INIT: bool = false;

/// 初始化lycrex工具
#[deprecated(since = "0.0.0", note = "由于目前程序时间会自动初始化，所以这里不需要手动初始化，等到更多功能需要时再手动初始化")]
pub fn init_lycrex_tool() {
    use crate::utils::time::init_program_time;
    init_program_time();
    unsafe {
        IS_INIT = true;
    }
}

/// 检查lycrex工具是否初始化
#[deprecated(since = "0.0.0", note = "由于目前程序时间会自动初始化，所以这里不需要手动初始化，等到更多功能需要时再手动初始化")]
pub fn is_init() -> bool {
    unsafe {
        IS_INIT
    }
}