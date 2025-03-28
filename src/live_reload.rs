#![allow(dead_code)]

use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use anyhow::{bail, Context, Result};
use notify::{EventKind, RecursiveMode, Watcher};

use crate::config::Config;
use crate::debug;
use crate::parse::ContentType;
use crate::tree::Tree;

pub struct FileWatcher {
    cfg: Rc<Config>,
    content_type: ContentType,

    max_data_size: usize,

    data: Arc<Mutex<Data>>,
}

impl FileWatcher {
    pub fn parse_tree(&self) -> Result<Option<Tree>> {
        let data_lock = self.data.lock().unwrap();
        let new_data = data_lock.get_data();
        drop(data_lock);

        let new_data = match new_data {
            Some(new_data) => {
                debug!("FileWatcher: file updated, size {}", new_data.len());
                new_data
            }
            None => return Ok(None),
        };

        if new_data.len() > self.max_data_size {
            debug!("FileWatcher: file size exceeds the limit, skip");
            return Ok(None);
        }

        let tree = match Tree::parse(self.cfg.clone(), &new_data, self.content_type) {
            Ok(tree) => tree,
            Err(e) => {
                debug!("FileWatcher: failed to parse updated data: {:#}, skip", e);
                return Ok(None);
            }
        };
        Ok(Some(tree))
    }
}

struct Data {
    data: RefCell<Option<String>>,
    err: RefCell<Option<String>>,
}

impl Data {
    fn get_data(&self) -> Option<String> {
        self.data.borrow_mut().take()
    }

    fn get_err(&self) -> Option<String> {
        self.err.borrow_mut().take()
    }

    fn set_data(&self, data: String) {
        self.data.replace(Some(data));
    }

    fn set_err(&self, err: String) {
        self.err.replace(Some(err));
    }
}

fn watch_file(path: PathBuf, data: Arc<Mutex<Data>>) -> Result<()> {
    let (tx, rx) = mpsc::channel();

    let mut watcher = notify::recommended_watcher(tx)?;

    watcher.watch(&path, RecursiveMode::NonRecursive)?;

    debug!("Begin to watch file {:?} update events", path.display());
    for result in rx {
        let event = result?;
        match event.kind {
            EventKind::Create(_) | EventKind::Modify(_) => {
                let bytes = fs::read(&path)?;
                let s = String::from_utf8(bytes).context("read file as utf-8")?;
                let data_lock = data.lock().unwrap();
                data_lock.set_data(s);
                drop(data_lock);
            }
            EventKind::Remove(_) => bail!("the file {} was removed by user", path.display()),
            _ => {}
        }
    }

    bail!("watcher thread exited");
}
