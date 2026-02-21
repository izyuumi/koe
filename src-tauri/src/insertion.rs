// Text insertion into the frontmost app
// Strategy: clipboard paste with NSPasteboard changeCount tracking

use std::process::Command;

/// Time to wait before restoring the clipboard, giving the target app time to
/// read the pasted content.
const CLIPBOARD_RESTORE_DELAY_MS: u64 = 300;

/// Insert text at the current cursor position in the frontmost app.
/// Uses the clipboard + Cmd+V approach for maximum compatibility.
/// Tracks NSPasteboard changeCount to avoid clobbering external clipboard writes.
pub fn insert_text(text: &str) {
    // Save current clipboard content and changeCount
    let old_clipboard = get_clipboard();

    // Set clipboard to our text
    set_clipboard(text);

    // Record the changeCount after our write
    let our_change_count = get_pasteboard_change_count();

    // Simulate Cmd+V
    paste_via_applescript();

    // Restore clipboard after a delay, but only if no external app touched it
    let old = old_clipboard;
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(CLIPBOARD_RESTORE_DELAY_MS));

        // Check if changeCount is still what we set it to.
        // If another app wrote to the clipboard, changeCount will have incremented.
        let current_count = get_pasteboard_change_count();
        if current_count == our_change_count {
            if let Some(old_text) = old {
                set_clipboard(&old_text);
            }
        }
        // Otherwise, another app changed the clipboard — don't restore.
    });
}

/// Get the NSPasteboard changeCount via AppleScript (simple cross-process approach)
fn get_pasteboard_change_count() -> i64 {
    // Use a small Swift snippet via osascript to get changeCount
    // NSPasteboard.general.changeCount is the most reliable way
    let output = Command::new("osascript")
        .arg("-e")
        .arg(r#"use framework "AppKit"
return (current application's NSPasteboard's generalPasteboard()'s changeCount()) as integer"#)
        .output();

    match output {
        Ok(o) => {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            s.parse::<i64>().unwrap_or(-1)
        }
        Err(_) => -1,
    }
}

fn get_clipboard() -> Option<String> {
    Command::new("pbpaste")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
}

fn set_clipboard(text: &str) {
    let child = Command::new("pbcopy")
        .stdin(std::process::Stdio::piped())
        .spawn();

    match child {
        Ok(mut child) => {
            if let Some(mut stdin) = child.stdin.take() {
                use std::io::Write;
                let _ = stdin.write_all(text.as_bytes());
            }
            let _ = child.wait();
        }
        Err(e) => {
            eprintln!("[koe] Failed to run pbcopy: {e}");
        }
    }
}

fn paste_via_applescript() {
    Command::new("osascript")
        .arg("-e")
        .arg(r#"tell application "System Events" to keystroke "v" using command down"#)
        .output()
        .ok();
}
