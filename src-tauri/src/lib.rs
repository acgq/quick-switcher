use serde::{Deserialize, Serialize};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::Manager;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

mod search;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub id: usize,
    pub title: String,
    pub process_name: String,
}

/// Extract process name from a full path string.
/// Returns the last segment after the last backslash, or empty string if not found.
pub fn extract_process_name_from_path(path: &str) -> String {
    path.rsplit('\\').next().unwrap_or("").to_string()
}

/// Check if a window should be included in the window list.
/// Windows are included if they are visible and have a non-empty title.
pub fn should_include_window(title: &str, is_visible: bool) -> bool {
    is_visible && !title.is_empty()
}

#[cfg(target_os = "windows")]
mod platform {
    use super::WindowInfo;
    use super::extract_process_name_from_path;
    use windows_core::BOOL;
    use windows::Win32::Foundation::{HWND, LPARAM, CloseHandle, TRUE};
    use windows::Win32::System::ProcessStatus::GetModuleFileNameExW;
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible,
        SetForegroundWindow, ShowWindow, SW_RESTORE,
    };

    pub fn get_windows() -> Vec<WindowInfo> {
        let mut windows = Vec::new();
        unsafe {
            let _ = EnumWindows(Some(enum_windows_callback), LPARAM(&mut windows as *mut _ as isize));
        }
        windows
    }

    unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let windows = &mut *(lparam.0 as *mut Vec<WindowInfo>);

        if !IsWindowVisible(hwnd).as_bool() {
            return TRUE;
        }

        let mut title = [0u16; 512];
        let len = GetWindowTextW(hwnd, &mut title);
        if len == 0 {
            return TRUE;
        }

        let title_str = String::from_utf16_lossy(&title[..len as usize]);
        if title_str.is_empty() {
            return TRUE;
        }

        let mut process_id: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));

        let process_name = get_process_name(process_id);

        windows.push(WindowInfo {
            id: hwnd.0 as usize,
            title: title_str,
            process_name,
        });

        TRUE
    }

    fn get_process_name(pid: u32) -> String {
        unsafe {
            let handle = match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
                Ok(h) => h,
                Err(_) => return String::new(),
            };

            let mut name = [0u16; 260];
            let len = GetModuleFileNameExW(Some(handle), None, &mut name);
            let _ = CloseHandle(handle);

            if len == 0 {
                return String::new();
            }

            let path = String::from_utf16_lossy(&name[..len as usize]);
            extract_process_name_from_path(&path)
        }
    }

    pub fn switch_window(id: usize) {
        unsafe {
            let hwnd = HWND(id as *mut std::ffi::c_void);
            let _ = ShowWindow(hwnd, SW_RESTORE);
            let _ = SetForegroundWindow(hwnd);
        }
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use super::WindowInfo;
    pub fn get_windows() -> Vec<WindowInfo> { vec![] }
    pub fn switch_window(_id: usize) {}
}

#[cfg(target_os = "linux")]
mod platform {
    use super::WindowInfo;
    pub fn get_windows() -> Vec<WindowInfo> { vec![] }
    pub fn switch_window(_id: usize) {}
}

#[tauri::command]
fn get_windows() -> Vec<WindowInfo> {
    platform::get_windows()
}

#[tauri::command]
fn search_windows(windows: Vec<WindowInfo>, query: String) -> Vec<WindowInfo> {
    if query.is_empty() {
        return windows;
    }

    // Filter by search query
    let mut filtered: Vec<WindowInfo> = windows
        .into_iter()
        .filter(|w| {
            search::matches(&w.title, &query) || search::matches(&w.process_name, &query)
        })
        .collect();

    // Sort by match score
    filtered.sort_by(|a, b| {
        let score_a = search::match_score(&a.title, &query).max(search::match_score(&a.process_name, &query));
        let score_b = search::match_score(&b.title, &query).max(search::match_score(&b.process_name, &query));
        score_b.cmp(&score_a)
    });

    filtered
}

#[tauri::command]
fn switch_window(window_id: usize) {
    platform::switch_window(window_id);
}

#[tauri::command]
fn hide_window(app: tauri::AppHandle) {
    let window = app.get_webview_window("main").unwrap();
    window.hide().unwrap();
}

