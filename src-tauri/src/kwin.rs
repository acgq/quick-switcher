//! KWin Scripting API integration for KDE Plasma Wayland
//!
//! This module implements window management by dynamically loading
//! JavaScript scripts into KWin via DBus.

use std::io::Write;
use std::sync::mpsc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use dbus::{
    blocking::{Connection, SyncConnection},
    channel::MatchingReceiver,
    message::MatchRule,
};
use handlebars::{Handlebars, Context};
use serde::Deserialize;

/// Window information returned by KWin
#[derive(Debug, Clone, Deserialize)]
pub struct KWinWindow {
    pub id: String,
    pub title: String,
    #[serde(rename = "class_name")]
    pub class_name: String,
    pub pid: u32,
}

/// Check if we're running on KDE Plasma 6
pub fn is_kde_plasma6() -> bool {
    std::env::var("KDE_SESSION_VERSION") == Ok("6".to_string())
}

/// Get all windows via KWin scripting API
pub fn get_windows() -> Vec<KWinWindow> {
    if !is_kde_plasma6() {
        return Vec::new();
    }

    run_script(SCRIPT_GET_WINDOWS).unwrap_or_default()
}

/// Activate a window by its ID
pub fn activate_window(window_id: &str) -> bool {
    if !is_kde_plasma6() {
        return false;
    }

    let script = SCRIPT_ACTIVATE_WINDOW.replace("{{WINDOW_ID}}", window_id);
    run_script_void(&script).is_ok()
}

/// Globals for script template rendering
#[derive(serde::Serialize)]
struct ScriptGlobals {
    dbus_addr: String,
    marker: String,
    debug: bool,
}

/// Run a KWin script and return window list
fn run_script(script_template: &str) -> Result<Vec<KWinWindow>, Box<dyn std::error::Error>> {
    let unique_suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis();
    let marker = format!("quick-switcher-{unique_suffix}");

    // Establish the DBus listener connection first
    let self_conn = SyncConnection::new_session()?;
    let dbus_addr = self_conn.unique_name().to_string();

    // Prepare script with DBus address
    let globals = ScriptGlobals {
        dbus_addr: dbus_addr.clone(),
        marker: marker.clone(),
        debug: false,
    };

    let mut reg = Handlebars::new();
    reg.set_strict_mode(true);
    let render_context = Context::wraps(&globals)?;

    let script_content = reg.render_template_with_context(script_template, &render_context)?;

    // Set up result channel
    let (tx, rx) = mpsc::channel();

    let _receiver = self_conn.start_receive(
        MatchRule::new_method_call(),
        Box::new(move |message, _connection| -> bool {
            if let Some(member) = message.member() {
                if let Some(arg) = message.get1::<String>() {
                    match member.as_ref() {
                        "result" => {
                            let _ = tx.send(ScriptMessage::Result(arg));
                        }
                        "error" => {
                            let _ = tx.send(ScriptMessage::Error(arg));
                        }
                        _ => {}
                    }
                }
            }
            true
        }),
    );

    // Write script to temp file
    let mut script_file = tempfile::NamedTempFile::with_prefix("quick-switcher-")?;
    script_file.write_all(script_content.as_bytes())?;
    let script_path = script_file.into_temp_path();

    // Load and run script via DBus
    let kwin_conn = Connection::new_session()?;
    let kwin_proxy = kwin_conn.with_proxy(
        "org.kde.KWin",
        "/Scripting",
        Duration::from_millis(5000),
    );

    // Load script
    let script_id: i32;
    (script_id,) = kwin_proxy.method_call(
        "org.kde.kwin.Scripting",
        "loadScript",
        (script_path.to_str().unwrap(), &marker),
    )?;

    if script_id < 0 {
        return Err("Failed to load KWin script".into());
    }

    let script_proxy = kwin_conn.with_proxy(
        "org.kde.KWin",
        format!("/Scripting/Script{}", script_id),
        Duration::from_millis(5000),
    );

    // Run script
    let _: () = script_proxy.method_call("org.kde.kwin.Script", "run", ())?;
    let _: () = script_proxy.method_call("org.kde.kwin.Script", "stop", ())?;

    // Poll for results
    let start = Instant::now();
    let timeout = Duration::from_secs(5);
    let mut results = Vec::new();

    loop {
        self_conn.process(Duration::from_millis(100))?;
        match rx.try_recv() {
            Ok(ScriptMessage::Result(payload)) => {
                // Try to parse as KWinWindow JSON
                if let Ok(window) = parse_window_info(&payload) {
                    results.push(window);
                }
            }
            Ok(ScriptMessage::Error(msg)) => {
                eprintln!("[KWin] Script error: {}", msg);
            }
            Err(mpsc::TryRecvError::Empty) => {
                if start.elapsed() > timeout {
                    break;
                }
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                break;
            }
        }
    }

    // Unload script
    let _: Result<(), _> = kwin_proxy.method_call(
        "org.kde.kwin.Scripting",
        "unloadScript",
        (&marker,),
    );

    Ok(results)
}

