use std::io::Write;
use std::process::{Command, Stdio};

pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    if which::which("wl-copy").is_ok() {
        return pipe_to("wl-copy", &[], text);
    }

    if cfg!(target_os = "macos") {
        return pipe_to("pbcopy", &[], text);
    }

    if which::which("xclip").is_ok() {
        return pipe_to("xclip", &["-selection", "clipboard"], text);
    }

    if which::which("xsel").is_ok() {
        return pipe_to("xsel", &["--clipboard", "--input"], text);
    }

    Err("No clipboard tool found. Install wl-copy (Wayland), xclip, or xsel (X11).".to_string())
}

fn pipe_to(cmd: &str, args: &[&str], text: &str) -> Result<(), String> {
    let mut child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to run {cmd}: {e}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        // Ignore broken pipe — clipboard tool may exit early
        let _ = stdin.write_all(text.as_bytes());
    }

    let _ = child.wait();
    Ok(())
}
