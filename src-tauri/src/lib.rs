mod speech;
mod insertion;

use tauri::{
    AppHandle, Emitter, Manager,
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    image::Image,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

static IS_LISTENING: AtomicBool = AtomicBool::new(false);
static LANGUAGE: Mutex<String> = Mutex::new(String::new());
static ON_DEVICE: AtomicBool = AtomicBool::new(true);
/// Tracks whether the fn/Globe key is currently held down.
static FN_KEY_ACTIVE: AtomicBool = AtomicBool::new(false);
/// Tracks whether the current fn/Globe press is still an isolated tap candidate.
static FN_KEY_PENDING_TOGGLE: AtomicBool = AtomicBool::new(false);

/// HUD window dimensions and positioning
const HUD_WIDTH: f64 = 320.0;
const HUD_TOP_OFFSET: f64 = 40.0;

fn get_language() -> String {
    let lang = LANGUAGE.lock().unwrap();
    if lang.is_empty() { "en-US".to_string() } else { lang.clone() }
}

/// Update tray icon based on listening state
fn update_tray_icon(app: &AppHandle, listening: bool) {
    if let Some(tray) = app.tray_by_id("main-tray") {
        let icon_bytes: &[u8] = if listening {
            include_bytes!("../icons/tray-listening.png")
        } else {
            include_bytes!("../icons/tray-idle.png")
        };
        if let Ok(img) = Image::from_bytes(icon_bytes) {
            let _ = tray.set_icon(Some(img));
        }
    }
}

#[tauri::command]
fn set_dictation_settings(language: String, on_device: bool) -> Result<(), String> {
    *LANGUAGE.lock().unwrap() = language;
    ON_DEVICE.store(on_device, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
fn start_dictation(app: AppHandle) -> Result<(), String> {
    if IS_LISTENING.load(Ordering::SeqCst) {
        return Ok(());
    }
    IS_LISTENING.store(true, Ordering::SeqCst);
    let _ = app.emit("listening-state", serde_json::json!({"listening": true}));
    update_tray_icon(&app, true);

    // Show HUD and position at top-center
    if let Some(w) = app.get_webview_window("hud") {
        // Position at top-center of primary monitor
        if let Ok(Some(monitor)) = w.primary_monitor() {
            let screen_size = monitor.size();
            let scale = monitor.scale_factor();
            let screen_w = screen_size.width as f64 / scale;
            let x = (screen_w - HUD_WIDTH) / 2.0;
            let _ = w.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(x, HUD_TOP_OFFSET)));
        }
        let _ = w.show();
        let _ = w.set_focus();
    }

    let lang = get_language();
    let on_device = ON_DEVICE.load(Ordering::SeqCst);
    speech::start_recognition(app.clone(), &lang, on_device);
    Ok(())
}

#[tauri::command]
fn stop_dictation(app: AppHandle) -> Result<String, String> {
    if !IS_LISTENING.load(Ordering::SeqCst) {
        return Ok(String::new());
    }
    IS_LISTENING.store(false, Ordering::SeqCst);
    let _ = app.emit("listening-state", serde_json::json!({"listening": false}));
    update_tray_icon(&app, false);

    let text = speech::stop_recognition();

    // Don't hide HUD here — the frontend handles the delayed hide
    // so the user can see the final transcript briefly.

    let text = text.trim().to_string();
    if !text.is_empty() {
        // Append a trailing space so consecutive dictations chain naturally
        let text_with_space = format!("{text} ");
        insertion::insert_text(&text_with_space);
        return Ok(text_with_space);
    }

    Ok(text)
}

#[tauri::command]
fn toggle_dictation(app: AppHandle) -> Result<(), String> {
    if IS_LISTENING.load(Ordering::SeqCst) {
        stop_dictation(app)?;
    } else {
        start_dictation(app)?;
    }
    Ok(())
}

#[tauri::command]
fn open_microphone_settings() -> Result<(), String> {
    open_system_settings("x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone")
}

#[tauri::command]
fn open_speech_settings() -> Result<(), String> {
    open_system_settings("x-apple.systempreferences:com.apple.preference.security?Privacy_SpeechRecognition")
}

#[tauri::command]
fn open_accessibility_settings() -> Result<(), String> {
    open_system_settings("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
}

fn open_system_settings(url: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let status = std::process::Command::new("open")
            .arg(url)
            .status()
            .map_err(|e| format!("failed to launch System Settings: {e}"))?;

        if status.success() {
            Ok(())
        } else {
            Err("System Settings returned a non-zero status".to_string())
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = url;
        Err("Opening System Settings is only supported on macOS".to_string())
    }
}

/// Register NSEvent monitors for the fn/Globe key (macOS 15+).
///
/// We keep both a global monitor (when Koe is inactive) and a local monitor
/// (when Koe is active and the HUD has focus). Dictation only toggles on an
/// isolated fn tap: fn-down arms the toggle, any other key activity disarms it,
/// and fn-up performs the toggle if the press remained isolated.
#[cfg(target_os = "macos")]
fn setup_fn_key_monitor(app: AppHandle) {
    use objc2_app_kit::{NSEvent, NSEventMask, NSEventModifierFlags, NSEventType};
    use std::ptr::NonNull;

    fn handle_fn_key_event(app: &AppHandle, event: &NSEvent) {
        let event_type = event.r#type();
        if event_type == NSEventType::FlagsChanged {
            let flags = event.modifierFlags() & NSEventModifierFlags::DeviceIndependentFlagsMask;
            let fn_down = flags.contains(NSEventModifierFlags::Function);
            let fn_only = flags == NSEventModifierFlags::Function;
            let was_down = FN_KEY_ACTIVE.swap(fn_down, Ordering::SeqCst);

            if fn_down && !was_down {
                FN_KEY_PENDING_TOGGLE.store(fn_only, Ordering::SeqCst);
                return;
            }

            if fn_down && !fn_only {
                FN_KEY_PENDING_TOGGLE.store(false, Ordering::SeqCst);
                return;
            }

            if !fn_down && was_down {
                if FN_KEY_PENDING_TOGGLE.swap(false, Ordering::SeqCst) {
                    if let Err(err) = toggle_dictation(app.clone()) {
                        eprintln!("fn/Globe toggle failed: {err}");
                    }
                }
            }

            return;
        }

        if FN_KEY_ACTIVE.load(Ordering::SeqCst)
            && (event_type == NSEventType::KeyDown || event_type == NSEventType::KeyUp)
        {
            FN_KEY_PENDING_TOGGLE.store(false, Ordering::SeqCst);
        }
    }

    let event_mask = NSEventMask::FlagsChanged | NSEventMask::KeyDown | NSEventMask::KeyUp;

    let global_app = app.clone();
    let global_block = block2::RcBlock::new(move |event: NonNull<NSEvent>| {
        // SAFETY: The pointer is valid for the duration of this callback;
        // NSEvent guarantees the object outlives the handler invocation.
        let event = unsafe { event.as_ref() };
        handle_fn_key_event(&global_app, event);
    });

    // Register the global monitor. The returned Retained<AnyObject> token must stay
    // alive for the handler to remain active; we intentionally leak it here so it
    // persists for the entire app lifetime.
    match NSEvent::addGlobalMonitorForEventsMatchingMask_handler(event_mask, &global_block) {
        Some(monitor) => std::mem::forget(monitor),
        None => eprintln!("fn/Globe monitor not registered (check Accessibility/Input Monitoring permissions)."),
    }

    let local_block = block2::RcBlock::new(move |event: NonNull<NSEvent>| {
        // SAFETY: The pointer is valid for the duration of this callback;
        // NSEvent guarantees the object outlives the handler invocation.
        let event_ref = unsafe { event.as_ref() };
        let event_type = event_ref.r#type();
        handle_fn_key_event(&app, event_ref);
        // Swallow FlagsChanged events that involve the fn/Globe key so that macOS
        // does not also dispatch its own Globe action (Emoji & Symbols picker or
        // system Dictation) alongside the Koe toggle.
        if event_type == NSEventType::FlagsChanged {
            let flags = event_ref.modifierFlags()
                & NSEventModifierFlags::DeviceIndependentFlagsMask;
            if flags.contains(NSEventModifierFlags::Function)
                || FN_KEY_ACTIVE.load(Ordering::SeqCst)
            {
                return std::ptr::null_mut();
            }
        }
        event.as_ptr()
    });

    // The local monitor keeps fn/Globe working while Koe is the active app and
    // also lets us suppress fn-based chords before AppKit dispatches them.
    match unsafe {
        NSEvent::addLocalMonitorForEventsMatchingMask_handler(event_mask, &local_block)
    } {
        Some(monitor) => std::mem::forget(monitor),
        None => eprintln!("fn/Globe local monitor not registered."),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            start_dictation,
            stop_dictation,
            toggle_dictation,
            set_dictation_settings,
            open_microphone_settings,
            open_speech_settings,
            open_accessibility_settings,
        ])
        .setup(|app| {
            // Build tray menu
            let quit = MenuItem::with_id(app, "quit", "Quit Koe", true, None::<&str>)?;
            let toggle = MenuItem::with_id(app, "toggle", "Toggle Dictation (fn/Globe · ⌥Space)", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&toggle, &quit])?;

            // Use idle tray icon initially
            let idle_icon = Image::from_bytes(include_bytes!("../icons/tray-idle.png"))
                .unwrap_or_else(|_| app.default_window_icon().unwrap().clone());

            TrayIconBuilder::with_id("main-tray")
                .icon(idle_icon)
                .menu(&menu)
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "quit" => app.exit(0),
                        "toggle" => {
                            if let Err(err) = toggle_dictation(app.clone()) {
                                eprintln!("tray toggle failed: {err}");
                            }
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            // Register global shortcut: Option+Space (kept as secondary shortcut)
            let shortcut = Shortcut::new(Some(Modifiers::ALT), Code::Space);
            app.global_shortcut().on_shortcut(shortcut, move |app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    if let Err(err) = toggle_dictation(app.clone()) {
                        eprintln!("⌥Space toggle failed: {err}");
                    }
                }
            })?;

            // Register fn/Globe key monitor via NSEvent (macOS 15+)
            #[cfg(target_os = "macos")]
            setup_fn_key_monitor(app.handle().clone());

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Koe");
}
