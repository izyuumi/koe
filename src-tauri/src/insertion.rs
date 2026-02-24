// Text insertion into the frontmost app
// Strategy: clipboard paste via Cmd+V, transcript stays on clipboard for re-paste

use std::process::Command;

/// Insert text at the current cursor position in the frontmost app.
/// Uses the clipboard + Cmd+V approach for maximum compatibility.
/// The transcript remains on the clipboard so the user can paste it again.
pub fn insert_text(text: &str) {
    // Set clipboard to our text
    set_clipboard(text);

    // Simulate Cmd+V
    paste_via_applescript();

    // Leave the transcript on the clipboard so the user can paste it again.
    // Previously we restored the old clipboard content, but auto-copy is more
    // useful: the last dictation result stays available for ⌘V.
}

fn set_clipboard(text: &str) {
    match Command::new("pbcopy")
        .stdin(std::process::Stdio::piped())
        .spawn()
    {
        Ok(mut child) => {
            if let Some(mut stdin) = child.stdin.take() {
                use std::io::Write;
                let _ = stdin.write_all(text.as_bytes());
            }
            let _ = child.wait();
        }
        Err(e) => {
            eprintln!("Failed to run pbcopy: {}", e);
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
