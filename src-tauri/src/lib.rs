use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

mod search;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub id: usize,
    pub title: String,
    pub process_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutConfig {
    pub modifiers: Vec<String>,
    pub key: String,
}

impl Default for ShortcutConfig {
    fn default() -> Self {
        Self {
            modifiers: vec!["Alt".to_string(), "Ctrl".to_string()],
            key: "Space".to_string(),
        }
    }
}

// Global state for shortcut config
static SHORTCUT_CONFIG: LazyLock<Mutex<ShortcutConfig>> = LazyLock::new(|| {
    Mutex::new(load_config())
});

fn get_config_path() -> PathBuf {
    let app_data = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(app_data).join("quick-switcher").join("config.json")
}

fn load_config() -> ShortcutConfig {
    let path = get_config_path();
    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(config) = serde_json::from_str::<ShortcutConfig>(&content) {
            return config;
        }
    }
    ShortcutConfig::default()
}

fn save_config(config: &ShortcutConfig) {
    let path = get_config_path();
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).unwrap_or_else(|_| {});
        }
    }
    fs::write(&path, serde_json::to_string(config).unwrap_or_default()).unwrap_or_else(|_| {});
}

fn parse_modifiers(mods: &[String]) -> Option<Modifiers> {
    let mut result = Modifiers::empty();
    for mod_str in mods {
        match mod_str.to_lowercase().as_str() {
            "alt" => result |= Modifiers::ALT,
            "ctrl" => result |= Modifiers::CONTROL,
            "shift" => result |= Modifiers::SHIFT,
            "win" | "super" | "meta" => result |= Modifiers::SUPER,
            _ => {}
        }
    }
    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

fn parse_key(key: &str) -> Option<Code> {
    match key.to_lowercase().as_str() {
        "space" => Some(Code::Space),
        "a" => Some(Code::KeyA),
        "b" => Some(Code::KeyB),
        "c" => Some(Code::KeyC),
        "d" => Some(Code::KeyD),
        "e" => Some(Code::KeyE),
        "f" => Some(Code::KeyF),
        "g" => Some(Code::KeyG),
        "h" => Some(Code::KeyH),
        "i" => Some(Code::KeyI),
        "j" => Some(Code::KeyJ),
        "k" => Some(Code::KeyK),
        "l" => Some(Code::KeyL),
        "m" => Some(Code::KeyM),
        "n" => Some(Code::KeyN),
        "o" => Some(Code::KeyO),
        "p" => Some(Code::KeyP),
        "q" => Some(Code::KeyQ),
        "r" => Some(Code::KeyR),
        "s" => Some(Code::KeyS),
        "t" => Some(Code::KeyT),
        "u" => Some(Code::KeyU),
        "v" => Some(Code::KeyV),
        "w" => Some(Code::KeyW),
        "x" => Some(Code::KeyX),
        "y" => Some(Code::KeyY),
        "z" => Some(Code::KeyZ),
        "f1" => Some(Code::F1),
        "f2" => Some(Code::F2),
        "f3" => Some(Code::F3),
        "f4" => Some(Code::F4),
        "f5" => Some(Code::F5),
        "f6" => Some(Code::F6),
        "f7" => Some(Code::F7),
        "f8" => Some(Code::F8),
        "f9" => Some(Code::F9),
        "f10" => Some(Code::F10),
        "f11" => Some(Code::F11),
        "f12" => Some(Code::F12),
        _ => None,
    }
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
    use windows::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
    use windows::Win32::UI::Input::KeyboardAndMouse::{SetActiveWindow, SetFocus};
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetForegroundWindow, GetWindowTextW,
        GetWindowThreadProcessId, IsIconic, IsWindowVisible, IsZoomed,
        SetForegroundWindow, ShowWindow,
        SW_RESTORE, SW_SHOW, SW_SHOWMAXIMIZED,
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

            // Get current foreground window thread
            let foreground_hwnd = GetForegroundWindow();
            let foreground_thread = GetWindowThreadProcessId(foreground_hwnd, None);
            let current_thread = GetCurrentThreadId();

            // Attach to foreground thread to allow SetForegroundWindow to work
            if foreground_thread != current_thread {
                let _ = AttachThreadInput(current_thread, foreground_thread, true);
            }

            // Check if window is minimized
            let is_iconic = IsIconic(hwnd).as_bool();

            if is_iconic {
                // If minimized, restore it
                let _ = ShowWindow(hwnd, SW_RESTORE);
            } else if IsZoomed(hwnd).as_bool() {
                // If maximized, show maximized
                let _ = ShowWindow(hwnd, SW_SHOWMAXIMIZED);
            } else {
                // Otherwise just show
                let _ = ShowWindow(hwnd, SW_SHOW);
            }

            // Force window to foreground
            let _ = SetForegroundWindow(hwnd);
            let _ = SetFocus(Some(hwnd));
            let _ = SetActiveWindow(hwnd);

            // Detach from thread
            if foreground_thread != current_thread {
                let _ = AttachThreadInput(current_thread, foreground_thread, false);
            }
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
fn get_shortcut() -> ShortcutConfig {
    SHORTCUT_CONFIG.lock().unwrap().clone()
}

#[tauri::command]
fn set_shortcut(app: tauri::AppHandle, config: ShortcutConfig) -> Result<(), String> {
    // Save to file
    save_config(&config);

    // Update global state
    {
        let mut stored = SHORTCUT_CONFIG.lock().unwrap();
        *stored = config.clone();
    }

    // Re-register shortcut
    let modifiers = parse_modifiers(&config.modifiers);
    let key = parse_key(&config.key);

    if let Some(code) = key {
        let shortcut = Shortcut::new(modifiers, code);
        // First unregister all shortcuts
        if let Err(e) = app.global_shortcut().unregister_all() {
            return Err(format!("Failed to unregister shortcuts: {}", e));
        }
        // Register new shortcut with callback
        let app_handle = app.clone();
        if let Err(e) = app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, event| {
            if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                let window = app_handle.get_webview_window("main").unwrap();
                if window.is_visible().unwrap() {
                    window.hide().unwrap();
                } else {
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
                    let _ = app_handle.emit("window-shown", ());
                }
            }
        }) {
            return Err(format!("Failed to register shortcut: {}", e));
        }
        Ok(())
    } else {
        Err("Invalid key".to_string())
    }
}

