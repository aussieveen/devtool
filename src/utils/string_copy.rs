use std::io::Write;
use std::process::{Command, Stdio};

pub fn copy_to_clipboard(text: String) -> Result<(), String> {
    let str = text.as_str();
    if which::which("wl-copy").is_ok() {
        return pipe_to("wl-copy", &[], str);
    }

    if cfg!(target_os = "macos") {
        return pipe_to("pbcopy", &[], str);
    }

    Ok(())
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