#[tauri::command]
fn exit_app(app: tauri::AppHandle) {
    app.exit(0);
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            // Create tray menu
            let show_item = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

            // Build tray icon
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "show" => {
                            let window = app.get_webview_window("main").unwrap();
                            if let Some(screen) = window.primary_monitor().unwrap_or(None) {
                                let screen_size = screen.size();
                                let window_size = window.outer_size().unwrap_or(tauri::PhysicalSize { width: 600, height: 400 });
                                let x = (screen_size.width.saturating_sub(window_size.width)) / 2;
                                let y = (screen_size.height.saturating_sub(window_size.height)) / 2;
                                window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                                    x: x as i32,
                                    y: y as i32,
                                })).unwrap();
                            }
                            window.show().unwrap();
                            window.set_focus().unwrap();
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            // Setup global shortcut
            let shortcut = Shortcut::new(Some(Modifiers::ALT | Modifiers::CONTROL), Code::Space);
            let app_handle = app.handle().clone();

            app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, event| {
                // Only respond to key press, not release
                if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                    let window = app_handle.get_webview_window("main").unwrap();
                    if window.is_visible().unwrap() {
                        window.hide().unwrap();
                    } else {
                        // Center the window on screen
                        if let Some(screen) = window.primary_monitor().unwrap_or(None) {
                            let screen_size = screen.size();
                            let window_size = window.outer_size().unwrap_or(tauri::PhysicalSize { width: 600, height: 400 });
                            let x = (screen_size.width.saturating_sub(window_size.width)) / 2;
                            let y = (screen_size.height.saturating_sub(window_size.height)) / 2;
                            window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                                x: x as i32,
                                y: y as i32,
                            })).unwrap();
                        }
                        window.show().unwrap();
                        window.set_focus().unwrap();
                    }
                }
            })?;

            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_windows, search_windows, switch_window, hide_window, exit_app])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    // WindowInfo tests
    #[test]
    fn test_window_info_serde() {
        let window = WindowInfo {
            id: 123,
            title: "Test Window".to_string(),
            process_name: "test.exe".to_string(),
        };

        // Serialize
        let json = serde_json::to_string(&window).unwrap();
        assert!(json.contains("\"id\":123"));
        assert!(json.contains("\"title\":\"Test Window\""));
        assert!(json.contains("\"process_name\":\"test.exe\""));

        // Deserialize
        let decoded: WindowInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.id, 123);
        assert_eq!(decoded.title, "Test Window");
        assert_eq!(decoded.process_name, "test.exe");
    }

    #[test]
    fn test_window_info_clone() {
        let window = WindowInfo {
            id: 1,
            title: "Original".to_string(),
            process_name: "app.exe".to_string(),
        };
        let cloned = window.clone();
        assert_eq!(window.id, cloned.id);
        assert_eq!(window.title, cloned.title);
        assert_eq!(window.process_name, cloned.process_name);
    }

    // extract_process_name_from_path tests
    #[test]
    fn test_extract_process_name_normal_path() {
        let path = "C:\\Program Files\\App\\app.exe";
        let name = extract_process_name_from_path(path);
        assert_eq!(name, "app.exe");
    }

    #[test]
    fn test_extract_process_name_empty_path() {
        let name = extract_process_name_from_path("");
        assert_eq!(name, "");
    }

    #[test]
    fn test_extract_process_name_no_backslash() {
        let path = "app.exe";
        let name = extract_process_name_from_path(path);
        assert_eq!(name, "app.exe");
    }

    #[test]
    fn test_extract_process_name_single_backslash() {
        let path = "C:\\app.exe";
        let name = extract_process_name_from_path(path);
        assert_eq!(name, "app.exe");
    }

    #[test]
    fn test_extract_process_name_trailing_backslash() {
        let path = "C:\\Program Files\\";
        let name = extract_process_name_from_path(path);
        assert_eq!(name, "");
    }

    // should_include_window tests
    #[test]
    fn test_should_include_window_visible_with_title() {
        assert!(should_include_window("Test", true));
    }

    #[test]
    fn test_should_include_window_hidden() {
        assert!(!should_include_window("Test", false));
    }

    #[test]
    fn test_should_include_window_empty_title() {
        assert!(!should_include_window("", true));
    }

    #[test]
    fn test_should_include_window_hidden_empty() {
        assert!(!should_include_window("", false));
    }

    #[test]
    fn test_should_include_window_whitespace_title() {
        // Whitespace-only title is considered non-empty by is_empty()
        assert!(should_include_window("  ", true));
    }
}