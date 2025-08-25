use std::env;
use std::io::{self, Write};
use std::process::{Command, Stdio};

use anyhow::{bail, Context, Result};

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

pub fn write_clipboard(text: &str) -> Result<()> {
    let mut cmd = get_cmd()?;
    cmd.stdin(Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(child) => child,
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            let program = cmd.get_program().to_string_lossy();
            bail!("cannot find clipboard program '{program}' in your system, please install it first to support clipboard")
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
