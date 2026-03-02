// Export transcripts to txt, md, and srt formats via a native save dialog.

use std::fs;

#[derive(serde::Deserialize, Debug)]
pub struct TranscriptSegment {
    pub text: String,
    pub start_ms: u64,
    pub end_ms: u64,
}

/// Export a transcript to the given format, opening a native save-file dialog.
/// Returns the saved path on success, or an empty string if the user cancelled.
#[tauri::command]
pub fn export_transcript(
    segments: Vec<TranscriptSegment>,
    format: String,
) -> Result<String, String> {
    if segments.is_empty() {
        return Err("No transcript to export".to_string());
    }

    let (content, ext, filter_name) = match format.as_str() {
        "txt" => (format_as_txt(&segments), "txt", "Text Files"),
        "md" => (format_as_md(&segments), "md", "Markdown Files"),
        "srt" => (format_as_srt(&segments), "srt", "SubRip Subtitles"),
        _ => return Err(format!("Unknown export format: {format}")),
    };

    let path = rfd::FileDialog::new()
        .set_file_name(&format!("transcript.{ext}"))
        .add_filter(filter_name, &[ext])
        .save_file();

    match path {
        Some(path) => {
            fs::write(&path, content)
                .map_err(|e| format!("Failed to write file: {e}"))?;
            Ok(path.to_string_lossy().to_string())
        }
        None => Ok(String::new()), // User cancelled
    }
}

fn format_as_txt(segments: &[TranscriptSegment]) -> String {
    segments
        .iter()
        .map(|s| s.text.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_as_md(segments: &[TranscriptSegment]) -> String {
    let mut out = String::from("# Transcript\n\n");
    for (i, seg) in segments.iter().enumerate() {
        let timestamp = ms_to_human(seg.start_ms);
        if segments.len() == 1 {
            // Single segment — no per-line timestamps needed
            out.push_str(&seg.text);
            out.push('\n');
        } else {
            out.push_str(&format!("**[{}]** {}\n\n", timestamp, seg.text));
        }
        let _ = i; // suppress unused warning
    }
    out
}

fn format_as_srt(segments: &[TranscriptSegment]) -> String {
    let mut out = String::new();
    for (i, seg) in segments.iter().enumerate() {
        out.push_str(&format!(
            "{}\n{} --> {}\n{}\n\n",
            i + 1,
            ms_to_srt_time(seg.start_ms),
            ms_to_srt_time(seg.end_ms),
            seg.text,
        ));
    }
    out
}

/// Format milliseconds as `HH:MM:SS,mmm` (SRT format).
fn ms_to_srt_time(ms: u64) -> String {
    let hours = ms / 3_600_000;
    let minutes = (ms % 3_600_000) / 60_000;
    let seconds = (ms % 60_000) / 1_000;
    let millis = ms % 1_000;
    format!("{hours:02}:{minutes:02}:{seconds:02},{millis:03}")
}

/// Format milliseconds as a human-readable `MM:SS` string.
fn ms_to_human(ms: u64) -> String {
    let minutes = ms / 60_000;
    let seconds = (ms % 60_000) / 1_000;
    format!("{minutes:02}:{seconds:02}")
}
