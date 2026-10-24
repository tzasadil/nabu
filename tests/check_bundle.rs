//! Guards the exit codes of the refine gate script `check_bundle.py`.
//! Exit 2 on a folder without `tasks.md` matters most: the gate must never pass on empty input.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

const SCRIPT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/skills/nabu-inscribe/scripts/check_bundle.py"
);

/// A throwaway repo holding `src/lib.rs` and one tablet folder.
struct Bundle {
    _tmp: TempDir,
    folder: PathBuf,
}

impl Bundle {
    fn new(tasks: Option<&str>, blueprint: &str) -> Self {
        Self::named("demo", tasks, blueprint)
    }

    fn named(tablet: &str, tasks: Option<&str>, blueprint: &str) -> Self {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path().canonicalize().unwrap().join("repo");
        let folder = repo.join("nabu/tablets").join(tablet);
        fs::create_dir_all(&folder).unwrap();
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::write(repo.join("src/lib.rs"), "").unwrap();
        if let Some(tasks) = tasks {
            fs::write(folder.join("tasks.md"), tasks).unwrap();
        }
        fs::write(folder.join("blueprint.md"), blueprint).unwrap();
        Self { _tmp: tmp, folder }
    }

    /// Exit code and stdout.
    fn check(&self) -> (i32, String) {
        let out = Command::new("python3")
            .args(["-I", SCRIPT])
            .arg(&self.folder)
            .output()
            .unwrap();
        (
            out.status.code().unwrap(),
            String::from_utf8(out.stdout).unwrap(),
        )
    }
}

const CLEAN_TASKS: &str = "# TODO Checklist\n\n## 1. Work\n- [ ] 1.1 Edit `src/lib.rs`\n";
const CLEAN_BLUEPRINT: &str =
    "# Blueprint demo\n\n## Gotchas\n- UNCERTAIN: a claim. Settled by task 1.1.\n";

#[test]
fn clean_folder_exits_zero() {
    let (code, out) = Bundle::new(Some(CLEAN_TASKS), CLEAN_BLUEPRINT).check();
    assert_eq!((code, out.as_str()), (0, "0 error(s), 0 warning(s)\n"));
}

#[test]
fn folder_without_tasks_exits_two() {
    let (code, _) = Bundle::new(None, CLEAN_BLUEPRINT).check();
    assert_eq!(code, 2);
}

#[test]
fn unresolved_task_path_exits_one() {
    let tasks = "# TODO Checklist\n\n## 1. Work\n- [ ] 1.1 Edit `src/missing.rs`\n";
    let (code, out) = Bundle::new(Some(tasks), CLEAN_BLUEPRINT).check();
    assert_eq!(code, 1);
    assert!(
        out.contains("path does not resolve: `src/missing.rs`"),
        "{out}"
    );
}

#[test]
fn uncertain_outside_gotchas_exits_one() {
    let blueprint = "# Blueprint demo\n\n## Agent Decisions\n- UNCERTAIN: a claim.\n";
    let (code, out) = Bundle::new(Some(CLEAN_TASKS), blueprint).check();
    assert_eq!(code, 1);
    assert!(out.contains("`UNCERTAIN` outside Gotchas"), "{out}");
}

#[test]
fn tablet_named_nabu_still_finds_the_repo_root() {
    let (code, out) = Bundle::named("nabu", Some(CLEAN_TASKS), CLEAN_BLUEPRINT).check();
    assert_eq!((code, out.as_str()), (0, "0 error(s), 0 warning(s)\n"));
}
