use lazy_static::lazy_static;
pub use log::{error, info, warn};
use regex::Regex;
use std::path::Path;

const NAME_EXT_PAT: &str = r"(?P<name>[^/\\]+?)\.(?P<ext>tar\.(?:bz2|gz|xz)|tar|bz2|gz|xz)$";

lazy_static! {
    static ref NAME_EXT_RE: Regex = Regex::new(NAME_EXT_PAT).unwrap();
}

// filter out invalid paths and zip paths with file names
pub(crate) fn zip_file_name<'a, P>(paths: &'a [P]) -> Vec<(&'a Path, &'a str)>
where
    P: AsRef<Path>,
{
    paths
        .iter()
        .filter_map(|p| {
            let path = p.as_ref();
            if path.is_file() {
                if let Some(file_name) = path.file_name() {
                    if let Some(name) = file_name.to_str() {
                        return Some((path, name));
                    }
                }
            }
            None
        })
        .collect()
}

// `file_name` should always be valid, e.g., from the return of
// `path.file_name()`.
// Return Some(_) only if extension is supported.
pub(crate) fn name_ext<'a, T>(file_name: &'a T) -> Option<(&'a str, &'a str)>
where
    T: AsRef<str> + ?Sized,
{
    NAME_EXT_RE.captures(file_name.as_ref()).map(|caps| {
        (
            caps.name("name").unwrap().as_str(),
            caps.name("ext").unwrap().as_str(),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_extension() {
        assert_eq!(name_ext("/footar"), None);
        assert_eq!(name_ext("/.tar"), None);
        assert_eq!(name_ext("/.tartar"), None);
        assert_eq!(name_ext("/foo.tartar"), None);
        assert_eq!(name_ext("/foo.tar.tar"), Some(("foo.tar", "tar")));
        assert_eq!(name_ext("/foo.tar.tar.bz2"), Some(("foo.tar", "tar.bz2")));
        assert_eq!(name_ext("/foo.tar.tar.gz"), Some(("foo.tar", "tar.gz")));
        assert_eq!(name_ext("/foo.tar.tar.xz"), Some(("foo.tar", "tar.xz")));
        assert_eq!(name_ext("/foo.bar.bz2"), Some(("foo.bar", "bz2")));
        assert_eq!(name_ext("/foo.bar.gz"), Some(("foo.bar", "gz")));
        assert_eq!(name_ext("/foo.bar.xz"), Some(("foo.bar", "xz")));
    }
}
