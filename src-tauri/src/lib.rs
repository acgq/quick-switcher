use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};
use std::thread;
use std::time::Duration;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

mod search;

#[cfg(target_os = "linux")]
mod kwin;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub id: String,  // Use string ID to avoid JavaScript number precision issues
    pub title: String,
    pub process_name: String,
}

// Global cache for window list, updated periodically in background
static WINDOW_CACHE: LazyLock<Mutex<Vec<WindowInfo>>> = LazyLock::new(|| {
    Mutex::new(Vec::new())
});

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

/// Center window on the monitor where mouse cursor is located
fn center_window_on_mouse_monitor(window: &tauri::WebviewWindow) {
    // Get cursor position
    let cursor_pos = match window.cursor_position() {
        Ok(pos) => pos,
        Err(_) => {
            // Fallback to primary monitor
            if let Some(screen) = window.primary_monitor().unwrap_or(None) {
                center_on_monitor(window, &screen);
            }
            return;
        }
    };

    // Find the monitor containing the cursor
    let monitors = match window.available_monitors() {
        Ok(monitors) => monitors,
        Err(_) => {
            if let Some(screen) = window.primary_monitor().unwrap_or(None) {
                center_on_monitor(window, &screen);
            }
            return;
        }
    };

    // Find monitor that contains the cursor
    for monitor in monitors {
        let position = monitor.position();
        let size = monitor.size();

        // Check if cursor is within this monitor's bounds
        if cursor_pos.x >= position.x as f64
            && cursor_pos.x < (position.x + size.width as i32) as f64
            && cursor_pos.y >= position.y as f64
            && cursor_pos.y < (position.y + size.height as i32) as f64
        {
            center_on_monitor(window, &monitor);
            return;
        }
    }

    // Fallback to primary monitor
    if let Some(screen) = window.primary_monitor().unwrap_or(None) {
        center_on_monitor(window, &screen);
    }
}

