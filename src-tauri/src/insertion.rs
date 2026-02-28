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
    paste_via_applescript();

    std::thread::sleep(CLIPBOARD_RESTORE_DELAY);

    // Restore the desired clipboard content (best-effort).
    let _ = set_clipboard(text_for_clipboard);
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
            if let Some(mut stdin) = child.stdin.take() {
                use std::io::Write;
                if let Err(e) = stdin.write_all(text.as_bytes()) {
                    eprintln!("Failed to write to pbcopy stdin: {}", e);
                    let _ = child.wait();
                    return false;
                }
            }
            let _ = child.wait();
            true
        }
        Err(e) => {
            eprintln!("Failed to run pbcopy: {}", e);
            false
        }
    }
}

fn paste_via_applescript() {
    let mut child = match Command::new("osascript")
        .arg("-e")
        .arg(r#"tell application "System Events" to keystroke "v" using command down"#)
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to spawn osascript: {}", e);
            return;
        }
    };

    // Poll with timeout to avoid blocking indefinitely if System Events hangs
    let start = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => return,
            Ok(None) => {
                if start.elapsed() >= APPLESCRIPT_TIMEOUT {
                    eprintln!("osascript timed out after {:?}, killing", APPLESCRIPT_TIMEOUT);
                    let _ = child.kill();
                    let _ = child.wait();
                    return;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                eprintln!("Error waiting for osascript: {}", e);
                let _ = child.kill();
                let _ = child.wait();
                return;
            }
        }
    }
}
