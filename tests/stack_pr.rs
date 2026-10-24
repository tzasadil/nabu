//! Guards the stacked-PR script `stack_pr.py` of `nabu-ship` against a throwaway jj repo, a local bare
//! remote, and a fake `gh` that logs its calls. Two contracts matter: each PR is based on the slice below,
//! and a range that `jj git push` would reject pushes nothing.
//! Skipped when `jj` or `git` is missing: the script drives both.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

const SCRIPT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/skills/nabu-ship/scripts/stack_pr.py"
);
const FAKE_GH: &str = "#!/bin/sh\necho \"$@\" >> \"$GH_LOG\"\n\
[ \"$1 $2\" = \"pr view\" ] && echo \"https://example.test/pr/$3\"\nexit 0\n";

struct Stack {
    tmp: TempDir,
}

impl Stack {
    /// `main` pushed to the bare remote, then `demo: stack` -> `demo 1: first` (adds a blueprint) -> `demo 2: second`.
    fn new() -> Option<Self> {
        for tool in ["jj", "git"] {
            if Command::new(tool).arg("--version").output().is_err() {
                return None;
            }
        }
        let stack = Self {
            tmp: tempfile::tempdir().unwrap(),
        };
        let root = stack.root();
        fs::write(
            root.join("jj.toml"),
            "[user]\nname = \"t\"\nemail = \"t@example.com\"\n",
        )
        .unwrap();
        fs::create_dir(root.join("bin")).unwrap();
        fs::write(root.join("bin/gh"), FAKE_GH).unwrap();
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(root.join("bin/gh"), fs::Permissions::from_mode(0o755)).unwrap();
        assert!(
            Command::new("git")
                .args(["init", "-q", "--bare"])
                .arg(root.join("remote.git"))
                .status()
                .unwrap()
                .success()
        );
        stack.jj_in(&root, &["git", "init", "repo"]);
        fs::write(stack.repo().join("README.md"), "base\n").unwrap();
        stack.jj(&["describe", "-m", "base"]);
        stack.jj(&["bookmark", "create", "main", "-r", "@"]);
        let remote = root.join("remote.git");
        stack.jj(&["git", "remote", "add", "origin", remote.to_str().unwrap()]);
        stack.jj(&["git", "push", "--bookmark", "main"]);
        // The push made @ immutable => jj opened a fresh @ on top of main.
        stack.jj(&["describe", "-m", "demo: stack"]);
        stack.write("nabu/tablets/demo/stack.md", "# Stack\n");
        stack.jj(&["new", "-m", "demo 1: first"]);
        stack.write(
            "nabu/tablets/demo/1-first/blueprint.md",
            "# Blueprint\n\n## Feature description\n\nFirst slice adds X.\n\n## Agent Decisions\n\n- d\n",
        );
        stack.jj(&["new", "-m", "demo 2: second"]);
        stack.write("y.txt", "y\n");
        stack.jj(&["new"]);
        Some(stack)
    }

    fn root(&self) -> PathBuf {
        self.tmp.path().canonicalize().unwrap()
    }

    fn repo(&self) -> PathBuf {
        self.root().join("repo")
    }

    fn write(&self, rel: &str, text: &str) {
        let path = self.repo().join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    fn command(&self, program: &str, dir: &Path) -> Command {
        let root = self.root();
        let path = format!(
            "{}:{}",
            root.join("bin").display(),
            std::env::var("PATH").unwrap()
        );
        let mut cmd = Command::new(program);
        cmd.current_dir(dir)
            .env("JJ_CONFIG", root.join("jj.toml"))
            .env("GH_LOG", root.join("gh.log"))
            .env("PATH", path);
        cmd
    }

    fn jj_in(&self, dir: &Path, args: &[&str]) -> String {
        let out = self.command("jj", dir).args(args).output().unwrap();
        assert!(
            out.status.success(),
            "jj {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap()
    }

    fn jj(&self, args: &[&str]) -> String {
        self.jj_in(&self.repo(), args)
    }

    fn ship(&self, extra: &[&str]) -> Output {
        self.command("python3", &self.repo())
            .args([
                SCRIPT,
                "-t",
                "main",
                "-s",
                "description(glob:\"demo 1:*\")",
                "-e",
                "@-",
            ])
            .args(extra)
            .output()
            .unwrap()
    }

    fn gh_log(&self) -> String {
        fs::read_to_string(self.root().join("gh.log")).unwrap_or_default()
    }

    fn remote_bookmarks(&self) -> String {
        self.jj(&["bookmark", "list", "--all-remotes", "glob:demo-*"])
    }
}

#[test]
fn each_pr_is_based_on_the_slice_below() {
    let Some(stack) = Stack::new() else { return };
    let out = stack.ship(&[]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let creates: Vec<String> = stack
        .gh_log()
        .lines()
        .filter(|l| l.starts_with("pr create"))
        .map(String::from)
        .collect();
    assert_eq!(
        creates,
        [
            "pr create --base main --head demo-1-first --title demo 1: first --body First slice adds X.",
            "pr create --base demo-1-first --head demo-2-second --title demo 2: second --body demo 2: second",
        ]
    );
    let remote = stack.remote_bookmarks();
    assert!(
        remote.contains("demo-1-first:") && remote.contains("demo-2-second:"),
        "{remote}"
    );
    assert_eq!(remote.matches("@origin:").count(), 2, "{remote}");
}

#[test]
fn dry_run_pushes_and_creates_nothing() {
    let Some(stack) = Stack::new() else { return };
    let out = stack.ship(&["--dry-run"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        String::from_utf8(out.stdout).unwrap(),
        "PR: demo-1-first -> main\nPR: demo-2-second -> demo-1-first\n"
    );
    assert_eq!(stack.gh_log(), "");
    assert_eq!(stack.remote_bookmarks(), "");
}

#[test]
fn a_revision_without_description_below_the_stack_pushes_nothing() {
    let Some(stack) = Stack::new() else { return };
    stack.jj(&[
        "new",
        "--insert-after",
        "main",
        "--insert-before",
        "description(glob:\"demo: stack*\")",
        "--no-edit",
    ]);
    let out = stack.ship(&[]);
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.starts_with("error: revisions without description"),
        "{stderr}"
    );
    assert!(stderr.contains("\nfix: jj describe -r "), "{stderr}");
    assert_eq!(stack.gh_log(), "");
    assert_eq!(stack.remote_bookmarks(), "");
}
