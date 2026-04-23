use crate::gesture_os_control::domain::entities::context::ForegroundWindowInfo;

#[cfg(windows)]
pub fn read_foreground_window() -> Option<ForegroundWindowInfo> {
    use std::path::Path;

    use windows::Win32::Foundation::{CloseHandle, MAX_PATH};
    use windows::Win32::System::Threading::{
        OpenProcess, PROCESS_NAME_FORMAT, PROCESS_QUERY_LIMITED_INFORMATION,
        QueryFullProcessImageNameW,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
    };
    use windows::core::PWSTR;

    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.0.is_null() {
        return None;
    }

    let title_len = unsafe { GetWindowTextLengthW(hwnd) }.max(0) as usize;
    let mut title_buf = vec![0u16; title_len.saturating_add(1)];
    let copied = unsafe { GetWindowTextW(hwnd, &mut title_buf) };
    let window_title = String::from_utf16_lossy(&title_buf[..copied.max(0) as usize])
        .trim()
        .to_owned();

    let mut pid = 0u32;
    unsafe {
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
    }
    if pid == 0 {
        return Some(ForegroundWindowInfo {
            process_name: String::new(),
            window_title,
        });
    }

    let process_handle =
        unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()? };
    let mut path_buf = vec![0u16; MAX_PATH as usize];
    let mut size = path_buf.len() as u32;
    let full_process_path = unsafe {
        let ok = QueryFullProcessImageNameW(
            process_handle,
            PROCESS_NAME_FORMAT(0),
            PWSTR(path_buf.as_mut_ptr()),
            &mut size,
        )
        .is_ok();
        let _ = CloseHandle(process_handle);
        if !ok {
            return Some(ForegroundWindowInfo {
                process_name: String::new(),
                window_title,
            });
        }
        String::from_utf16_lossy(&path_buf[..size as usize])
    };

    let process_name = Path::new(&full_process_path)
        .file_name()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or(full_process_path);

    Some(ForegroundWindowInfo {
        process_name,
        window_title,
    })
}

#[cfg(not(windows))]
pub fn read_foreground_window() -> Option<ForegroundWindowInfo> {
    None
}