fn center_on_monitor(window: &tauri::WebviewWindow, monitor: &tauri::Monitor) {
    let screen_size = monitor.size();
    let screen_position = monitor.position();
    let window_size = window.outer_size().unwrap_or(tauri::PhysicalSize { width: 600, height: 400 });

    let x = screen_position.x + ((screen_size.width.saturating_sub(window_size.width)) / 2) as i32;
    let y = screen_position.y + ((screen_size.height.saturating_sub(window_size.height)) / 2) as i32;

    let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }));
}

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
    use windows::Win32::Foundation::{HWND, LPARAM, CloseHandle, RECT, TRUE};
    use windows::Win32::System::ProcessStatus::GetModuleFileNameExW;
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowTextW, GetWindowThreadProcessId, GetWindowRect,
        GetWindowLongPtrW, GetAncestor, IsIconic, IsWindowVisible, IsZoomed, ShowWindow, SwitchToThisWindow,
        GA_ROOT, GWL_EXSTYLE, SW_RESTORE, SW_SHOW, SW_SHOWMAXIMIZED, WS_EX_APPWINDOW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
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

        // Must be visible
        if !IsWindowVisible(hwnd).as_bool() {
            return TRUE;
        }

        // Must have a title
        let mut title = [0u16; 512];
        let len = GetWindowTextW(hwnd, &mut title);
        if len == 0 {
            return TRUE;
        }
        let title_str = String::from_utf16_lossy(&title[..len as usize]);
        if title_str.is_empty() {
            return TRUE;
        }

        // Filter tool windows, but allow them if they have WS_EX_APPWINDOW
        // This matches Alt+Tab behavior
        let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        let is_tool_window = (ex_style & WS_EX_TOOLWINDOW.0 as isize) != 0;
        let is_app_window = (ex_style & WS_EX_APPWINDOW.0 as isize) != 0;
        if is_tool_window && !is_app_window {
            return TRUE;
        }

        // Filter out windows that cannot be activated (like "Windows Input Experience")
        if (ex_style & WS_EX_NOACTIVATE.0 as isize) != 0 {
            return TRUE;
        }

        // Filter out child windows (only show root windows like Alt+Tab)
        if GetAncestor(hwnd, GA_ROOT) != hwnd {
            return TRUE;
        }

        // Filter out windows with zero size
        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_err() {
            return TRUE;
        }
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;
        if width <= 0 || height <= 0 {
            return TRUE;
        }

        // Get process info
        let mut process_id: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));
        let process_name = get_process_name(process_id);

        // Filter out Quick Switcher's own windows
        if process_name.to_lowercase().contains("quick-switcher") {
            return TRUE;
        }

        windows.push(WindowInfo {
            id: (hwnd.0 as usize).to_string(),
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

    pub fn switch_window(id: String) {
        if let Ok(window_id) = id.parse::<usize>() {
            unsafe {
                let hwnd = HWND(window_id as *mut std::ffi::c_void);

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

                // Use SwitchToThisWindow to forcefully bring window to front
                // This is more reliable than SetForegroundWindow for some apps
                SwitchToThisWindow(hwnd, true);
            }
        }
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use super::WindowInfo;
    use objc::{class, msg_send, sel, sel_impl};
    use std::ffi::CStr;

    // On macOS, BOOL is actually bool, so we use bool directly
    type ObjcId = *mut objc::runtime::Object;

    pub fn get_windows() -> Vec<WindowInfo> {
        unsafe {
            // Use NSWorkspace to get running applications
            let workspace: ObjcId = msg_send![class!(NSWorkspace), sharedWorkspace];
            let apps: ObjcId = msg_send![workspace, runningApplications];

            if apps.is_null() {
                return Vec::new();
            }

            let count: usize = msg_send![apps, count];
            let mut windows = Vec::new();

            for i in 0..count {
                let app: ObjcId = msg_send![apps, objectAtIndex: i];

                // Skip hidden apps (BOOL is bool on macOS)
                let hidden: bool = msg_send![app, isHidden];
                if hidden {
                    continue;
                }

                // Get app name
                let name: ObjcId = msg_send![app, localizedName];
                let process_name = nsstring_to_string(name);

                // Filter out Quick Switcher (match both "Quick Switcher" and "quick-switcher")
                let name_lower = process_name.to_lowercase();
                if name_lower.contains("quick-switcher") {
                    continue;
                }

                // Get window title using AXUIElement (requires accessibility permission)
                let pid: i32 = msg_send![app, processIdentifier];
                let title = get_window_title_for_pid(pid);

                if title.is_empty() {
                    // If no window title, use app name as fallback for apps with UI
                    // Check if app has UI (activation policy)
                    let activation_policy: i32 = msg_send![app, activationPolicy];
                    // NSApplicationActivationPolicyRegular = 0
                    if activation_policy != 0 {
                        continue;
                    }
                    windows.push(WindowInfo {
                        id: (pid as usize).to_string(),
                        title: process_name.clone(),
                        process_name,
                    });
                } else {
                    windows.push(WindowInfo {
                        id: (pid as usize).to_string(),
                        title,
                        process_name,
                    });
                }
            }

            windows
        }
    }

    unsafe fn nsstring_to_string(ns_str: ObjcId) -> String {
        if ns_str.is_null() {
            return String::new();
        }
        let bytes: *const i8 = msg_send![ns_str, UTF8String];
        if bytes.is_null() {
            return String::new();
        }
        CStr::from_ptr(bytes).to_string_lossy().into_owned()
    }

    unsafe fn get_window_title_for_pid(pid: i32) -> String {
        // Create AXUIElement for the application
        let app_element = AXUIElementCreateApplication(pid);
        if app_element.is_null() {
            return String::new();
        }

        // Get the focused window
        let mut window: *mut std::ffi::c_void = std::ptr::null_mut();
        let attr_name = CFStringCreateWithCString(
            std::ptr::null_mut(),
            "AXFocusedWindow\0".as_ptr() as *const i8,
            0x08000100, // kCFStringEncodingUTF8
        );

        let result = AXUIElementCopyAttributeValue(app_element, attr_name, &mut window);
        CFRelease(app_element);
        if !attr_name.is_null() {
            CFRelease(attr_name);
        }

        if result != 0 || window.is_null() {
            return String::new();
        }

        // Get window title
        let mut title: *mut std::ffi::c_void = std::ptr::null_mut();
        let title_attr = CFStringCreateWithCString(
            std::ptr::null_mut(),
            "AXTitle\0".as_ptr() as *const i8,
            0x08000100,
        );

        let result = AXUIElementCopyAttributeValue(window, title_attr, &mut title);
        CFRelease(window);
        if !title_attr.is_null() {
            CFRelease(title_attr);
        }

        if result != 0 || title.is_null() {
            return String::new();
        }

        // Convert CFString to Rust String
        let length = CFStringGetLength(title);
        let mut buffer = vec![0u16; length as usize + 1];
        CFStringGetCharacters(title, CFRange { location: 0, length }, buffer.as_mut_ptr());
        CFRelease(title);

        String::from_utf16_lossy(&buffer[..length as usize])
    }

    // CoreFoundation / Accessibility externs
    #[repr(C)]
    struct CFRange {
        location: isize,
        length: isize,
    }

    extern "C" {
        fn AXUIElementCreateApplication(pid: i32) -> *mut std::ffi::c_void;
        fn AXUIElementCopyAttributeValue(
            element: *mut std::ffi::c_void,
            attribute: *mut std::ffi::c_void,
            value: *mut *mut std::ffi::c_void,
        ) -> i32;
        fn CFStringCreateWithCString(
            alloc: *mut std::ffi::c_void,
            c_str: *const i8,
            encoding: u32,
        ) -> *mut std::ffi::c_void;
        fn CFStringGetLength(cf_str: *mut std::ffi::c_void) -> isize;
        fn CFStringGetCharacters(cf_str: *mut std::ffi::c_void, range: CFRange, buffer: *mut u16);
        fn CFRelease(cf: *mut std::ffi::c_void);
    }

    pub fn switch_window(window_id: String) {
        if let Ok(pid) = window_id.parse::<usize>() {
            unsafe {
                let workspace: ObjcId = msg_send![class!(NSWorkspace), sharedWorkspace];
                let apps: ObjcId = msg_send![workspace, runningApplications];

                if apps.is_null() {
                    return;
                }

                let count: usize = msg_send![apps, count];

                for i in 0..count {
                    let app: ObjcId = msg_send![apps, objectAtIndex: i];
                    let app_pid: i32 = msg_send![app, processIdentifier];

                    if app_pid as usize == pid {
                        // NSApplicationActivateIgnoringOtherApps = 1 << 1
                        let _: () = msg_send![app, activateWithOptions: 2];
                        break;
                    }
                }
            }
        }
    }
}

#[cfg(target_os = "linux")]
mod platform {
    use super::WindowInfo;
    use std::fs;

    #[derive(Debug, Clone, Copy, PartialEq)]
    enum DisplayServer {
        X11,
        Wayland,
    }

    fn detect_display_server() -> DisplayServer {
        if std::env::var("WAYLAND_DISPLAY").is_ok()
            || std::env::var("XDG_SESSION_TYPE")
                .map(|v| v == "wayland")
                .unwrap_or(false)
        {
            DisplayServer::Wayland
        } else {
            DisplayServer::X11
        }
    }

    /// Extract process name from /proc/{pid}/comm or /proc/{pid}/cmdline
    fn get_process_name_from_pid(pid: u32) -> String {
        // Try /proc/{pid}/comm first (shorter name)
        let comm_path = format!("/proc/{}/comm", pid);
        if let Ok(name) = fs::read_to_string(&comm_path) {
            return name.trim().to_string();
        }

        // Fallback to /proc/{pid}/cmdline
        let cmdline_path = format!("/proc/{}/cmdline", pid);
        if let Ok(cmdline) = fs::read_to_string(&cmdline_path) {
            // cmdline is null-separated, get first argument
            let first_arg = cmdline.split('\0').next().unwrap_or("");
            // Extract just the executable name from path
            first_arg
                .rsplit('/')
                .next()
                .unwrap_or(first_arg)
                .to_string()
        } else {
            String::new()
        }
    }

    mod x11_backend {
        use super::super::WindowInfo;
        use super::get_process_name_from_pid;
        use x11rb::connection::Connection;
        use x11rb::protocol::xproto::*;
        use x11rb::rust_connection::RustConnection;
        use x11rb::x11_utils::Serialize;
        use x11rb::CURRENT_TIME;

        // X11 ANY atom value (0 means any type)
        const ANY_ATOM: Atom = 0;

        const _NET_CLIENT_LIST: &str = "_NET_CLIENT_LIST";
        const _NET_WM_NAME: &str = "_NET_WM_NAME";
        const _NET_WM_PID: &str = "_NET_WM_PID";
        const _NET_ACTIVE_WINDOW: &str = "_NET_ACTIVE_WINDOW";

        pub fn get_windows() -> Vec<WindowInfo> {
            let (conn, screen_num) = match RustConnection::connect(None) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("[X11] Connection failed: {:?}", e);
                    return Vec::new();
                }
            };

            let screen = match conn.setup().roots.get(screen_num) {
                Some(s) => s,
                None => return Vec::new(),
            };
            let root = screen.root;

            // Get _NET_CLIENT_LIST atom
            let client_list_atom = match conn.intern_atom(false, _NET_CLIENT_LIST.as_bytes()) {
                Ok(cookie) => match cookie.reply() {
                    Ok(reply) => reply.atom,
                    Err(_) => return Vec::new(),
                },
                Err(_) => return Vec::new(),
            };

            // Get _NET_WM_NAME atom
            let wm_name_atom = match conn.intern_atom(false, _NET_WM_NAME.as_bytes()) {
                Ok(cookie) => match cookie.reply() {
                    Ok(reply) => reply.atom,
                    Err(_) => 0,
                },
                Err(_) => 0,
            };

            // Get _NET_WM_PID atom
            let wm_pid_atom = match conn.intern_atom(false, _NET_WM_PID.as_bytes()) {
                Ok(cookie) => match cookie.reply() {
                    Ok(reply) => reply.atom,
                    Err(_) => 0,
                },
                Err(_) => 0,
            };

            // Get CARDINAL atom for PID property type
            let cardinal_atom = AtomEnum::CARDINAL.into();

            // Get client list - use ANY_ATOM (0) for type
            let client_list_cookie = conn
                .get_property(false, root, client_list_atom, ANY_ATOM, 0, 1024);
            let client_list = match client_list_cookie {
                Ok(cookie) => match cookie.reply() {
                    Ok(reply) => reply,
                    Err(e) => {
                        eprintln!("[X11] Failed to get client list reply: {:?}", e);
                        return Vec::new();
                    }
                },
                Err(e) => {
                    eprintln!("[X11] Failed to get client list: {:?}", e);
                    return Vec::new();
                }
            };

            eprintln!("[X11] Found {} windows", client_list.value32().map(|i| i.count()).unwrap_or(0));

            let windows: Vec<Window> = client_list
                .value32()
                .map(|iter| iter.collect())
                .unwrap_or_default();

            let mut result = Vec::new();

            for window in windows {
                // Get window title
                let title = get_window_title(&conn, window, wm_name_atom);

                // Skip windows with empty titles
                if title.is_empty() {
                    continue;
                }

                // Get process ID
                let pid = get_window_pid(&conn, window, wm_pid_atom, cardinal_atom);
                let process_name = if pid > 0 {
                    get_process_name_from_pid(pid)
                } else {
                    String::new()
                };

                // Filter out Quick Switcher's own windows
                if process_name.to_lowercase().contains("quick-switcher") {
                    continue;
                }

                result.push(WindowInfo {
                    id: (window as usize).to_string(),
                    title,
                    process_name,
                });
            }

            result
        }

        fn get_window_title(conn: &RustConnection, window: Window, wm_name_atom: Atom) -> String {
            // Try _NET_WM_NAME first (UTF-8)
            if wm_name_atom != 0 {
                let prop = conn.get_property(false, window, wm_name_atom, ANY_ATOM, 0, 1024);
                if let Ok(cookie) = prop {
                    if let Ok(reply) = cookie.reply() {
                        if !reply.value.is_empty() {
                            // _NET_WM_NAME is UTF-8 string
                            return String::from_utf8_lossy(&reply.value).into_owned();
                        }
                    }
                }
            }

            // Fallback to WM_NAME (compound text or string)
            let wm_name: Atom = AtomEnum::WM_NAME.into();
            let prop = conn.get_property(false, window, wm_name, ANY_ATOM, 0, 1024);
            if let Ok(cookie) = prop {
                if let Ok(reply) = cookie.reply() {
                    if !reply.value.is_empty() {
                        return String::from_utf8_lossy(&reply.value).into_owned();
                    }
                }
            }

            String::new()
        }

        fn get_window_pid(conn: &RustConnection, window: Window, wm_pid_atom: Atom, cardinal_atom: Atom) -> u32 {
            if wm_pid_atom == 0 {
                return 0;
            }

            let prop = conn.get_property(false, window, wm_pid_atom, cardinal_atom, 0, 1);
            if let Ok(cookie) = prop {
                if let Ok(reply) = cookie.reply() {
                    if let Some(pid) = reply.value32().and_then(|mut iter| iter.next()) {
                        return pid;
                    }
                }
            }

            0
        }

        pub fn switch_window(window_id: String) {
            if let Ok(id) = window_id.parse::<usize>() {
                let (conn, screen_num) = match RustConnection::connect(None) {
                    Ok(c) => c,
                    Err(_) => return,
                };

                let screen = match conn.setup().roots.get(screen_num) {
                    Some(s) => s,
                    None => return,
                };
                let root = screen.root;
                let window = id as Window;

                // Get _NET_ACTIVE_WINDOW atom
                let active_window_atom = match conn.intern_atom(false, _NET_ACTIVE_WINDOW.as_bytes()) {
                    Ok(cookie) => match cookie.reply() {
                        Ok(reply) => reply.atom,
                        Err(_) => return,
                    },
                    Err(_) => return,
                };

                // Send _NET_ACTIVE_WINDOW client message
                let event = ClientMessageEvent {
                    response_type: CLIENT_MESSAGE_EVENT,
                    sequence: 0,
                    format: 32,
                    window,
                    type_: active_window_atom,
                    data: [2u32, 0, 0, 0, 0].into(), // source=2 (user), timestamp=0
                };

                // Send event with SubstructureRedirectMask | SubstructureNotifyMask
                let _ = conn.send_event(
                    false,
                    root,
                    EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
                    event.serialize(),
                );

                let _ = conn.flush();

                // Also try XSetInputFocus as fallback
                let _ = conn.set_input_focus(InputFocus::POINTER_ROOT, window, CURRENT_TIME);
                let _ = conn.flush();
            }
        }
    }

    mod wayland_backend {
        use super::super::WindowInfo;
        use super::super::kwin;

        /// Get windows via KWin scripting API (KDE Wayland)
        pub fn get_windows() -> Option<Vec<WindowInfo>> {
            // Check if running KDE Plasma 6
            if !kwin::is_kde_plasma6() {
                eprintln!("[Wayland] Not KDE Plasma 6, skipping KWin backend");
                return None;
            }

            eprintln!("[Wayland] Using integrated KWin scripting");
            let kwin_windows = kwin::get_windows();

            eprintln!("[Wayland] KWin returned {} windows", kwin_windows.len());

            if kwin_windows.is_empty() {
                None
            } else {
                Some(kwin_windows.into_iter().map(|w| WindowInfo {
                    id: w.id,
                    title: w.title,
                    process_name: w.class_name,
                }).collect())
            }
        }

        /// Activate window using KWin scripting API
        pub fn activate_window_by_uuid(uuid: &str) -> bool {
            eprintln!("[Wayland] Attempting to activate window via KWin: {}", uuid);
            kwin::activate_window(uuid)
        }
    }

    pub fn get_windows() -> Vec<WindowInfo> {
        let display_server = detect_display_server();
        eprintln!("[Platform] Detected display server: {:?}", display_server);
        match display_server {
            DisplayServer::X11 => {
                let windows = x11_backend::get_windows();
                eprintln!("[Platform] X11 returned {} windows", windows.len());
                windows
            },
            DisplayServer::Wayland => {
                // Try Wayland native backends first
                if let Some(windows) = wayland_backend::get_windows() {
                    eprintln!("[Platform] Wayland native returned {} windows", windows.len());
                    windows
                } else {
                    eprintln!("[Platform] Wayland native failed, trying XWayland fallback");
                    let windows = x11_backend::get_windows();
                    eprintln!("[Platform] XWayland returned {} windows", windows.len());
                    windows
                }
            },
        }
    }

    pub fn switch_window(window_id: String) {
        eprintln!("[Platform] switch_window called with id: {}", window_id);
        match detect_display_server() {
            DisplayServer::X11 => {
                eprintln!("[Platform] Using X11 backend");
                x11_backend::switch_window(window_id);
            },
            DisplayServer::Wayland => {
                eprintln!("[Platform] Using Wayland backend");
                // For Wayland, window_id IS the UUID string
                eprintln!("[Platform] Calling activate_window_by_uuid with uuid: {}", window_id);
                wayland_backend::activate_window_by_uuid(&window_id);
            },
        }
    }
}

