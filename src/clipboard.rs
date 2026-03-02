use std::env;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::process::{Command, Stdio};

use anyhow::{bail, Context, Result};

/// The terminal multiplexer environment we are running inside, if any.
enum Muxer {
    /// tmux — requires DCS passthrough wrapping with doubled ESC bytes.
    Tmux,
    /// GNU Screen — requires DCS passthrough wrapping.
    Screen,
    /// Zellij — supports OSC 52 natively, no wrapping needed.
    Zellij,
    /// No known multiplexer detected.
    None,
}

fn detect_muxer() -> Muxer {
    // Order matters: it is possible to nest muxers (e.g. tmux inside Zellij).
    // Check the innermost (most specific) first.
    if env::var("ZELLIJ").is_ok() {
        Muxer::Zellij
    } else if env::var("TMUX").is_ok() {
        Muxer::Tmux
    } else if env::var("STY").is_ok() {
        Muxer::Screen
    } else {
        Muxer::None
    }
}

/// Build the OSC 52 clipboard write sequence, wrapped appropriately for the
/// current terminal multiplexer.
///
/// The raw OSC 52 sequence is:
///   ESC ] 52 ; c ; <base64> BEL
///
/// Multiplexer wrapping:
///   tmux  — `ESC P tmux; ESC <osc52_with_doubled_escs> ESC \`
///   screen — `ESC P <osc52> ESC \`
///   zellij / bare terminal — no wrapping needed.
fn build_osc52_sequence(text: &str) -> String {
    use base64::Engine;
    let encoded = base64::engine::general_purpose::STANDARD.encode(text.as_bytes());

    let osc52 = format!("\x1b]52;c;{encoded}\x07");

    match detect_muxer() {
        Muxer::Tmux => {
            // Inside tmux the ESC bytes in the inner sequence must be doubled
            // and the whole thing wrapped in a DCS passthrough.
            let inner = osc52.replace('\x1b', "\x1b\x1b");
            format!("\x1bPtmux;{inner}\x1b\\")
        }
        Muxer::Screen => {
            // GNU Screen DCS passthrough.
            format!("\x1bP{osc52}\x1b\\")
        }
        Muxer::Zellij | Muxer::None => osc52,
    }
}

/// Write to the clipboard using the OSC 52 escape sequence.
///
/// The sequence is written directly to the controlling TTY (`/dev/tty`) so it
/// reaches the outer terminal emulator even when stdout is owned by a TUI
/// framework like ratatui / crossterm.
fn write_osc52(text: &str) -> Result<()> {
    let seq = build_osc52_sequence(text);

    let mut tty = OpenOptions::new()
        .write(true)
        .open("/dev/tty")
        .context("open /dev/tty for OSC 52 clipboard write")?;

    tty.write_all(seq.as_bytes())
        .context("write OSC 52 sequence to /dev/tty")?;
    tty.flush().context("flush /dev/tty after OSC 52 write")?;

    Ok(())
}

/// Return `true` when we should prefer OSC 52 over a system clipboard command.
///
/// This is the case when we are inside any known terminal multiplexer, or when
/// no suitable clipboard command can be found on the system.
fn should_use_osc52() -> bool {
    !matches!(detect_muxer(), Muxer::None)
}

fn get_cmd() -> Result<Command> {
    let cmd = match env::consts::OS {
        "macos" => Command::new("pbcopy"),
        "linux" => {
            if env::var("WAYLAND_DISPLAY").is_ok() {
                Command::new("wl-copy")
            } else {
                let mut cmd = Command::new("xclip");
                cmd.args(["-selection", "clipboard"]);
                cmd
            }
        }
        "windows" => Command::new("clip"),
        _ => bail!(
            "os {} does not support clipboard, you can create issue if you have requirement",
            env::consts::OS
        ),
    };
    Ok(cmd)
}

fn write_clipboard_cmd(text: &str) -> Result<()> {
    let mut cmd = get_cmd()?;
    cmd.stdin(Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(child) => child,
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            // No system clipboard tool — fall back to OSC 52 as a last resort.
            return write_osc52(text);
        }
        Err(err) => return Err(err).context("launch clipboard program failed"),
    };

    let stdin = child.stdin.as_mut().unwrap();
    if let Err(err) = stdin.write_all(text.as_bytes()) {
        return Err(err).context("write text to clipboard program");
    }
    drop(child.stdin.take());

    let status = child.wait().context("wait clipboard program done")?;
    if !status.success() {
        let code = status
            .code()
            .map_or("<unknown>".to_string(), |code| code.to_string());
        bail!("clipboard program exited with bad status {code}",);
    }

    Ok(())
}

pub fn write_clipboard(text: &str) -> Result<()> {
    if should_use_osc52() {
        return write_osc52(text);
    }
    write_clipboard_cmd(text)
}
