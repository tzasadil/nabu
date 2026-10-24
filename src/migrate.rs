//! `nabu migrate`: prints the prompt an agent follows to convert a spec folder to `nabu/`.
//! The agent moves the files, so the same prompt serves other spec frameworks and `jj op restore`
//! undoes a bad run. The text is built in only: one consumer, nothing to layer.

use crate::prompt::{block, render};
use std::path::Path;

const TEXT: &str = include_str!("migrate.md");

/// The block names, in print order. `migrate.md` holds one section per name.
const SECTIONS: [&str; 3] = ["instruction", "layout", "mapping"];

pub fn run(root: &Path) -> String {
    let blocks: Vec<Option<String>> = SECTIONS
        .iter()
        .map(|name| block(name, section(name)))
        .collect();
    render(&[("migrate", root.display().to_string())], &blocks)
}

/// Text after the `@@ <name>` line, up to the next `@@` line. Sections hold no `@@` line of their own.
fn section(name: &str) -> &'static str {
    let marker = format!("@@ {name}\n");
    let (_, rest) = TEXT
        .split_once(&marker)
        .unwrap_or_else(|| panic!("src/migrate.md has no `@@ {name}` section"));
    rest.split_once("\n@@ ").map_or(rest, |(text, _)| text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_section_is_found_and_none_swallows_another() {
        for name in SECTIONS {
            let text = section(name);
            assert!(!text.trim().is_empty(), "{name} is empty");
            assert!(!text.contains("\n@@ "), "{name} runs into the next section");
        }
        assert!(section("instruction").contains("sha256sum"));
        assert!(section("mapping").contains("9067f0b770c3"));
    }
}
