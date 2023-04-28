use crate::prelude::*;
use anyhow::Result;
use curl::{
    easy::{Easy2, Handler, WriteError},
    multi::Multi,
};
use std::{
    collections::HashMap,
    fs::{DirBuilder, File},
    io::Write,
    path::{Path, PathBuf},
};

const DEFAULT_PARALLEL: usize = 3;

struct Collector(File);

impl Handler for Collector {
    fn write(&mut self, data: &[u8]) -> std::result::Result<usize, WriteError> {
        if let Err(e) = self.0.write(data) {
            error!("{}", e);
        }
        Ok(data.len())
    }
}

#[derive(Debug, PartialEq)]
pub enum OnDuplicated {
    Rename,
    Replace,
    Skip,
}

#[derive(Debug)]
pub struct Downloader {
    handle: Multi,
    on_duplicated: OnDuplicated,
}

impl Downloader {
    pub fn new() -> Self {
        let mut handle = Multi::new();
        handle
            .set_max_total_connections(DEFAULT_PARALLEL)
            .expect("Fail to set max parallel connections");

        Self {
            handle,
            on_duplicated: OnDuplicated::Skip,
        }
    }

    pub fn handle(mut self, handle: Multi) -> Self {
        self.handle = handle;
        self
    }

    pub fn on_duplicated(mut self, action: OnDuplicated) -> Self {
        self.on_duplicated = action;
        self
    }

    pub fn parallel(mut self, n: usize) -> Self {
        self.handle
            .set_max_total_connections(n)
            .expect("Fail to set max parallel connections");
        self
    }

    pub fn download_sync<S, D>(&mut self, src: &[S], dst: D) -> Result<Vec<PathBuf>>
    where
        S: AsRef<str> + Clone + PartialEq,
        D: AsRef<Path>,
    {
        let mut src = src.to_vec();
        src.dedup();

        let dst = dst.as_ref();
        if !dst.exists() {
            info!("Creating {dst:?}");
            DirBuilder::new().recursive(true).create(dst)?;
        }

        let urls_names_paths = src.iter().filter_map(|url| {
            let url = url.as_ref();

            // TODO use `url` crate instead
            if let Some(name) = Path::new(url).file_name() {
                let name = name.to_string_lossy();
                let path = dst.join(name.as_ref());
                Some((url, name, path))
            } else {
                None
            }
        });

        let multi = &mut self.handle;
        let mut handles = HashMap::new();

        for (i, (url, mut name, mut path)) in urls_names_paths.enumerate() {
            let fd = if path.exists() {
                match self.on_duplicated {
                    OnDuplicated::Rename => {
                        let name_as_path = Path::new(name.as_ref());
                        let name_no_ext = name_as_path.with_extension("");
                        let name_no_ext = name_no_ext.to_str().unwrap();

                        let ext = name_as_path
                            .extension()
                            .unwrap_or_default()
                            .to_string_lossy();

                        let ext = if ext.len() > 0 {
                            format!(".{ext}")
                        } else {
                            String::new()
                        };

                        let mut suffix = 2usize;
                        let mut rename = format!("{name_no_ext}{suffix}{ext}");
                        while dst.join(&rename).exists() {
                            suffix += 1;
                            rename = format!("{name_no_ext}{suffix}{ext}");
                        }

                        name = rename.into();
                        path = path.with_file_name(name.as_ref());

                        warn!("{url:?} will be saved as {name:?}");
                        File::create(&path)?
                    }
                    OnDuplicated::Replace => {
                        warn!("{url:?} will rewrite an existing item");
                        File::options().write(true).open(&path)?
                    }
                    OnDuplicated::Skip => {
                        warn!("{url:?} will be skipped as item {name:?} already exists");
                        continue;
                    }
                }
            } else {
                File::create(&path)?
            };

            let mut dl = Easy2::new(Collector(fd));
            dl.url(url)?;

            // for collecting results
            let mut handle = multi.add2(dl)?;
            handle.set_token(i)?;
            handles.insert(i, (path, handle));
        }

        let mut alive = true;
        let mut downloaded = Vec::with_capacity(handles.len());

        while alive {
            // keep going to consume all messages
            if multi.perform()? == 0 {
                alive = false;
            }

            multi.messages(|msg| {
                let i = msg.token().unwrap();
                let (path, handle) = handles.get_mut(&i).unwrap();

                if let Some(res) = msg.result_for2(handle) {
                    match res {
                        Err(e) => {
                            error!("{e}");
                        }
                        Ok(()) => {
                            downloaded.push(path.clone());
                            let size = handle.get_ref().0.metadata().unwrap().len();
                            info!("Finish downloading {path:?} (size: {size})");
                        }
                    }
                }
            });
        }

        Ok(downloaded)
    }
}

pub fn download_sync<S, D>(src: &[S], dst: D) -> Result<Vec<PathBuf>>
where
    S: AsRef<str> + Clone + PartialEq,
    D: AsRef<Path>,
{
    let mut downloader = Downloader::new();
    downloader.download_sync(src, dst)
}
