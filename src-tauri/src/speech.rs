// Speech recognition using Apple's Speech framework via objc2
// For MVP: record audio, then transcribe. Streaming comes later.

use std::process::Command;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};

static LAST_TRANSCRIPT: Mutex<Option<String>> = Mutex::new(None);
static CURRENT_PROCESS: Mutex<Option<u32>> = Mutex::new(None);

/// Start speech recognition by spawning a Swift helper process.
/// The Swift helper does the actual SFSpeechRecognizer work since calling
/// Speech framework from Rust FFI is extremely fragile.
pub fn start_recognition(app: AppHandle) {
    // Clear previous transcript
    *LAST_TRANSCRIPT.lock().unwrap() = None;

    // Spawn the Swift speech helper
    std::thread::spawn(move || {
        let helper_path = get_helper_path();

        let child = Command::new(&helper_path)
            .arg("--language")
            .arg("en-US")
            .arg("--on-device")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn();

        match child {
            Ok(mut child) => {
                let pid = child.id();
                *CURRENT_PROCESS.lock().unwrap() = Some(pid);

                // Read stdout for transcript updates
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
                            }
                        }
                    }
                }

                let _ = child.wait();
                *CURRENT_PROCESS.lock().unwrap() = None;
            }
            Err(e) => {
                eprintln!("Failed to start speech helper: {}", e);
            }
        }
    });
}

/// Stop recognition by killing the helper process
pub fn stop_recognition() -> String {
    if let Some(pid) = CURRENT_PROCESS.lock().unwrap().take() {
        // Send SIGTERM to helper
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
    // In dev: look for compiled helper next to the binary
    // In production: it's bundled in Resources
    let exe = std::env::current_exe().unwrap();
    let dir = exe.parent().unwrap();

    // Check next to binary first
    let helper = dir.join("koe-speech-helper");
    if helper.exists() {
        return helper.to_string_lossy().to_string();
    }

    // Check in Resources (bundled app)
    let resources = dir.parent().unwrap().join("Resources").join("koe-speech-helper");
    if resources.exists() {
        return resources.to_string_lossy().to_string();
    }

    // Fallback: try PATH
    "koe-speech-helper".to_string()
}
