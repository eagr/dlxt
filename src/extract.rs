use crate::prelude::*;
use anyhow::Result;
use bzip2::bufread::BzDecoder;
use flate2::bufread::GzDecoder;
use log::info;
use std::{
    fs::{self, DirBuilder, File},
    io::{BufReader, Read, Write},
    path::{Path, PathBuf},
};
use tar::Archive;
use xz2::bufread::XzDecoder;

pub enum OnUnsupported {
    Skip,
    Copy,
}

pub struct Extractor {
    on_unsupported: OnUnsupported,
}

impl Extractor {
    pub fn new() -> Self {
        Self {
            on_unsupported: OnUnsupported::Skip,
        }
    }

    pub fn on_unsupported(mut self, action: OnUnsupported) -> Self {
        self.on_unsupported = action;
        self
    }

    pub fn extract_sync<S, D>(&self, src: &[S], dst: D) -> Result<Vec<PathBuf>>
    where
        S: AsRef<Path> + Clone + PartialEq,
        D: AsRef<Path>,
    {
        let mut src = src.to_vec();
        src.dedup();
        let paths_names = zip_file_name(&src);

        let dst = dst.as_ref();
        if !dst.exists() {
            info!("Creating {dst:?}");
            DirBuilder::new().recursive(true).create(dst)?;
        }

        let mut extracted = Vec::with_capacity(src.len());

        for (path, file_name) in paths_names {
            if let Some((name, ext)) = name_ext(file_name) {
                match ext {
                    "xz" | "gz" | "bz2" => {
                        decompress_sync(
                            ext,
                            &mut File::open(path)?,
                            &mut File::create(dst.join(name))?,
                        )?;
                    }
                    "tar.xz" | "tar.gz" | "tar.bz2" => {
                        let ext = ext.split('.').last().unwrap();
                        let ar_path = dst.join(name).with_extension(".tar");
                        decompress_sync(ext, &mut File::open(path)?, &mut File::create(&ar_path)?)?;
                        unpack_sync(&mut File::open(&ar_path)?, dst)?;
                        fs::remove_file(&ar_path)?;
                    }
                    "tar" => {
                        unpack_sync(&mut File::open(path)?, dst)?;
                    }
                    _ => unimplemented!(),
                }
                extracted.push(path.to_path_buf());
            } else {
                match self.on_unsupported {
                    OnUnsupported::Skip => {
                        info!("Skip {path:?} as its extension is unsupported");
                        continue;
                    }
                    OnUnsupported::Copy => {
                        let to = dst.join(file_name);
                        info!("Copying {path:?} to {to:?}");
                        fs::copy(path, to)?;
                    }
                }
            }
        }

        Ok(extracted)
    }
}

pub fn extract_sync<S, D>(src: &[S], dst: D) -> Result<Vec<PathBuf>>
where
    S: AsRef<Path> + Clone + PartialEq,
    D: AsRef<Path>,
{
    let extractor = Extractor::new();
    extractor.extract_sync(src, dst)
}

fn decompress_sync<S, D>(ext: &str, src: &mut S, dst: &mut D) -> Result<()>
where
    S: Read,
    D: Write,
{
    let reader = BufReader::new(src);
    let mut decoder: Box<dyn Read> = match ext {
        "xz" => Box::new(XzDecoder::new(reader)),
        "gz" => Box::new(GzDecoder::new(reader)),
        "bz2" => Box::new(BzDecoder::new(reader)),
        _ => unimplemented!(),
    };

    std::io::copy(&mut decoder, dst)?;
    Ok(())
}

fn unpack_sync<S, D>(src: &mut S, dst: D) -> Result<()>
where
    S: Read,
    D: AsRef<Path>,
{
    let mut tarball = Archive::new(src);
    let dst = dst.as_ref();

    info!("Unpacking tarball to {dst:?}");
    tarball.unpack(dst)?;
    Ok(())
}