#[tauri::command]
fn get_windows() -> Vec<WindowInfo> {
    // Return cached windows immediately (non-blocking)
    WINDOW_CACHE.lock().unwrap().clone()
}

/// Start background thread to periodically update window cache
fn start_window_cache_updater(app_handle: tauri::AppHandle) {
    thread::spawn(move || {
        let mut last_windows: Vec<WindowInfo> = Vec::new();

        loop {
            // Update cache
            let windows = platform::get_windows();

            // Check if data actually changed (compare id and title)
            let changed = windows.len() != last_windows.len() ||
                windows.iter().zip(last_windows.iter()).any(|(a, b)| {
                    a.id != b.id || a.title != b.title || a.process_name != b.process_name
                });

            if changed {
                {
                    let mut cache = WINDOW_CACHE.lock().unwrap();
                    *cache = windows.clone();
                }
                last_windows = windows.clone();

                // Notify frontend if main window is visible
                if let Some(window) = app_handle.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = app_handle.emit("windows-updated", windows);
                    }
                }
            }

            // Update every 5000ms
            thread::sleep(Duration::from_millis(5000));
        }
    });
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
fn switch_window(window_id: String) {
    eprintln!("[Command] switch_window called with window_id: {}", window_id);
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
                    center_window_on_mouse_monitor(&window);
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
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None
        ))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            // Start background window cache updater
            start_window_cache_updater(app.handle().clone());

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
                            center_window_on_mouse_monitor(&window);
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
                            center_window_on_mouse_monitor(&window);
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

            // DevTools disabled - uncomment below to enable in debug mode
            // #[cfg(debug_assertions)]
            // {
            //     let window = app.get_webview_window("main").unwrap();
            //     window.open_devtools();
            // }
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
            id: "123".to_string(),
            title: "Test Window".to_string(),
            process_name: "test.exe".to_string(),
        };

        // Serialize
        let json = serde_json::to_string(&window).unwrap();
        assert!(json.contains("\"id\":\"123\""));
        assert!(json.contains("\"title\":\"Test Window\""));
        assert!(json.contains("\"process_name\":\"test.exe\""));

        // Deserialize
        let decoded: WindowInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.id, "123");
        assert_eq!(decoded.title, "Test Window");
        assert_eq!(decoded.process_name, "test.exe");
    }

    #[test]
    fn test_window_info_clone() {
        let window = WindowInfo {
            id: "1".to_string(),
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

    #[test]
    #[cfg(target_os = "windows")]
    fn test_win_get_windows(){
        let windows = platform::get_windows();

        assert!(windows.len() > 1)
    }
}