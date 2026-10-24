//! Helpers shared by unit tests.

use std::fs;
use std::path::Path;

/// Appends the `/`-joined path of every file under `dir`, links followed. `rel` prefixes the
/// paths, empty for none.
pub fn walk_files(dir: &Path, rel: &str, out: &mut Vec<String>) {
    for entry in fs::read_dir(dir).unwrap() {
        let name = entry.unwrap().file_name().to_string_lossy().to_string();
        let child = if rel.is_empty() {
            name.clone()
        } else {
            format!("{rel}/{name}")
        };
        if dir.join(&name).is_dir() {
            walk_files(&dir.join(&name), &child, out);
        } else {
            out.push(child);
        }
    }
}
