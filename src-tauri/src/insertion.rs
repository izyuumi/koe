// Text insertion into the frontmost app
// Strategy: clipboard paste (most reliable across apps)

use std::process::Command;

/// Insert text at the current cursor position in the frontmost app.
/// Uses the clipboard + Cmd+V approach for maximum compatibility.
pub fn insert_text(text: &str) {
    // Save current clipboard
    let old_clipboard = get_clipboard();

    // Set clipboard to our text
    set_clipboard(text);

    // Simulate Cmd+V
    paste_via_applescript();

    // Restore clipboard after a short delay
    let old = old_clipboard;
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(200));
        if let Some(old_text) = old {
            set_clipboard(&old_text);
        }
    });
}

fn get_clipboard() -> Option<String> {
    Command::new("pbpaste")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.to_string())
}

fn set_clipboard(text: &str) {
    let mut child = Command::new("pbcopy")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .expect("failed to run pbcopy");

    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        let _ = stdin.write_all(text.as_bytes());
    }
    let _ = child.wait();
}

fn paste_via_applescript() {
    Command::new("osascript")
        .arg("-e")
        .arg(r#"tell application "System Events" to keystroke "v" using command down"#)
        .output()
        .ok();
}