/// Run a script that doesn't return data
fn run_script_void(script_content: &str) -> Result<(), Box<dyn std::error::Error>> {
    let unique_suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis();
    let script_name = format!("quick-switcher-{unique_suffix}");

    let mut script_file = tempfile::NamedTempFile::with_prefix("quick-switcher-")?;
    script_file.write_all(script_content.as_bytes())?;
    let script_path = script_file.into_temp_path();

    let kwin_conn = Connection::new_session()?;
    let kwin_proxy = kwin_conn.with_proxy(
        "org.kde.KWin",
        "/Scripting",
        Duration::from_millis(5000),
    );

    let script_id: i32;
    (script_id,) = kwin_proxy.method_call(
        "org.kde.kwin.Scripting",
        "loadScript",
        (script_path.to_str().unwrap(), &script_name),
    )?;

    if script_id < 0 {
        return Err("Failed to load KWin script".into());
    }

    let script_proxy = kwin_conn.with_proxy(
        "org.kde.KWin",
        format!("/Scripting/Script{}", script_id),
        Duration::from_millis(5000),
    );

    let _: () = script_proxy.method_call("org.kde.kwin.Script", "run", ())?;
    let _: () = script_proxy.method_call("org.kde.kwin.Script", "stop", ())?;

    let _: Result<(), _> = kwin_proxy.method_call(
        "org.kde.kwin.Scripting",
        "unloadScript",
        (&script_name,),
    );

    Ok(())
}

/// Parse window info from JSON payload
fn parse_window_info(payload: &str) -> Result<KWinWindow, Box<dyn std::error::Error>> {
    // KWin sends JSON.stringify output as a DBus string
    // Try parsing directly first; if that fails, try unescaping
    serde_json::from_str(payload)
        .or_else(|_| {
            let unescaped: String = serde_json::from_str(payload)?;
            serde_json::from_str(&unescaped)
        })
        .map_err(|e| e.into())
}

enum ScriptMessage {
    Result(String),
    Error(String),
}

// KWin JavaScript script templates

/// Script to get all windows - uses handlebars template
const SCRIPT_GET_WINDOWS: &str = r#"
function output_result(message) {
    callDBus("{{dbus_addr}}", "/", "", "result", message.toString());
}

var windows = workspace.windowList();
for (var i = 0; i < windows.length; i++) {
    var w = windows[i];
    if (w.caption && w.caption.length > 0) {
        output_result(JSON.stringify({
            id: w.internalId.toString(),
            title: w.caption,
            class_name: w.resourceClass,
            pid: w.pid
        }));
    }
}
"#;

/// Script to activate a window (no template variables needed besides WINDOW_ID)
const SCRIPT_ACTIVATE_WINDOW: &str = r#"
var windows = workspace.windowList();
for (var i = 0; i < windows.length; i++) {
    var w = windows[i];
    if (w.internalId.toString() === "{{WINDOW_ID}}") {
        workspace.activeWindow = w;
        break;
    }
}
"#;