#[tauri::command]
fn open_settings(app: tauri::AppHandle) {
    // Check if settings window already exists
    if app.get_webview_window("settings").is_none() {
        WebviewWindowBuilder::new(
            &app,
            "settings",
            WebviewUrl::App("index.html?settings".into()),
        )
        .title("Settings")
        .inner_size(400.0, 500.0)
        .resizable(false)
        .decorations(true)
        .build()
        .unwrap();
    } else {
        let window = app.get_webview_window("settings").unwrap();
        window.show().unwrap();
        window.set_focus().unwrap();
    }
}

#[tauri::command]
fn close_settings(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("settings") {
        window.close().unwrap();
    }
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            // Create tray menu
            let show_item = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
            let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &settings_item, &quit_item])?;

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
                            let _ = app.emit("window-shown", ());
                        }
                        "settings" => {
                            if app.get_webview_window("settings").is_none() {
                                WebviewWindowBuilder::new(
                                    app,
                                    "settings",
                                    WebviewUrl::App("index.html?settings".into()),
                                )
                                .title("Settings - Quick Switcher")
                                .inner_size(400.0, 500.0)
                                .resizable(false)
                                .decorations(true)
                                .build()
                                .unwrap();
                            } else {
                                let window = app.get_webview_window("settings").unwrap();
                                window.show().unwrap();
                                window.set_focus().unwrap();
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            // Setup global shortcut from config
            let config = SHORTCUT_CONFIG.lock().unwrap().clone();
            let modifiers = parse_modifiers(&config.modifiers);
            let key = parse_key(&config.key);

            if let Some(code) = key {
                let shortcut = Shortcut::new(modifiers, code);
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
                            // Emit event to clear search
                            let _ = app_handle.emit("window-shown", ());
                        }
                    }
                })?;
            }

            // Hide window when it loses focus
            let main_window = app.get_webview_window("main").unwrap();
            let window_clone = main_window.clone();
            main_window.on_window_event(move |event| {
                if let tauri::WindowEvent::Focused(false) = event {
                    window_clone.hide().unwrap();
                }
            });

            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_windows,
            search_windows,
            switch_window,
            hide_window,
            get_shortcut,
            set_shortcut,
            open_settings,
            close_settings
        ])
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