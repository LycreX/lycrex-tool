#[cfg(target_os = "windows")]
use windows::core::PCWSTR;

use std::thread;
use std::time::Duration;

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM, COLORREF, HINSTANCE, RECT};
#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateFontW, CreateSolidBrush, DeleteObject, DrawTextW, DT_CENTER, DT_WORDBREAK,
    EndPaint, PAINTSTRUCT, SelectObject, SetBkColor, SetTextColor,
};
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW, GetSystemMetrics,
    MSG, PostQuitMessage, RegisterClassW, ShowWindow, SW_SHOW, TranslateMessage, WINDOW_EX_STYLE,
    WNDCLASSW, WS_POPUP, WS_VISIBLE, WM_DESTROY, WM_PAINT, WM_CLOSE, PostMessageW, SM_CXSCREEN, SM_CYSCREEN,
};
#[cfg(target_os = "windows")]
use std::ffi::OsStr;
#[cfg(target_os = "windows")]
use std::os::windows::ffi::OsStrExt;

/// Logo
/// 
/// # Arguments
/// * `display_duration` - logo显示时间（秒），默认为1秒
pub fn display_logo(display_duration: Option<u64>) {
    let duration = display_duration.unwrap_or(1);
    
    #[cfg(target_os = "windows")]
    {
        thread::spawn(move || print_logo_windows(duration));
    }
    #[cfg(target_os = "linux")]
    {
        print_logo_default();
    }
}

/// Logo
/// 
/// # Arguments
/// * `display_duration` - 显示时间（秒）
#[cfg(target_os = "windows")]
fn print_logo_windows(display_duration: u64) {
    const CLASS_NAME: &str = "LycrexLogoWindow";
    let class_name: Vec<u16> = OsStr::new(CLASS_NAME).encode_wide().chain(Some(0)).collect();
    let window_name: Vec<u16> = OsStr::new("Lycrex Tool").encode_wide().chain(Some(0)).collect();

    unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match msg {
            WM_PAINT => {
                // 绘制消息
                let mut ps = PAINTSTRUCT::default();
                let hdc = unsafe { BeginPaint(hwnd, &mut ps) };
                
                // 背景色为黑色，文字色为白色
                unsafe {
                    SetBkColor(hdc, COLORREF(0x000000));
                    SetTextColor(hdc, COLORREF(0x00FFFFFF));
                }
                
                // 字体
                let font = unsafe { CreateFontW(
                    14, 0, 0, 0, 400, 0, 0, 0, 1, 3, 2, 0, 0, PCWSTR::null(),
                ) };
                
                let old_font = unsafe { SelectObject(hdc, font) };
                
                // 格式化显示文本
                let text = format!(
                    "{}\nLycrex Tool Version: {}\n{}",
                    crate::constants::LOGO_WINDOWS,
                    crate::constants::VERSION,
                    crate::constants::COPYRIGHT
                );
                let mut text_w: Vec<u16> = OsStr::new(&text).encode_wide().collect();
                
                // 文本绘制区域
                let mut rect = RECT {
                    left: 40,
                    top: 20,
                    right: 460,
                    bottom: 220,
                };
                
                // 绘制文本
                unsafe {
                    DrawTextW(hdc, &mut text_w, &mut rect, DT_WORDBREAK | DT_CENTER);
                }
                
                // 清理资源
                unsafe { SelectObject(hdc, old_font) };
                unsafe { DeleteObject(font) };
                unsafe { EndPaint(hwnd, &ps) };
                LRESULT(0)
            }
            WM_DESTROY => {
                // 窗口销毁
                unsafe { PostQuitMessage(0) };
                LRESULT(0)
            }
            WM_CLOSE => {
                // 窗口关闭
                unsafe { let _ = DestroyWindow(hwnd); };
                LRESULT(0)
            }
            _ => {
                // 其他消息
                unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
            },
        }
    }

    unsafe {
        // 注册窗口类
        let hinstance = HINSTANCE(0);
        let wc = WNDCLASSW {
            lpfnWndProc: Some(wnd_proc),
            hInstance: hinstance,
            lpszClassName: PCWSTR::from_raw(class_name.as_ptr()),
            hbrBackground: CreateSolidBrush(COLORREF(0x000000)),
            ..Default::default()
        };
        RegisterClassW(&wc);
        
        // 获取屏幕尺寸并计算窗口位置
        let screen_width = GetSystemMetrics(SM_CXSCREEN);
        let screen_height = GetSystemMetrics(SM_CYSCREEN);
        let window_width = 500;
        let window_height = 240;
        let x = (screen_width - window_width) / 2;
        let y = (screen_height - window_height) / 2;
        
        // 创建窗口
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            PCWSTR::from_raw(class_name.as_ptr()),
            PCWSTR::from_raw(window_name.as_ptr()),
            WS_POPUP | WS_VISIBLE,
            x,
            y,
            window_width,
            window_height,
            HWND(0),
            None,
            hinstance,
            None,
        );
        
        // 显示窗口
        ShowWindow(hwnd, SW_SHOW);
        
        // 启动定时器线程，在指定时间后关闭窗口
        let hwnd_clone = hwnd;
        thread::spawn(move || {
            thread::sleep(Duration::from_secs(display_duration));
            // 发送关闭消息到窗口
            let _result = PostMessageW(hwnd_clone, WM_CLOSE, WPARAM(0), LPARAM(0));
        });
        
        // 消息循环
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, HWND(0), 0, 0).0 > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

#[allow(dead_code)]
fn print_logo_default() {
    println!("{}", crate::constants::LOGO);
    println!("Lycrex Tool Version: {}", crate::constants::VERSION);
    println!("{}", crate::constants::COPYRIGHT);
}
