pub mod download;
pub mod extract;
mod prelude;

pub use download::download_sync;
pub use extract::extract_sync;

pub use anyhow::Result;
use std::path::Path;

pub fn dlxt_sync<D>(src: &[&str], dst: D) -> Result<()>
where
    D: AsRef<Path>,
{
    let dst = dst.as_ref();
    let downloaded = download_sync(src, dst)?;
    let extracted = extract_sync(&downloaded, dst)?;

    for path in extracted.iter() {
        std::fs::remove_file(path)?;
    }

    Ok(())
}
