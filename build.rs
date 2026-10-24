//! Generates the list of skill and agent files that `nabu init` installs.
//! A hand list would drift on every skill edit.

use std::fs;
use std::path::Path;

// The skip rule is shared with the unit tests of the crate, which build scripts cannot import.
#[path = "src/runtime_state.rs"]
mod runtime_state;

use runtime_state::is_runtime_state;

/// `fs::read_dir` order is arbitrary, so sort for a reproducible build.
/// `is_dir` and `is_file` follow symlinks: `skills/nabu-verify/implementer-rules.md` is a link,
/// and `include_bytes!` embeds its target's bytes => an install never depends on a link.
fn walk(dir: &Path, rel: &str, out: &mut Vec<String>) {
    let mut names: Vec<String> = fs::read_dir(dir)
        .unwrap_or_else(|err| panic!("cannot read {}: {err}", dir.display()))
        .map(|entry| entry.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    names.sort();
    for name in names {
        let child = format!("{rel}/{name}");
        let path = dir.join(&name);
        if is_runtime_state(&child) {
            continue;
        }
        if path.is_dir() {
            walk(&path, &child, out);
        } else if path.is_file() {
            out.push(child);
        } else {
            // A dangling link: skipping it would ship an install that silently lacks a file.
            panic!("{} is neither a file nor a folder", path.display());
        }
    }
}

fn main() {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest = Path::new(&manifest);
    let mut files = Vec::new();
    for top in ["skills", "agents"] {
        // Cargo scans the folder recursively for changes.
        println!("cargo:rerun-if-changed={top}");
        walk(&manifest.join(top), top, &mut files);
    }
    let entries: String = files
        .iter()
        .map(|rel| {
            let abs = manifest.join(rel);
            format!(
                "    ({rel:?}, include_bytes!({:?})),\n",
                abs.to_str().unwrap()
            )
        })
        .collect();
    let out = Path::new(&std::env::var("OUT_DIR").unwrap()).join("embedded.rs");
    fs::write(
        out,
        format!("/// Repo-relative path (`skills/<name>/...`, `agents/<file>`) and file bytes.\npub const EMBEDDED: &[(&str, &[u8])] = &[\n{entries}];\n"),
    )
    .unwrap();
}
