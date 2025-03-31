use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::{fs, thread};

use anyhow::{bail, Context, Result};
use notify::{EventKind, RecursiveMode, Watcher};

use crate::config::Config;
use crate::debug;
use crate::parse::ContentType;
use crate::tree::Tree;

pub struct FileWatcher {
    path: PathBuf,

    cfg: Rc<Config>,
    content_type: ContentType,

    max_data_size: usize,

    data: Arc<Mutex<Data>>,
}

impl FileWatcher {
    pub fn new(
        path: PathBuf,
        cfg: Rc<Config>,
        content_type: ContentType,
        max_data_size: usize,
    ) -> Self {
        let data = Arc::new(Mutex::new(Data {
            data: RefCell::new(None),
            err: RefCell::new(None),
        }));

        Self {
            path,
            cfg,
            content_type,
            max_data_size,
            data,
        }
    }

    pub fn start(&self) {
        let data_clone = self.data.clone();
        let path = self.path.clone();
        thread::spawn(move || {
            if let Err(e) = watch_file(path, data_clone) {
                debug!("FileWatcher: watch file error: {:#}", e);
            }
        });
    }

    pub fn get_err(&self) -> Option<String> {
        let data_lock = self.data.lock().unwrap();
        let err = data_lock.get_err();
        drop(data_lock);
        err
    }

    pub fn parse_tree(&self) -> Result<Option<Tree>> {
        let data_lock = self.data.lock().unwrap();
        let new_data = data_lock.get_data();

        let new_data = match new_data {
            Some(new_data) => {
                debug!("FileWatcher: file updated, size {}", new_data.len());
                new_data
            }
            None => return Ok(None),
        };

        if new_data.len() > self.max_data_size {
            data_lock.set_err("file size exceeds the limit");
            return Ok(None);
        }

        let tree = match Tree::parse(self.cfg.clone(), &new_data, self.content_type) {
            Ok(tree) => tree,
            Err(e) => {
                let msg = format!("{:#}", e);
                data_lock.set_err(msg);
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

    fn set_err(&self, err: impl ToString) {
        self.err.replace(Some(err.to_string()));
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
