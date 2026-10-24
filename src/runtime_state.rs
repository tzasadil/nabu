//! Files of a skill folder that `nabu init` never installs. Loaded by `build.rs` through `#[path]`
//! (it cannot use the crate), so this file must not use anything from the crate.

/// `rel` is the repo-relative path with `/`, such as `skills/nabu-learn/last-run.json`.
/// `last-run.json` and `receipts.md` are the cursor and ledger of `nabu-learn`, which runs its repo
/// copy and keeps them there. `receipts.md` quotes private sessions => gitignored, never shipped.
pub fn is_runtime_state(rel: &str) -> bool {
    rel == "skills/nabu-learn/last-run.json"
        || rel == "skills/nabu-learn/receipts.md"
        || rel.ends_with(".pyc")
        || rel.split('/').any(|part| part == "__pycache__")
}

#[cfg(test)]
mod tests {
    use super::is_runtime_state;

    #[test]
    fn only_runtime_state_is_skipped() {
        for skipped in [
            "skills/nabu-learn/last-run.json",
            "skills/nabu-learn/receipts.md",
            "skills/nabu-learn/scripts/__pycache__/mine.cpython-313.pyc",
            "skills/nabu-verify/scripts/__pycache__",
            "skills/nabu-inscribe/scripts/check_bundle.pyc",
        ] {
            assert!(is_runtime_state(skipped), "{skipped}");
        }
        for installed in [
            "skills/nabu-learn/SKILL.md",
            "skills/nabu-learn/scripts/mine_critiques.py",
            "skills/other/last-run.json",
            "skills/nabu-learn/__pycache__x/file.md",
            "agents/implementer.md",
        ] {
            assert!(!is_runtime_state(installed), "{installed}");
        }
    }
}
