mod speech;
mod insertion;

use tauri::{
    AppHandle, Emitter, Manager,
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use std::sync::atomic::{AtomicBool, Ordering};

static IS_LISTENING: AtomicBool = AtomicBool::new(false);

#[tauri::command]
fn start_dictation(app: AppHandle) -> Result<(), String> {
    if IS_LISTENING.load(Ordering::SeqCst) {
        return Ok(());
    }
    IS_LISTENING.store(true, Ordering::SeqCst);
    let _ = app.emit("listening-state", serde_json::json!({"listening": true}));

    // Show HUD
    if let Some(w) = app.get_webview_window("hud") {
        let _ = w.show();
        let _ = w.set_focus();
    }

    speech::start_recognition(app.clone());
    Ok(())
}

#[tauri::command]
fn stop_dictation(app: AppHandle) -> Result<String, String> {
    if !IS_LISTENING.load(Ordering::SeqCst) {
        return Ok(String::new());
    }
    IS_LISTENING.store(false, Ordering::SeqCst);
    let _ = app.emit("listening-state", serde_json::json!({"listening": false}));

    let text = speech::stop_recognition();

    // Hide HUD
    if let Some(w) = app.get_webview_window("hud") {
        let _ = w.hide();
    }

    // Insert text at cursor
    if !text.is_empty() {
        insertion::insert_text(&text);
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            start_dictation,
            stop_dictation,
            toggle_dictation,
        ])
        .setup(|app| {
            // Build tray menu
            let quit = MenuItem::with_id(app, "quit", "Quit Koe", true, None::<&str>)?;
            let toggle = MenuItem::with_id(app, "toggle", "Toggle Dictation (⌥Space)", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&toggle, &quit])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "quit" => app.exit(0),
                        "toggle" => {
                            let _ = toggle_dictation(app.clone());
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            // Register global shortcut: Option+Space
            let shortcut = Shortcut::new(Some(Modifiers::ALT), Code::Space);
            app.global_shortcut().on_shortcut(shortcut, move |app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    let _ = toggle_dictation(app.clone());
                }
            })?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Koe");
}
