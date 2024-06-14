use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use anyhow::{bail, Context, Result};

use crate::config::Config;

pub struct Edit {
    path: String,
    data: String,
    cmd: Command,
}

impl Edit {
    pub fn new(cfg: &Config, identify: String, data: String, extension: &'static str) -> Self {
        let mut cmd = Command::new(&cfg.editor.program);
        cmd.stdin(Stdio::inherit());
        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::inherit());

        let name = identify.replace('/', "_");
        let path = PathBuf::from(&cfg.editor.dir).join(format!("otree_{name}.{extension}"));
        let path = format!("{}", path.display());

        for arg in cfg.editor.args.iter() {
            if !arg.contains("{file}") {
                cmd.arg(arg);
                continue;
            }

            let arg = arg.replace("{file}", &path);
            cmd.arg(arg);
        }

        Self { path, data, cmd }
    }

    pub fn run(mut self) {
        if let Err(err) = self._run() {
            eprintln!("Edit error: {err:#}");
            eprintln!();
            eprintln!("Press any key to continue...");
            io::stdout().flush().unwrap();

            // Wait for a single character input
            let mut buffer = [0; 1];
            io::stdin().read_exact(&mut buffer).unwrap();
        }
    }

    fn _run(&mut self) -> Result<()> {
        self.write_file().context("write edit file")?;

        let result = self.edit_file().context("edit file");
        if result.is_err() {
            let _ = self.delete_file();
            return result;
        }

        self.delete_file().context("delete edit file")?;
        Ok(())
    }

    fn write_file(&self) -> Result<()> {
        let path = PathBuf::from(&self.path);
        if let Some(dir) = path.parent() {
            match fs::metadata(dir) {
                Ok(meta) => {
                    if !meta.is_dir() {
                        bail!("'{}' is not a directory", dir.display());
                    }
                }
                Err(err) if err.kind() == io::ErrorKind::NotFound => {
                    fs::create_dir_all(dir)
                        .with_context(|| format!("create directory '{}'", dir.display()))?;
                }
                Err(err) => {
                    return Err(err)
                        .with_context(|| format!("get metadata for '{}'", dir.display()))
                }
            }
        }

        fs::write(&path, &self.data)
            .with_context(|| format!("write data to file '{}'", path.display()))?;

        Ok(())
    }

    fn edit_file(&mut self) -> Result<()> {
        let status = self.cmd.status().context("execute editor command")?;
        if !status.success() {
            bail!("editor command exited with bad code");
        }
        Ok(())
    }

    fn delete_file(&self) -> Result<()> {
        fs::remove_file(&self.path).with_context(|| format!("delete file '{}'", self.path))
    }
}
