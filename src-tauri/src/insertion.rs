// Text insertion into the frontmost app
// Strategy: clipboard paste via Cmd+V, transcript stays on clipboard for re-paste

use std::process::Command;
use std::time::Duration;

/// Timeout for osascript calls to prevent indefinite blocking.
const APPLESCRIPT_TIMEOUT: Duration = Duration::from_secs(5);
/// Give the target app time to consume Cmd+V before we overwrite the pasteboard.
const CLIPBOARD_RESTORE_DELAY: Duration = Duration::from_millis(150);

/// Insert text at the current cursor position in the frontmost app.
/// Uses the clipboard + Cmd+V approach for maximum compatibility.
///
/// `text_to_insert` is what will be pasted into the target app. `text_for_clipboard`
/// is what should remain on the clipboard after insertion (e.g., the raw transcript
/// without any chaining/padding characters).
pub fn insert_text_with_clipboard(text_to_insert: &str, text_for_clipboard: &str) {
    // Set clipboard to the string we want to paste; abort if write fails.
    if !set_clipboard(text_to_insert) {
        return;
    }

    // Simulate Cmd+V
    if let Err(e) = paste_via_applescript() {
        eprintln!("paste_via_applescript failed: {e}");
        restore_clipboard_if_unchanged(text_to_insert, text_for_clipboard);
        return;
    }

    std::thread::sleep(CLIPBOARD_RESTORE_DELAY);

    // Restore the desired clipboard content (best-effort).
    restore_clipboard_if_unchanged(text_to_insert, text_for_clipboard);
}

/// Convenience wrapper: insert and leave the same text on the clipboard.
pub fn insert_text(text: &str) {
    insert_text_with_clipboard(text, text);
}

/// Write text to the system clipboard via pbcopy. Returns true on success.
fn set_clipboard(text: &str) -> bool {
    match Command::new("pbcopy")
        .stdin(std::process::Stdio::piped())
        .spawn()
    {
        Ok(mut child) => {
            match child.stdin.take() {
                None => {
                    // stdin pipe unexpectedly unavailable — treat as failure.
                    eprintln!("Failed to open pbcopy stdin pipe");
                    let _ = child.wait();
                    return false;
                }
                Some(mut stdin) => {
                    use std::io::Write;
                    if let Err(e) = stdin.write_all(text.as_bytes()) {
                        eprintln!("Failed to write to pbcopy stdin: {}", e);
                        let _ = child.wait();
                        return false;
                    }
                    // Drop stdin to close the pipe before waiting so pbcopy
                    // doesn't block waiting for more input.
                }
            }
            match child.wait() {
                Ok(status) if status.success() => true,
                Ok(status) => {
                    eprintln!("pbcopy exited with non-zero status: {status}");
                    false
                }
                Err(e) => {
                    eprintln!("Failed to wait for pbcopy: {e}");
                    false
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to run pbcopy: {}", e);
            false
        }
    }
}

/// Read text from the system clipboard via pbpaste.
fn get_clipboard() -> Option<String> {
    match Command::new("pbpaste").output() {
        Ok(output) => {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                eprintln!("pbpaste exited with non-zero status: {}", output.status);
                None
            }
        }
        Err(e) => {
            eprintln!("Failed to run pbpaste: {e}");
            None
        }
    }
}

/// Only restore the clipboard if it still holds the injected paste text.
fn restore_clipboard_if_unchanged(text_to_insert: &str, text_for_clipboard: &str) {
    match get_clipboard() {
        Some(current) if current == text_to_insert => {
            let _ = set_clipboard(text_for_clipboard);
        }
        Some(_) => {}
        None => {
            eprintln!("Failed to read clipboard; skipping restore");
        }
    }
}

fn paste_via_applescript() -> Result<(), String> {
    let mut child = match Command::new("osascript")
        .arg("-e")
        .arg(r#"tell application "System Events" to keystroke "v" using command down"#)
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            return Err(format!("Failed to spawn osascript: {e}"));
        }
    };

    // Poll with timeout to avoid blocking indefinitely if System Events hangs
    let start = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if status.success() {
                    return Ok(());
                } else {
                    return Err(format!(
                        "osascript exited with non-zero status: {status}"
                    ));
                }
            }
            Ok(None) => {
                if start.elapsed() >= APPLESCRIPT_TIMEOUT {
                    eprintln!("osascript timed out after {:?}, killing", APPLESCRIPT_TIMEOUT);
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!(
                        "osascript timed out after {:?}",
                        APPLESCRIPT_TIMEOUT
                    ));
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("Error waiting for osascript: {e}"));
            }
        }
    }
}
