use std::fs;
use std::io::Write;
use std::sync::OnceLock;

use anyhow::Result;

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        if let Err(_) = $crate::debug::write_logs(format!($($arg)*)) {
            // FIXME: How to handle err when writing logs failed?
            // We cannot write error to stderr since it is used for TUI.
        }
    };
}

static FILE: OnceLock<String> = OnceLock::new();

pub fn set_file(file: String) {
    FILE.set(file).unwrap();
}

pub fn write_logs(mut msg: String) -> Result<()> {
    msg.push('\n');
    let Some(file) = FILE.get() else {
        return Ok(());
    };

    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(file)?;

    file.write_all(msg.as_bytes())?;

    Ok(())
}
