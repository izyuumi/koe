// Speech recognition using Apple's Speech framework via a Swift helper process.

use std::process::Command;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};

static LAST_TRANSCRIPT: Mutex<Option<String>> = Mutex::new(None);
static CURRENT_PROCESS: Mutex<Option<u32>> = Mutex::new(None);

/// Start speech recognition by spawning a Swift helper process.
/// If a helper is already running, it will be killed first.
pub fn start_recognition(app: AppHandle, language: &str, on_device: bool) {
    // Kill any existing helper process to prevent duplicates
    if let Some(pid) = CURRENT_PROCESS.lock().unwrap().take() {
        eprintln!("[koe] Killing existing speech helper (pid {})", pid);
        unsafe {
            libc::kill(pid as i32, libc::SIGTERM);
        }
    }

    // Clear previous transcript
    *LAST_TRANSCRIPT.lock().unwrap() = None;

    let lang = language.to_string();

    std::thread::spawn(move || {
        let helper_path = get_helper_path();

        let mut cmd = Command::new(&helper_path);
        cmd.arg("--language").arg(&lang);
        if on_device {
            cmd.arg("--on-device");
        }
        cmd.stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        let child = cmd.spawn();

        match child {
            Ok(mut child) => {
                let pid = child.id();
                *CURRENT_PROCESS.lock().unwrap() = Some(pid);

                if let Some(stdout) = child.stdout.take() {
                    use std::io::BufRead;
                    let reader = std::io::BufReader::new(stdout);
                    for line in reader.lines() {
                        if let Ok(line) = line {
                            if line.starts_with("PARTIAL:") {
                                let text = line.trim_start_matches("PARTIAL:").trim();
                                let _ = app.emit("transcript-partial", serde_json::json!({"text": text}));
                            } else if line.starts_with("FINAL:") {
                                let text = line.trim_start_matches("FINAL:").trim();
                                *LAST_TRANSCRIPT.lock().unwrap() = Some(text.to_string());
                                let _ = app.emit("transcript-final", serde_json::json!({"text": text}));
                            } else if line.starts_with("LEVEL:") {
                                if let Ok(level) = line.trim_start_matches("LEVEL:").trim().parse::<f64>() {
                                    let _ = app.emit("mic-level", serde_json::json!({"level": level}));
                                }
                            } else if line.starts_with("ERROR:") {
                                let msg = line.trim_start_matches("ERROR:").trim();
                                let _ = app.emit("speech-error", serde_json::json!({"message": msg}));
                            }
                        }
                    }
                }

                let status = child.wait();
                *CURRENT_PROCESS.lock().unwrap() = None;

                // Watchdog: detect unexpected exit (crash)
                if let Ok(status) = status {
                    if !status.success() {
                        let code = status.code().unwrap_or(-1);
                        let _ = app.emit("speech-error", serde_json::json!({
                            "message": format!("Speech helper exited unexpectedly (code {})", code)
                        }));
                        let _ = app.emit("listening-state", serde_json::json!({"listening": false}));
                    }
                } else {
                    let _ = app.emit("speech-error", serde_json::json!({
                        "message": "Speech helper process lost"
                    }));
                    let _ = app.emit("listening-state", serde_json::json!({"listening": false}));
                }
            }
            Err(e) => {
                eprintln!("Failed to start speech helper: {}", e);
                let _ = app.emit("speech-error", serde_json::json!({
                    "message": format!("Failed to start speech helper: {}", e)
                }));
                let _ = app.emit("listening-state", serde_json::json!({"listening": false}));
            }
        }
    });
}

/// Stop recognition by killing the helper process
pub fn stop_recognition() -> String {
    if let Some(pid) = CURRENT_PROCESS.lock().unwrap().take() {
        unsafe {
            libc::kill(pid as i32, libc::SIGTERM);
        }
    }

    LAST_TRANSCRIPT
        .lock()
        .unwrap()
        .take()
        .unwrap_or_default()
}

fn get_helper_path() -> String {
    let exe = std::env::current_exe().unwrap();
    let dir = exe.parent().unwrap();

    let helper = dir.join("koe-speech-helper");
    if helper.exists() {
        return helper.to_string_lossy().to_string();
    }

    let resources = dir.parent().unwrap().join("Resources").join("koe-speech-helper");
    if resources.exists() {
        return resources.to_string_lossy().to_string();
    }

    if let Some(target_dir) = dir.parent() {
        if let Some(src_tauri) = target_dir.parent() {
            let dev_helper = src_tauri.join("koe-speech-helper");
            if dev_helper.exists() {
                return dev_helper.to_string_lossy().to_string();
            }
        }
    }

    "koe-speech-helper".to_string()
}
