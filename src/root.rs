use crate::error::{Error, Result};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// A nabu marker beats a nearer `.jj/`: a jj workspace nested inside a nabu repo
/// resolves to the outer root until it gets its own `nabu/`.
pub fn find_root(start: &Path) -> Result<PathBuf> {
    let has_marker =
        |dir: &&Path| dir.join("nabu/config.toml").is_file() || dir.join("nabu/tablets").is_dir();
    start
        .ancestors()
        .find(has_marker)
        .or_else(|| start.ancestors().find(|dir| dir.join(".jj").is_dir()))
        .map(Path::to_path_buf)
        .ok_or_else(|| {
            Error::new(
                format!("no nabu root found from {}", start.display()),
                "run inside a jj repo or create nabu/config.toml",
            )
        })
}

/// Reads the environment directly because `dirs`-style crates answer
/// `~/Library/Application Support` on macOS, and the user wants `~/.config/nabu/`.
pub fn global_dir(xdg_config_home: Option<OsString>, home: Option<OsString>) -> Option<PathBuf> {
    // An empty value counts as unset, and a relative one is invalid (XDG Base Directory spec).
    let xdg = xdg_config_home
        .map(PathBuf::from)
        .filter(|path| path.is_absolute());
    match xdg {
        Some(dir) => Some(dir.join("nabu")),
        None => home
            .filter(|home| !home.is_empty())
            .map(|home| PathBuf::from(home).join(".config/nabu")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn mkdir(root: &Path, rel: &str) -> PathBuf {
        let dir = root.join(rel);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn marker_found_from_nested_cwd() {
        let tmp = tempfile::tempdir().unwrap();
        mkdir(tmp.path(), "repo/nabu");
        fs::write(tmp.path().join("repo/nabu/config.toml"), "").unwrap();
        let cwd = mkdir(tmp.path(), "repo/a/b");
        assert_eq!(find_root(&cwd).unwrap(), tmp.path().join("repo"));
    }

    #[test]
    fn tablets_folder_is_a_marker() {
        let tmp = tempfile::tempdir().unwrap();
        mkdir(tmp.path(), "repo/nabu/tablets");
        let cwd = mkdir(tmp.path(), "repo/a");
        assert_eq!(find_root(&cwd).unwrap(), tmp.path().join("repo"));
    }

    #[test]
    fn marker_beats_nearer_jj() {
        let tmp = tempfile::tempdir().unwrap();
        mkdir(tmp.path(), "repo/nabu/tablets");
        mkdir(tmp.path(), "repo/ws/.jj");
        let cwd = mkdir(tmp.path(), "repo/ws/src");
        assert_eq!(find_root(&cwd).unwrap(), tmp.path().join("repo"));
    }

    #[test]
    fn jj_is_the_fallback() {
        let tmp = tempfile::tempdir().unwrap();
        mkdir(tmp.path(), "repo/.jj");
        let cwd = mkdir(tmp.path(), "repo/src");
        assert_eq!(find_root(&cwd).unwrap(), tmp.path().join("repo"));
    }

    #[test]
    fn no_root_is_an_error_with_a_fix() {
        let tmp = tempfile::tempdir().unwrap();
        let err = find_root(tmp.path()).unwrap_err().to_string();
        assert!(err.contains("no nabu root found"), "{err}");
        assert!(err.contains("fix:"), "{err}");
    }

    #[test]
    fn global_dir_prefers_xdg() {
        let dir = global_dir(Some("/xdg".into()), Some("/home/u".into()));
        assert_eq!(dir, Some(PathBuf::from("/xdg/nabu")));
    }

    #[test]
    fn empty_or_relative_xdg_falls_back_to_home() {
        for xdg in ["", "relative/dir"] {
            let dir = global_dir(Some(xdg.into()), Some("/home/u".into()));
            assert_eq!(dir, Some(PathBuf::from("/home/u/.config/nabu")), "{xdg:?}");
        }
    }

    #[test]
    fn no_variables_skip_the_global_layer() {
        assert_eq!(global_dir(None, None), None);
        assert_eq!(global_dir(Some("".into()), Some("".into())), None);
    }
}
