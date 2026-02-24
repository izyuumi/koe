// Text insertion into the frontmost app
// Strategy: clipboard paste via Cmd+V, transcript stays on clipboard for re-paste

use std::process::Command;
use std::time::Duration;

/// Timeout for osascript calls to prevent indefinite blocking.
const APPLESCRIPT_TIMEOUT: Duration = Duration::from_secs(5);

/// Insert text at the current cursor position in the frontmost app.
/// Uses the clipboard + Cmd+V approach for maximum compatibility.
/// The transcript remains on the clipboard so the user can paste it again.
pub fn insert_text(text: &str) {
    // Set clipboard to our text; abort if write fails
    if !set_clipboard(text) {
        return;
    }

    // Simulate Cmd+V
    paste_via_applescript();

    // Leave the transcript on the clipboard so the user can paste it again.
    // Previously we restored the old clipboard content, but auto-copy is more
    // useful: the last dictation result stays available for ⌘V.
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
                return;
            }
        }
    }
}
