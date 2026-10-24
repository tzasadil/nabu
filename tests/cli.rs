//! Runs the real binary in a temp copy of a fixture, with `HOME` and
//! `XDG_CONFIG_HOME` pointing at temp folders so the user's config never leaks in.

use jiff::Timestamp;
use jiff::tz::TimeZone;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

/// A schema without a `stack` key, distinct from the built-in `specless`.
/// Its template `t.md` must be in some layer: `write_other_schema` adds both.
const OTHER_SCHEMA: &str = r#"name = "other"
[[artifacts]]
id = "t"
generates = "t.md"
template = "t.md"
[build]
requires = []
tracks = "t.md"
"#;

struct Sandbox {
    _tmp: TempDir,
    /// Canonical, because the binary reports `getcwd`, which resolves symlinks.
    base: PathBuf,
    repo: PathBuf,
    home: PathBuf,
    xdg: PathBuf,
}

struct Output {
    status: i32,
    stdout: String,
    stderr: String,
}

impl Sandbox {
    fn empty() -> Self {
        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path().canonicalize().unwrap();
        let sandbox = Self {
            repo: base.join("repo"),
            home: base.join("home"),
            xdg: base.join("xdg"),
            base,
            _tmp: tmp,
        };
        for dir in [&sandbox.repo, &sandbox.home, &sandbox.xdg] {
            fs::create_dir_all(dir).unwrap();
        }
        sandbox
    }

    fn with_fixture(fixture: &str) -> Self {
        let sandbox = Self::empty();
        sandbox.overlay(fixture);
        sandbox
    }

    fn overlay(&self, fixture: &str) {
        copy_dir(
            &Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures")
                .join(fixture),
            &self.repo,
        );
    }

    fn write(&self, rel: &str, text: &str) {
        let path = self.repo.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    fn write_global(&self, rel: &str, text: &str) {
        let path = self.xdg.join("nabu").join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    fn write_global_bytes(&self, rel: &str, bytes: &[u8]) {
        let path = self.xdg.join("nabu").join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }

    fn write_other_schema(&self) {
        self.write_global("schemas/other/schema.toml", OTHER_SCHEMA);
        self.write_global("schemas/other/templates/t.md", "# T\n");
    }

    fn nabu(&self, args: &[&str]) -> Output {
        self.nabu_in(&self.repo, args)
    }

    fn nabu_in(&self, cwd: &Path, args: &[&str]) -> Output {
        self.nabu_run(cwd, args, &[])
    }

    /// `envs` extends the otherwise empty environment, e.g. `TZ` for date tests.
    fn nabu_env(&self, args: &[&str], envs: &[(&str, &str)]) -> Output {
        self.nabu_run(&self.repo, args, envs)
    }

    fn nabu_run(&self, cwd: &Path, args: &[&str], envs: &[(&str, &str)]) -> Output {
        let out = Command::new(env!("CARGO_BIN_EXE_nabu"))
            .args(args)
            .current_dir(cwd)
            .env_clear()
            .env("HOME", &self.home)
            .env("XDG_CONFIG_HOME", &self.xdg)
            .envs(envs.iter().copied())
            .output()
            .unwrap();
        let clean = |bytes: &[u8]| {
            String::from_utf8_lossy(bytes).replace(self.base.to_str().unwrap(), "[TMP]")
        };
        Output {
            status: out.status.code().unwrap(),
            stdout: clean(&out.stdout),
            stderr: clean(&out.stderr),
        }
    }

    /// Sorted relative paths, with file text. Used to prove a refusal wrote nothing.
    fn tree(&self) -> String {
        folder_tree(&self.repo)
    }
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn tree_lines(dir: &Path, rel: &str, lines: &mut Vec<String>) {
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name().to_string_lossy().to_string();
        if name == ".jj" {
            continue;
        }
        let child = if rel.is_empty() {
            name
        } else {
            format!("{rel}/{name}")
        };
        if entry.path().is_dir() {
            lines.push(format!("{child}/"));
            tree_lines(&entry.path(), &child, lines);
        } else {
            lines.push(format!(
                "{child}\n{}",
                fs::read_to_string(entry.path()).unwrap()
            ));
        }
    }
}

/// Success: exit 0, stdout only. Failure: exit 1, stderr only.
fn show(out: &Output) -> String {
    match out.status {
        0 => {
            assert_eq!(out.stderr, "");
            format!("exit 0\n{}", out.stdout)
        }
        _ => {
            assert_eq!(out.stdout, "");
            format!("exit {}\n{}", out.status, out.stderr)
        }
    }
}

/// For commands whose exit code is not tied to the stream: stdout is the report, stderr stays empty.
fn show_report(out: &Output) -> String {
    assert_eq!(out.stderr, "");
    format!("exit {}\n{}", out.status, out.stdout)
}

fn assert_refused(sandbox: &Sandbox, args: &[&str]) -> String {
    assert_refused_env(sandbox, args, &[])
}

fn assert_refused_env(sandbox: &Sandbox, args: &[&str], envs: &[(&str, &str)]) -> String {
    let before = sandbox.tree();
    let out = sandbox.nabu_env(args, envs);
    assert_eq!(out.status, 1, "{}", out.stderr);
    assert_eq!(sandbox.tree(), before, "refusal changed the tree");
    show(&out)
}

// Root

#[test]
fn no_root_fails_with_a_fix() {
    let sandbox = Sandbox::empty();
    insta::assert_snapshot!(show(&sandbox.nabu(&["new", "demo"])));
}

#[test]
fn nested_cwd_finds_the_root() {
    let sandbox = Sandbox::with_fixture("repo");
    let nested = sandbox.repo.join("nabu/tablets/42-empty-list-notice");
    let out = sandbox.nabu_in(&nested, &["new", "demo"]);
    insta::assert_snapshot!(show(&out));
}

// Layers and config

#[test]
fn malformed_repo_config_names_the_file() {
    let sandbox = Sandbox::with_fixture("repo");
    sandbox.write("nabu/config.toml", "schema = \n");
    insta::assert_snapshot!(assert_refused(&sandbox, &["new", "demo"]));
}

#[test]
fn unknown_key_in_global_config_names_the_file() {
    let sandbox = Sandbox::with_fixture("repo");
    sandbox.write_global("config.toml", "shema = \"specless\"\n");
    insta::assert_snapshot!(assert_refused(&sandbox, &["new", "demo"]));
}

#[test]
fn global_and_repo_config_layers_apply() {
    let sandbox = Sandbox::with_fixture("repo");
    sandbox.write_global("config.toml", "schema = \"other\"\n");
    sandbox.write_other_schema();
    let out = sandbox.nabu(&["new", "from-global"]);
    assert_eq!(out.status, 0, "{}", out.stderr);
    let file =
        fs::read_to_string(sandbox.repo.join("nabu/tablets/from-global/.nabu.toml")).unwrap();
    assert_eq!(file, "schema = \"other\"\n");
    // The repo layer is nearer and wins.
    sandbox.write("nabu/config.toml", "schema = \"specless\"\n");
    sandbox.nabu(&["new", "from-repo"]);
    let file = fs::read_to_string(sandbox.repo.join("nabu/tablets/from-repo/.nabu.toml")).unwrap();
    assert_eq!(file, "schema = \"specless\"\n");
}

#[test]
fn empty_xdg_config_home_falls_back_to_home() {
    let sandbox = Sandbox::with_fixture("repo");
    let config = sandbox.home.join(".config/nabu");
    fs::create_dir_all(&config).unwrap();
    fs::write(config.join("config.toml"), "bogus = 1\n").unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_nabu"))
        .args(["new", "demo"])
        .current_dir(&sandbox.repo)
        .env_clear()
        .env("HOME", &sandbox.home)
        .env("XDG_CONFIG_HOME", "")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains(".config/nabu/config.toml"), "{stderr}");
}

// Schemas

#[test]
fn unknown_schema_lists_available_ones() {
    let sandbox = Sandbox::with_fixture("repo");
    sandbox.write("nabu/config.toml", "schema = \"nope\"\n");
    insta::assert_snapshot!(assert_refused(&sandbox, &["new", "demo"]));
}

#[test]
fn schema_missing_a_required_key_names_the_file() {
    let sandbox = Sandbox::with_fixture("repo");
    sandbox.write(
        "nabu/schemas/specless/schema.toml",
        "name = \"specless\"\nartifacts = []\n",
    );
    insta::assert_snapshot!(assert_refused(&sandbox, &["new", "demo"]));
}

#[test]
fn schema_with_unknown_key_names_the_file() {
    let sandbox = Sandbox::with_fixture("repo");
    sandbox.write(
        "nabu/schemas/specless/schema.toml",
        r#"name = "specless"
artifacts = []
stak = "x"
[build]
requires = []
tracks = "t.md"
"#,
    );
    insta::assert_snapshot!(assert_refused(&sandbox, &["new", "demo"]));
}

#[test]
fn unresolvable_stack_template_names_file_and_template() {
    let sandbox = Sandbox::with_fixture("repo");
    sandbox.write(
        "nabu/schemas/specless/schema.toml",
        r#"name = "specless"
stack = "missing.md"
artifacts = []
[build]
requires = []
tracks = "t.md"
"#,
    );
    insta::assert_snapshot!(assert_refused(&sandbox, &["new", "demo"]));
}

// Names and lookup

#[test]
fn slice_under_unknown_tablet_lists_active_tablets_only() {
    let sandbox = Sandbox::with_fixture("repo");
    insta::assert_snapshot!(assert_refused(
        &sandbox,
        &["new", "split-sync-worker/first"]
    ));
}

#[test]
fn invalid_names_are_refused() {
    let sandbox = Sandbox::with_fixture("repo");
    let mut report = String::new();
    for target in ["Bad_Name", "a_b", "demo/1st", "demo/Up", "a/b/c", "demo/"] {
        report += &format!(
            "$ nabu new {target}\n{}\n",
            assert_refused(&sandbox, &["new", target])
        );
    }
    insta::assert_snapshot!(report);
}

// new

#[test]
fn zero_config_new_tablet_writes_the_tablet_file() {
    let sandbox = Sandbox::empty();
    fs::create_dir(sandbox.repo.join(".jj")).unwrap();
    let out = sandbox.nabu(&["new", "demo"]);
    insta::assert_snapshot!(show(&out));
    insta::assert_snapshot!(sandbox.tree());
}

#[test]
fn new_stack_writes_the_resolved_stack_template() {
    let sandbox = Sandbox::with_fixture("repo");
    let out = sandbox.nabu(&["new", "demo", "--stack"]);
    assert_eq!(show(&out), "exit 0\nnabu/tablets/demo\n");
    let written = fs::read_to_string(sandbox.repo.join("nabu/tablets/demo/stack.md")).unwrap();
    let template = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("defaults/schemas/specless/templates/stack.md"),
    )
    .unwrap();
    assert_eq!(written, template);
}

#[test]
fn repo_template_overrides_the_stack_template_only_for_that_file() {
    let sandbox = Sandbox::with_fixture("repo");
    sandbox.write("nabu/schemas/specless/templates/stack.md", "my stack\n");
    sandbox.nabu(&["new", "demo", "--stack"]);
    let written = fs::read_to_string(sandbox.repo.join("nabu/tablets/demo/stack.md")).unwrap();
    assert_eq!(written, "my stack\n");
}

#[test]
fn new_slice_numbers_after_the_fixture_slices() {
    let sandbox = Sandbox::with_fixture("repo");
    let out = sandbox.nabu(&["new", "todo-reminders/snooze"]);
    assert_eq!(show(&out), "exit 0\nnabu/tablets/todo-reminders/4-snooze\n");
    assert!(
        sandbox
            .repo
            .join("nabu/tablets/todo-reminders/4-snooze")
            .is_dir()
    );
    let marker = fs::read_to_string(
        sandbox
            .repo
            .join("nabu/tablets/todo-reminders/4-snooze/.nabu.toml"),
    )
    .unwrap();
    assert_eq!(marker, "", "the slice marker is empty");
}

#[test]
fn new_slice_follows_the_highest_number_and_ignores_other_folders() {
    let sandbox = Sandbox::with_fixture("repo");
    let tablet = sandbox.repo.join("nabu/tablets/todo-reminders");
    // A gap must not be refilled, and a non-slice folder must not count.
    fs::create_dir(tablet.join("7-late")).unwrap();
    fs::write(tablet.join("7-late/.nabu.toml"), "").unwrap();
    fs::create_dir(tablet.join("99notes")).unwrap();
    // A slice-shaped folder without the marker is not a slice.
    fs::create_dir(tablet.join("12-unmarked")).unwrap();
    let out = sandbox.nabu(&["new", "todo-reminders/snooze"]);
    assert_eq!(show(&out), "exit 0\nnabu/tablets/todo-reminders/8-snooze\n");
}

#[test]
fn new_slice_uses_the_tablets_schema_not_the_configured_one() {
    let sandbox = Sandbox::with_fixture("repo");
    // The configured schema has no `stack` key. The tablet's own `specless` does.
    sandbox.write_global("config.toml", "schema = \"other\"\n");
    sandbox.write_other_schema();
    let out = sandbox.nabu(&["new", "todo-reminders/snooze"]);
    assert_eq!(show(&out), "exit 0\nnabu/tablets/todo-reminders/4-snooze\n");
}

#[test]
fn bad_tablet_file_names_the_file() {
    let sandbox = Sandbox::with_fixture("repo");
    for (text, needle) in [("schema = 1\n", "schema"), ("bogus = 1\n", "bogus")] {
        sandbox.write("nabu/tablets/todo-reminders/.nabu.toml", text);
        let shown = assert_refused(&sandbox, &["new", "todo-reminders/snooze"]);
        assert!(
            shown.contains("nabu/tablets/todo-reminders/.nabu.toml"),
            "{shown}"
        );
        assert!(shown.contains(needle), "{shown}");
    }
}

#[test]
fn refusals_leave_the_tree_unchanged() {
    let sandbox = Sandbox::with_fixture("repo");
    let cases: &[&[&str]] = &[
        &["new", "42-empty-list-notice"],
        &["new", "todo-reminders/reminder-delivery"],
        &["new", "nope/first"],
        &["new", "42-empty-list-notice/first"],
        &["new", "todo-reminders/first", "--stack"],
    ];
    let mut report = String::new();
    for args in cases {
        report += &format!(
            "$ nabu {}\n{}\n",
            args.join(" "),
            assert_refused(&sandbox, args)
        );
    }
    insta::assert_snapshot!(report);
}

#[test]
fn stack_under_the_override_schema_is_refused() {
    let sandbox = Sandbox::with_fixture("repo");
    sandbox.overlay("override");
    insta::assert_snapshot!(assert_refused(&sandbox, &["new", "demo", "--stack"]));
    // The override still creates plain tablets, and slices need a stack key.
    assert_eq!(sandbox.nabu(&["new", "demo"]).status, 0);
    insta::assert_snapshot!(assert_refused(&sandbox, &["new", "demo/first"]));
}

// Graph checks

#[test]
fn status_refuses_a_schema_with_a_dependency_cycle() {
    let sandbox = Sandbox::with_fixture("repo");
    let builtin = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("defaults/schemas/specless/schema.toml"),
    )
    .unwrap();
    // `blueprint` now requires `tasks`, which already requires `blueprint`.
    sandbox.write(
        "nabu/schemas/specless/schema.toml",
        &builtin.replace("requires = []", "requires = [\"tasks\"]"),
    );
    insta::assert_snapshot!(assert_refused(
        &sandbox,
        &["status", "42-empty-list-notice"]
    ));
}

// status

const TASKS_TEMPLATE: &str = include_str!("../defaults/schemas/specless/templates/tasks.md");

#[test]
fn status_plain_tablet() {
    let sandbox = Sandbox::with_fixture("repo");
    insta::assert_snapshot!(show(&sandbox.nabu(&["status", "42-empty-list-notice"])));
}

#[test]
fn status_stacked_tablet() {
    let sandbox = Sandbox::with_fixture("repo");
    insta::assert_snapshot!(show(&sandbox.nabu(&["status", "todo-reminders"])));
}

#[test]
fn status_slice_target_prints_one_block_under_the_tablet_header() {
    let sandbox = Sandbox::with_fixture("repo");
    insta::assert_snapshot!(show(&sandbox.nabu(&["status", "todo-reminders/2"])));
}

#[test]
fn template_equal_tasks_file_is_not_done_and_blocks_archive() {
    let sandbox = Sandbox::with_fixture("repo");
    let slice = "nabu/tablets/todo-reminders/2-reminder-delivery";
    sandbox.write(
        &format!("{slice}/tasks.md"),
        &TASKS_TEMPLATE.replace('\n', "\r\n"),
    );
    insta::assert_snapshot!(show(&sandbox.nabu(&["status", "todo-reminders/2"])));
}

#[test]
fn stray_files_and_folders_outside_the_layout_are_ignored() {
    let sandbox = Sandbox::with_fixture("repo");
    let stacked_before = show(&sandbox.nabu(&["status", "todo-reminders"]));
    let plain_before = show(&sandbox.nabu(&["status", "42-empty-list-notice"]));
    // A stacked tablet reads artifacts from slices only.
    sandbox.write("nabu/tablets/todo-reminders/blueprint.md", "# Stray\n");
    // A plain tablet has no slices, even when a folder looks like one.
    let fake = "nabu/tablets/42-empty-list-notice/1-fake";
    sandbox.write(&format!("{fake}/.nabu.toml"), "");
    sandbox.write(&format!("{fake}/tasks.md"), "- [ ] never counted\n");
    assert_eq!(
        show(&sandbox.nabu(&["status", "todo-reminders"])),
        stacked_before
    );
    assert_eq!(
        show(&sandbox.nabu(&["status", "42-empty-list-notice"])),
        plain_before
    );
}

#[test]
fn status_refusals() {
    let sandbox = Sandbox::with_fixture("repo");
    let mut report = String::new();
    for target in [
        "42-empty-list-notice/1",
        "a/b/c",
        "todo-reminders/9",
        "nope",
        "..",
    ] {
        report += &format!(
            "$ nabu status {target}\n{}\n",
            assert_refused(&sandbox, &["status", target])
        );
    }
    insta::assert_snapshot!(report);
}

#[test]
fn unreadable_artifact_stops_status_and_names_the_file() {
    let sandbox = Sandbox::with_fixture("repo");
    let path = "nabu/tablets/42-empty-list-notice/tasks.md";
    fs::write(sandbox.repo.join(path), [0xff, 0xfe, 0x00]).unwrap();
    // Not `assert_refused`: its tree snapshot reads every file as text.
    let out = sandbox.nabu(&["status", "42-empty-list-notice"]);
    assert_eq!((out.status, out.stdout.as_str()), (1, ""));
    assert!(out.stderr.contains(path), "{}", out.stderr);
}

// list

#[test]
fn list_fixture_repo_skips_archived_tablets() {
    let sandbox = Sandbox::with_fixture("repo");
    insta::assert_snapshot!(show(&sandbox.nabu(&["list"])));
}

#[test]
fn list_without_tablets() {
    let sandbox = Sandbox::empty();
    fs::create_dir(sandbox.repo.join(".jj")).unwrap();
    insta::assert_snapshot!(show(&sandbox.nabu(&["list"])));
}

#[test]
fn list_prints_an_error_row_per_broken_tablet_and_still_prints_the_rest() {
    let sandbox = Sandbox::with_fixture("repo");
    sandbox.write("nabu/tablets/bad-toml/.nabu.toml", "schema = \n");
    let builtin = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("defaults/schemas/specless/schema.toml"),
    )
    .unwrap();
    sandbox.write(
        "nabu/schemas/cyclic/schema.toml",
        &builtin.replace("requires = []", "requires = [\"tasks\"]"),
    );
    sandbox.write("nabu/schemas/cyclic/templates/blueprint.md", "# B\n");
    sandbox.write("nabu/schemas/cyclic/templates/backlog.md", "# L\n");
    sandbox.write("nabu/schemas/cyclic/templates/tasks.md", "# T\n");
    sandbox.write("nabu/schemas/cyclic/templates/stack.md", "# S\n");
    sandbox.write("nabu/tablets/bad-graph/.nabu.toml", "schema = \"cyclic\"\n");
    let before = sandbox.tree();
    let out = sandbox.nabu(&["list"]);
    assert_eq!(out.status, 1);
    assert_eq!(sandbox.tree(), before);
    insta::assert_snapshot!(show_report(&out));
}

#[test]
fn list_counts_a_stacked_tablet_without_slices_and_never_marks_it_ready() {
    let sandbox = Sandbox::empty();
    fs::create_dir(sandbox.repo.join(".jj")).unwrap();
    assert_eq!(sandbox.nabu(&["new", "demo", "--stack"]).status, 0);
    assert_eq!(
        show(&sandbox.nabu(&["list"])),
        "exit 0\ndemo  0/0 tasks  0 slices\n"
    );
}

#[test]
fn fresh_slice_shows_blocked_artifacts_with_their_unfinished_ids() {
    let sandbox = Sandbox::empty();
    fs::create_dir(sandbox.repo.join(".jj")).unwrap();
    for args in [&["new", "demo", "--stack"][..], &["new", "demo/first"]] {
        assert_eq!(sandbox.nabu(args).status, 0);
    }
    assert_eq!(
        show(&sandbox.nabu(&["status", "demo/first"])),
        "exit 0\n\
         demo  stacked  specless  0/0 tasks\n\
         stack.md  ready\n\
         \n\
         1-first  0/0 tasks\n  \
         blueprint  ready\n  \
         backlog    blocked  needs blueprint\n  \
         tasks      blocked  needs blueprint\n"
    );
}

// Composite

#[test]
fn composite_new_stack_then_two_slices() {
    let sandbox = Sandbox::empty();
    fs::create_dir(sandbox.repo.join(".jj")).unwrap();
    let mut report = String::new();
    for args in [
        &["new", "demo", "--stack"][..],
        &["new", "demo/first"],
        &["new", "demo/second"],
    ] {
        report += &format!("$ nabu {}\n{}\n", args.join(" "), show(&sandbox.nabu(args)));
    }
    insta::assert_snapshot!(report);
    insta::assert_snapshot!(sandbox.tree());
}

#[test]
fn malformed_config_refuses_slice_creation_too() {
    let sandbox = Sandbox::with_fixture("repo");
    sandbox.write("nabu/config.toml", "schema = \n");
    let shown = assert_refused(&sandbox, &["new", "todo-reminders/snooze"]);
    assert!(shown.contains("nabu/config.toml"), "{shown}");
    sandbox.write("nabu/config.toml", "shema = \"specless\"\n");
    let shown = assert_refused(&sandbox, &["new", "todo-reminders/snooze"]);
    assert!(shown.contains("shema"), "{shown}");
}

#[test]
fn stack_value_that_is_not_a_plain_file_name_is_refused_before_any_write() {
    let sandbox = Sandbox::with_fixture("repo");
    sandbox.write(
        "nabu/schemas/specless/schema.toml",
        "name = \"specless\"\nstack = \"sub/x.md\"\nartifacts = []\n[build]\nrequires = []\ntracks = \"t.md\"\n",
    );
    sandbox.write("nabu/schemas/specless/templates/sub/x.md", "x");
    let shown = assert_refused(&sandbox, &["new", "demo", "--stack"]);
    assert!(
        shown.contains("nabu/schemas/specless/schema.toml"),
        "{shown}"
    );
    assert!(shown.contains("\"sub/x.md\""), "{shown}");
}

#[test]
fn stray_binary_files_in_layers_are_ignored() {
    let sandbox = Sandbox::with_fixture("repo");
    let junk = [0xff, 0xfe, 0x00, 0x80];
    sandbox.write_global_bytes(".DS_Store", &junk);
    sandbox.write_global_bytes("schemas/.DS_Store", &junk);
    sandbox.write_global_bytes("schemas/other/templates/.DS_Store", &junk);
    sandbox.write_global_bytes("schemas/other/templates/swap.md.swp", &junk);
    sandbox.write_other_schema();
    let out = sandbox.nabu(&["new", "demo"]);
    assert_eq!(show(&out), "exit 0\nnabu/tablets/demo\n");
}

#[test]
fn unknown_schema_in_a_tablet_file_points_the_fix_at_that_file() {
    let sandbox = Sandbox::with_fixture("repo");
    sandbox.write(
        "nabu/tablets/todo-reminders/.nabu.toml",
        "schema = \"nope\"\n",
    );
    let shown = assert_refused(&sandbox, &["new", "todo-reminders/snooze"]);
    assert!(
        shown.contains("in nabu/tablets/todo-reminders/.nabu.toml to one of"),
        "{shown}"
    );
}

#[test]
fn folder_errors_print_paths_relative_to_the_root() {
    let sandbox = Sandbox::with_fixture("repo");
    // A file where the tablets folder should be: reading and creating both fail.
    fs::remove_dir_all(sandbox.repo.join("nabu/tablets")).unwrap();
    sandbox.write("nabu/config.toml", "schema = \"specless\"\n");
    sandbox.write("nabu/tablets", "not a folder");
    for args in [["new", "nope/first"], ["new", "demo"]] {
        let shown = assert_refused(&sandbox, &args);
        assert!(shown.contains("nabu/tablets"), "{shown}");
        assert!(!shown.contains("[TMP]"), "{shown}");
    }
}

#[test]
fn composite_status_and_list_follow_the_written_files() {
    let sandbox = Sandbox::empty();
    fs::create_dir(sandbox.repo.join(".jj")).unwrap();
    for args in [&["new", "demo", "--stack"][..], &["new", "demo/first"]] {
        assert_eq!(sandbox.nabu(args).status, 0);
    }
    sandbox.write(
        "nabu/tablets/demo/1-first/blueprint.md",
        "# Blueprint\n\nReal content.\n",
    );
    sandbox.write(
        "nabu/tablets/demo/1-first/tasks.md",
        "# TODO\n\n- [x] 1.1 Done\n",
    );
    let mut report = String::new();
    for args in [&["status", "demo"][..], &["list"]] {
        report += &format!("$ nabu {}\n{}\n", args.join(" "), show(&sandbox.nabu(args)));
    }
    insta::assert_snapshot!(report);
}

#[test]
fn stack_file_that_is_a_folder_does_not_turn_a_stacked_tablet_plain() {
    let sandbox = Sandbox::with_fixture("repo");
    let stack = "nabu/tablets/todo-reminders/stack.md";
    fs::remove_file(sandbox.repo.join(stack)).unwrap();
    fs::create_dir(sandbox.repo.join(stack)).unwrap();
    let shown = assert_refused(&sandbox, &["status", "todo-reminders"]);
    assert!(
        shown.contains(&format!("{stack} is not a regular file")),
        "{shown}"
    );
    let out = sandbox.nabu(&["list"]);
    assert_eq!(out.status, 1);
    assert!(
        out.stdout
            .contains(&format!("error  {stack} is not a regular file")),
        "{}",
        out.stdout
    );
}

// instructions

const STACKED: &str = "todo-reminders";
const FIRST_SLICE: &str = "todo-reminders/1-reminder-schedule";

#[test]
fn instructions_artifact_for_a_fixture_slice() {
    let sandbox = Sandbox::with_fixture("repo");
    insta::assert_snapshot!(show(&sandbox.nabu(&["instructions", "tasks", FIRST_SLICE])));
}

#[test]
fn instructions_artifact_serves_context_and_rules_from_every_layer() {
    let sandbox = Sandbox::with_fixture("repo");
    sandbox.write_global(
        "config.toml",
        "context = \"\\n\\nglobal context\\n\"\n[rules]\nblueprint = [\"global rule\"]\ntasks = [\"other artifact\"]\n",
    );
    sandbox.write(
        "nabu/config.toml",
        "context = \"repo context\"\n[rules]\nblueprint = [\"repo rule\"]\n",
    );
    insta::assert_snapshot!(show(&sandbox.nabu(&[
        "instructions",
        "blueprint",
        "42-empty-list-notice"
    ])));
}

#[test]
fn instructions_artifact_blocked_still_prints() {
    let sandbox = Sandbox::with_fixture("repo");
    assert_eq!(
        sandbox.nabu(&["new", &format!("{STACKED}/snooze")]).status,
        0
    );
    insta::assert_snapshot!(show(&sandbox.nabu(&[
        "instructions",
        "tasks",
        &format!("{STACKED}/snooze")
    ])));
}

#[test]
fn instructions_build_lists_pending_tasks() {
    let sandbox = Sandbox::with_fixture("repo");
    sandbox.write(
        "nabu/tablets/todo-reminders/1-reminder-schedule/tasks.md",
        "# TODO\n\n## 1. Build\n- [x] 1.1 Done\n- [ ] 1.2 Open\n  - [ ] 1.3 Nested open\n\n## 2. Gate\n- [ ] 2.1 Verify\n",
    );
    insta::assert_snapshot!(show(&sandbox.nabu(&["instructions", "build", FIRST_SLICE])));
}

#[test]
fn instructions_build_all_done() {
    let sandbox = Sandbox::with_fixture("repo");
    insta::assert_snapshot!(show(&sandbox.nabu(&[
        "instructions",
        "build",
        "todo-reminders/2"
    ])));
}

#[test]
fn instructions_build_blocked_without_tasks_file() {
    let sandbox = Sandbox::with_fixture("repo");
    assert_eq!(
        sandbox.nabu(&["new", &format!("{STACKED}/snooze")]).status,
        0
    );
    insta::assert_snapshot!(show(&sandbox.nabu(&[
        "instructions",
        "build",
        "todo-reminders/snooze"
    ])));
}

#[test]
fn instructions_stack_for_the_stacked_fixture() {
    let sandbox = Sandbox::with_fixture("repo");
    insta::assert_snapshot!(show(&sandbox.nabu(&["instructions", "stack", STACKED])));
}

#[test]
fn instructions_stack_for_a_plain_tablet_is_ready() {
    let sandbox = Sandbox::with_fixture("repo");
    insta::assert_snapshot!(show(&sandbox.nabu(&[
        "instructions",
        "stack",
        "42-empty-list-notice"
    ])));
}

#[test]
fn instructions_unknown_artifact_lists_the_ids() {
    let sandbox = Sandbox::with_fixture("repo");
    insta::assert_snapshot!(assert_refused(
        &sandbox,
        &["instructions", "spec", "42-empty-list-notice"]
    ));
}

#[test]
fn instructions_stacked_root_points_at_the_slices() {
    let sandbox = Sandbox::with_fixture("repo");
    insta::assert_snapshot!(assert_refused(
        &sandbox,
        &["instructions", "tasks", STACKED]
    ));
}

#[test]
fn instructions_stack_with_a_slice_target_is_refused() {
    let sandbox = Sandbox::with_fixture("repo");
    insta::assert_snapshot!(assert_refused(
        &sandbox,
        &["instructions", "stack", FIRST_SLICE]
    ));
}

#[test]
fn instructions_stack_without_a_stack_key_is_refused() {
    let sandbox = Sandbox::with_fixture("repo");
    sandbox.write_other_schema();
    sandbox.write(
        "nabu/tablets/plain-other/.nabu.toml",
        "schema = \"other\"\n",
    );
    insta::assert_snapshot!(assert_refused(
        &sandbox,
        &["instructions", "stack", "plain-other"]
    ));
}

#[test]
fn instructions_build_refuses_a_rules_key_that_is_no_artifact() {
    let sandbox = Sandbox::with_fixture("repo");
    sandbox.write_global("config.toml", "[rules]\nbuild = [\"x\"]\n");
    sandbox.write("nabu/config.toml", "[rules]\nbuild = [\"y\"]\n");
    insta::assert_snapshot!(assert_refused(
        &sandbox,
        &["instructions", "build", FIRST_SLICE]
    ));
}

#[test]
fn instructions_all_prompt_kinds_check_rules_keys() {
    let sandbox = Sandbox::with_fixture("repo");
    sandbox.write("nabu/config.toml", "[rules]\nghost = [\"x\"]\n");
    for args in [
        &["instructions", "tasks", FIRST_SLICE][..],
        &["instructions", "build", FIRST_SLICE],
        &["instructions", "stack", STACKED],
    ] {
        let shown = assert_refused(&sandbox, args);
        assert!(
            shown.contains("nabu/config.toml sets [rules] \"ghost\""),
            "{shown}"
        );
    }
}

#[test]
fn schema_with_a_path_in_generates_is_refused_by_every_command() {
    let sandbox = Sandbox::with_fixture("repo");
    let builtin = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("defaults/schemas/specless/schema.toml"),
    )
    .unwrap();
    sandbox.write(
        "nabu/schemas/specless/schema.toml",
        &builtin.replacen(
            "generates = \"blueprint.md\"",
            "generates = \"../blueprint.md\"",
            1,
        ),
    );
    for args in [
        &["new", "demo"][..],
        &["status", "42-empty-list-notice"],
        &["instructions", "tasks", FIRST_SLICE],
    ] {
        let shown = assert_refused(&sandbox, args);
        assert!(
            shown.contains("nabu/schemas/specless/schema.toml"),
            "{shown}"
        );
        assert!(shown.contains("\"../blueprint.md\""), "{shown}");
    }
    let list = sandbox.nabu(&["list"]);
    assert_eq!(list.status, 1);
    assert!(list.stdout.contains("../blueprint.md"), "{}", list.stdout);
}

#[test]
fn composite_instructions_write_path_is_where_new_made_the_folder() {
    let sandbox = Sandbox::empty();
    fs::create_dir(sandbox.repo.join(".jj")).unwrap();
    for args in [&["new", "demo", "--stack"][..], &["new", "demo/first"]] {
        assert_eq!(sandbox.nabu(args).status, 0);
    }
    let first = sandbox.nabu(&["instructions", "blueprint", "demo/first"]);
    let text = show(&first);
    let write = "write: nabu/tablets/demo/1-first/blueprint.md\n";
    assert!(text.contains(write), "{text}");
    assert!(text.contains("\nstate: ready\n"), "{text}");
    assert!(sandbox.repo.join("nabu/tablets/demo/1-first").is_dir());
    sandbox.write(
        "nabu/tablets/demo/1-first/blueprint.md",
        "# Blueprint\n\nReal.\n",
    );
    let text = show(&sandbox.nabu(&["instructions", "blueprint", "demo/first"]));
    assert!(text.contains(write), "{text}");
    assert!(text.contains("\nstate: done\n"), "{text}");
}

#[test]
fn instructions_read_list_follows_requires_transitively_in_schema_order() {
    let sandbox = Sandbox::empty();
    fs::create_dir(sandbox.repo.join(".jj")).unwrap();
    let artifact = |id: &str, requires: &str| {
        format!(
            "[[artifacts]]\nid = \"{id}\"\ngenerates = \"{id}.md\"\ntemplate = \"t.md\"\nrequires = [{requires}]\n"
        )
    };
    let schema = format!(
        "name = \"chain\"\n{}{}{}{}[build]\nrequires = [\"c\"]\ntracks = \"c.md\"\n",
        artifact("a", ""),
        artifact("b", "\"a\""),
        artifact("c", "\"b\""),
        artifact("unrelated", ""),
    );
    sandbox.write("nabu/schemas/chain/schema.toml", &schema);
    sandbox.write("nabu/schemas/chain/templates/t.md", "# T\n");
    sandbox.write("nabu/config.toml", "schema = \"chain\"\n");
    sandbox.write("nabu/tablets/demo/.nabu.toml", "schema = \"chain\"\n");
    for id in ["a", "b", "unrelated"] {
        sandbox.write(&format!("nabu/tablets/demo/{id}.md"), "# Real\n");
    }
    let read = |args: &[&str]| {
        let text = show(&sandbox.nabu(args));
        text.lines()
            .find_map(|line| line.strip_prefix("read: "))
            .unwrap()
            .to_string()
    };
    // `c` requires only `b`, which requires `a`: the grandparent is read, the unrelated artifact is not.
    let expected = "nabu/tablets/demo/a.md, nabu/tablets/demo/b.md";
    assert_eq!(read(&["instructions", "c", "demo"]), expected);
    assert_eq!(read(&["instructions", "build", "demo"]), expected);
    // Only done artifacts are listed: a missing ancestor drops out.
    fs::remove_file(sandbox.repo.join("nabu/tablets/demo/a.md")).unwrap();
    assert_eq!(
        read(&["instructions", "c", "demo"]),
        "nabu/tablets/demo/b.md"
    );
}

// archive

/// Zones 26 hours apart: at any instant their dates differ.
const FAR_EAST: &str = "Etc/GMT-14";
const FAR_WEST: &str = "Etc/GMT+12";
const PLAIN: &str = "42-empty-list-notice";

fn date_in(zone: &str) -> String {
    Timestamp::now()
        .to_zoned(TimeZone::get(zone).unwrap())
        .date()
        .to_string()
}

/// Dates a run may have used: the clock is read before and after it, and midnight may fall between.
fn dates_around(zone: &str, run: impl FnOnce()) -> Vec<String> {
    let before = date_in(zone);
    run();
    let after = date_in(zone);
    if before == after {
        vec![before]
    } else {
        vec![before, after]
    }
}

fn mask_dates(text: &str, dates: &[String]) -> String {
    dates
        .iter()
        .fold(text.to_string(), |text, date| text.replace(date, "[DATE]"))
}

/// The one `nabu/archive` folder of `tablet` created by a test, as its date prefix.
fn archived_date(sandbox: &Sandbox, tablet: &str) -> String {
    let suffix = format!("-{tablet}");
    let mut found: Vec<String> = fs::read_dir(sandbox.repo.join("nabu/archive"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().to_string())
        .filter_map(|name| name.strip_suffix(&suffix).map(str::to_string))
        .collect();
    assert_eq!(found.len(), 1, "{found:?}");
    found.remove(0)
}

fn folder_tree(dir: &Path) -> String {
    let mut lines = Vec::new();
    tree_lines(dir, "", &mut lines);
    lines.sort();
    lines.join("\n")
}

#[test]
fn archive_moves_a_stacked_tablet_whole() {
    let sandbox = Sandbox::with_fixture("repo");
    let source = sandbox.repo.join("nabu/tablets").join(STACKED);
    let content = folder_tree(&source);
    let mut out = None;
    let dates = dates_around(FAR_EAST, || {
        out = Some(sandbox.nabu_env(&["archive", STACKED], &[("TZ", FAR_EAST)]));
    });
    let out = out.unwrap();
    insta::assert_snapshot!(mask_dates(&show(&out), &dates));

    assert!(!source.exists());
    let date = archived_date(&sandbox, STACKED);
    let target = sandbox.repo.join(format!("nabu/archive/{date}-{STACKED}"));
    assert_eq!(folder_tree(&target), content);
    assert!(content.contains("stack.md"), "fixture must be stacked");
    assert!(content.contains("3-reminder-api/"));
    // Other tablets stay active.
    assert!(sandbox.repo.join("nabu/tablets").join(PLAIN).is_dir());
}

#[test]
fn archive_names_the_folder_with_the_local_date() {
    let mut names = Vec::new();
    for zone in [FAR_EAST, FAR_WEST] {
        let sandbox = Sandbox::with_fixture("repo");
        let dates = dates_around(zone, || {
            let out = sandbox.nabu_env(&["archive", PLAIN], &[("TZ", zone)]);
            assert_eq!(out.status, 0, "{}", out.stderr);
        });
        let date = archived_date(&sandbox, PLAIN);
        assert!(dates.contains(&date), "{zone}: {date} not in {dates:?}");
        names.push(date);
    }
    assert_ne!(names[0], names[1]);
}

#[test]
fn archive_creates_the_archive_folder_when_missing() {
    let sandbox = Sandbox::with_fixture("repo");
    fs::remove_dir_all(sandbox.repo.join("nabu/archive")).unwrap();
    let out = sandbox.nabu_env(&["archive", PLAIN], &[("TZ", FAR_EAST)]);
    assert_eq!(out.status, 0, "{}", out.stderr);
    archived_date(&sandbox, PLAIN);
}

#[test]
fn archive_ignores_the_schema_of_the_tablet() {
    let sandbox = Sandbox::with_fixture("repo");
    sandbox.write(
        &format!("nabu/tablets/{PLAIN}/.nabu.toml"),
        "schema = \"no-such-schema\"\n",
    );
    let out = sandbox.nabu_env(&["archive", PLAIN], &[("TZ", FAR_EAST)]);
    assert_eq!(out.status, 0, "{}", out.stderr);
    archived_date(&sandbox, PLAIN);
}

#[test]
fn archive_falls_back_to_the_utc_date_with_a_warning() {
    let sandbox = Sandbox::with_fixture("repo");
    let mut out = None;
    let dates = dates_around("UTC", || {
        out = Some(sandbox.nabu_env(&["archive", PLAIN], &[("TZ", "Not/A_Zone")]));
    });
    let out = out.unwrap();
    assert_eq!(out.status, 0, "{}", out.stderr);
    assert_eq!(
        out.stderr,
        "warning: system time zone unknown, using the UTC date; set TZ\n"
    );
    assert!(dates.contains(&archived_date(&sandbox, PLAIN)));
}

#[test]
fn archive_refuses_a_slice_target() {
    let sandbox = Sandbox::with_fixture("repo");
    let target = format!("{STACKED}/1");
    insta::assert_snapshot!(assert_refused(&sandbox, &["archive", &target]));
}

#[test]
fn archive_refuses_an_unknown_tablet() {
    let sandbox = Sandbox::with_fixture("repo");
    insta::assert_snapshot!(assert_refused(&sandbox, &["archive", "nope"]));
}

#[test]
fn archive_refuses_when_the_target_folder_exists() {
    let sandbox = Sandbox::with_fixture("repo");
    // Empty folders: `fs::rename` would replace them without the explicit check.
    // Tomorrow covers a midnight that passes between this read and the run.
    let today = date_in(FAR_EAST);
    let tomorrow = today
        .parse::<jiff::civil::Date>()
        .unwrap()
        .tomorrow()
        .unwrap();
    for date in [today.clone(), tomorrow.to_string()] {
        fs::create_dir_all(sandbox.repo.join(format!("nabu/archive/{date}-{PLAIN}"))).unwrap();
    }
    let dates = [today, tomorrow.to_string()];
    let report = assert_refused_env(&sandbox, &["archive", PLAIN], &[("TZ", FAR_EAST)]);
    insta::assert_snapshot!(mask_dates(&report, &dates));
}

#[test]
fn archive_refuses_when_the_archive_folder_cannot_be_created() {
    let sandbox = Sandbox::with_fixture("repo");
    fs::remove_dir_all(sandbox.repo.join("nabu/archive")).unwrap();
    // A file in the way: only the OS error text varies, so no snapshot.
    sandbox.write("nabu/archive", "not a folder\n");
    let report = assert_refused_env(&sandbox, &["archive", PLAIN], &[("TZ", FAR_EAST)]);
    assert!(
        report.starts_with("exit 1\nerror: cannot create nabu/archive: "),
        "{report}"
    );
    assert!(
        report.contains("\nfix: check that nabu/archive is a folder"),
        "{report}"
    );
}

// Migrate

/// The text of the prompt is not pinned: the header and the block order are the contract.
fn assert_migrate_prompt(out: &Output) {
    assert_eq!(out.status, 0, "{}", out.stderr);
    assert_eq!(out.stderr, "");
    assert!(
        out.stdout.starts_with("migrate: [TMP]/repo\n\n"),
        "{}",
        out.stdout
    );
    let tags: Vec<&str> = out
        .stdout
        .lines()
        .filter(|line| matches!(*line, "<instruction>" | "<layout>" | "<mapping>"))
        .collect();
    assert_eq!(tags, ["<instruction>", "<layout>", "<mapping>"]);
}

#[test]
fn migrate_prints_the_prompt_in_a_repo_without_nabu() {
    let sandbox = Sandbox::empty();
    fs::create_dir(sandbox.repo.join(".jj")).unwrap();
    assert_migrate_prompt(&sandbox.nabu(&["migrate"]));
}

#[test]
fn migrate_ignores_a_broken_repo_config() {
    let sandbox = Sandbox::with_fixture("repo");
    sandbox.write("nabu/config.toml", "schema = \n");
    assert_migrate_prompt(&sandbox.nabu(&["migrate"]));
}

// Init

fn manifest() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// Names of the skill folders and agent files in the checkout: what `init` must install.
fn shipped() -> (Vec<String>, Vec<String>) {
    let names = |dir: &str| {
        let mut names: Vec<String> = fs::read_dir(manifest().join(dir))
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().to_string())
            .collect();
        names.sort();
        names
    };
    (names("skills"), names("agents"))
}

/// Relative paths of every file under `dir`, links followed.
fn files_under(dir: &Path) -> Vec<String> {
    fn walk(dir: &Path, rel: &str, out: &mut Vec<String>) {
        for entry in fs::read_dir(dir).unwrap() {
            let name = entry.unwrap().file_name().to_string_lossy().to_string();
            let child = if rel.is_empty() {
                name.clone()
            } else {
                format!("{rel}/{name}")
            };
            if dir.join(&name).is_dir() {
                walk(&dir.join(&name), &child, out);
            } else {
                out.push(child);
            }
        }
    }
    let mut out = Vec::new();
    walk(dir, "", &mut out);
    out.sort();
    out
}

fn mode(path: &Path) -> u32 {
    fs::metadata(path).unwrap().permissions().mode() & 0o777
}

/// Sorted names directly in `dir`, links not followed.
fn entries(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    names.sort();
    names
}

fn is_empty_dir(dir: &Path) -> bool {
    fs::read_dir(dir).unwrap().next().is_none()
}

fn assert_init_ok(out: &Output) {
    assert_eq!(out.status, 0, "{}", out.stderr);
    assert_eq!(out.stderr, "");
}

/// One line per skill folder and agent file, then notices, then the config line.
fn assert_install_lines(out: &Output, skills_dir: &str, agents_dir: &str, config_line: &str) {
    let (skills, agents) = shipped();
    let mut expected: Vec<String> = skills.iter().map(|s| format!("{skills_dir}/{s}")).collect();
    expected.extend(agents.iter().map(|a| format!("{agents_dir}/{a}")));
    expected.push(config_line.to_string());
    let lines: Vec<&str> = out.stdout.lines().collect();
    assert_eq!(lines, expected);
}

/// Installed copies equal the checkout, minus runtime state, with links read through.
fn assert_installed(skills_dir: &Path, agents_dir: &Path) {
    let (skills, agents) = shipped();
    for skill in &skills {
        let mut expected = files_under(&manifest().join("skills").join(skill));
        expected.retain(|rel| {
            rel != "last-run.json" && rel != "receipts.md" && !rel.contains("__pycache__")
        });
        let installed = skills_dir.join(skill);
        assert_eq!(files_under(&installed), expected, "{skill}");
        for rel in &expected {
            let source = fs::read(manifest().join("skills").join(skill).join(rel)).unwrap();
            let copy = fs::read(installed.join(rel)).unwrap();
            assert_eq!(copy, source, "{skill}/{rel}");
            assert!(
                !fs::symlink_metadata(installed.join(rel))
                    .unwrap()
                    .is_symlink()
            );
            if copy.starts_with(b"#!") {
                assert_eq!(mode(&installed.join(rel)), 0o755, "{skill}/{rel}");
            }
        }
    }
    assert!(!skills_dir.join("nabu-learn/last-run.json").exists());
    assert!(!skills_dir.join("nabu-learn/receipts.md").exists());
    assert_eq!(
        fs::read(skills_dir.join("nabu-verify/implementer-rules.md")).unwrap(),
        fs::read(skills_dir.join("with-implementer/implementer-rules.md")).unwrap()
    );
    for agent in &agents {
        let source = fs::read(manifest().join("agents").join(agent)).unwrap();
        assert_eq!(fs::read(agents_dir.join(agent)).unwrap(), source, "{agent}");
    }
    // Keeps the shebang check above from passing vacuously if no installed file is a script.
    assert!(mode(&skills_dir.join("nabu-verify/scripts/diff-scan")) & 0o111 != 0);
}

fn init_repo() -> Sandbox {
    let sandbox = Sandbox::empty();
    fs::create_dir(sandbox.repo.join(".jj")).unwrap();
    sandbox
}

#[test]
fn init_global_installs_everything_from_a_folder_without_jj() {
    let sandbox = Sandbox::empty();
    let out = sandbox.nabu(&["init"]);
    assert_init_ok(&out);
    assert_install_lines(
        &out,
        "[TMP]/home/.agents/skills",
        "[TMP]/home/.claude/agents",
        "config: [TMP]/xdg/nabu/config.toml written",
    );
    assert_installed(
        &sandbox.home.join(".agents/skills"),
        &sandbox.home.join(".claude/agents"),
    );
    // A missing `~/.claude/skills` becomes a link to the installed skills.
    let link = sandbox.home.join(".claude/skills");
    assert_eq!(
        fs::read_link(&link).unwrap(),
        sandbox.home.join(".agents/skills")
    );
    // Writes land only in the home folders and the config file, never in the current folder.
    assert_eq!(entries(&sandbox.home), [".agents", ".claude"]);
    assert_eq!(entries(&sandbox.home.join(".agents")), ["skills"]);
    assert_eq!(entries(&sandbox.home.join(".claude")), ["agents", "skills"]);
    assert_eq!(files_under(&sandbox.xdg), ["nabu/config.toml"]);
    assert!(
        is_empty_dir(&sandbox.repo),
        "init touched the current folder"
    );
    // `--global` is the default spelled out.
    assert_init_ok(&sandbox.nabu(&["init", "--global"]));
    // The starter config is valid nabu config.
    fs::create_dir(sandbox.repo.join(".jj")).unwrap();
    let list = sandbox.nabu(&["list"]);
    assert_eq!(list.status, 0, "{}", list.stderr);
}

#[test]
fn init_global_keeps_an_existing_config_even_when_malformed() {
    for text in ["schema = \"specless\"\n# mine\n", "schema = \n"] {
        let sandbox = Sandbox::empty();
        sandbox.write_global("config.toml", text);
        let out = sandbox.nabu(&["init"]);
        assert_init_ok(&out);
        assert_eq!(
            out.stdout.lines().last(),
            Some("config: [TMP]/xdg/nabu/config.toml kept")
        );
        let kept = fs::read_to_string(sandbox.xdg.join("nabu/config.toml")).unwrap();
        assert_eq!(kept, text);
    }
}

#[test]
fn init_global_replaces_ours_and_spares_foreign_skills() {
    let sandbox = Sandbox::empty();
    let skills = sandbox.home.join(".agents/skills");
    fs::create_dir_all(skills.join("nabu-build")).unwrap();
    fs::write(skills.join("nabu-build/extra.md"), "stale\n").unwrap();
    let outside = sandbox.base.join("wiki/nabu-tweak");
    fs::create_dir_all(&outside).unwrap();
    fs::write(outside.join("SKILL.md"), "wiki copy\n").unwrap();
    std::os::unix::fs::symlink(&outside, skills.join("nabu-tweak")).unwrap();
    fs::create_dir_all(skills.join("foreign")).unwrap();
    fs::write(skills.join("foreign/SKILL.md"), "mine\n").unwrap();

    assert_init_ok(&sandbox.nabu(&["init"]));

    assert!(!skills.join("nabu-build/extra.md").exists());
    assert!(
        !fs::symlink_metadata(skills.join("nabu-tweak"))
            .unwrap()
            .is_symlink()
    );
    assert_eq!(
        fs::read_to_string(outside.join("SKILL.md")).unwrap(),
        "wiki copy\n"
    );
    assert_eq!(
        fs::read_to_string(skills.join("foreign/SKILL.md")).unwrap(),
        "mine\n"
    );
    assert_installed(&skills, &sandbox.home.join(".claude/agents"));
}

#[test]
fn init_global_replaces_an_agent_that_is_a_link_without_writing_through_it() {
    let sandbox = Sandbox::empty();
    let agents = sandbox.home.join(".claude/agents");
    fs::create_dir_all(&agents).unwrap();
    let source = sandbox.base.join("wiki-implementer.md");
    fs::write(&source, "wiki copy\n").unwrap();
    std::os::unix::fs::symlink(&source, agents.join("implementer.md")).unwrap();
    assert_init_ok(&sandbox.nabu(&["init"]));
    assert_eq!(fs::read_to_string(&source).unwrap(), "wiki copy\n");
    assert_installed(&sandbox.home.join(".agents/skills"), &agents);
}

#[test]
fn init_global_leaves_the_claude_skills_entry_alone() {
    // Already a link to the installed skills: left, no notice.
    let sandbox = Sandbox::empty();
    fs::create_dir_all(sandbox.home.join(".agents")).unwrap();
    fs::create_dir_all(sandbox.home.join(".claude")).unwrap();
    let link = sandbox.home.join(".claude/skills");
    std::os::unix::fs::symlink("../.agents/skills", &link).unwrap();
    let out = sandbox.nabu(&["init"]);
    assert_init_ok(&out);
    assert!(!out.stdout.contains("notice"), "{}", out.stdout);
    assert_eq!(
        fs::read_link(&link).unwrap(),
        Path::new("../.agents/skills")
    );

    // A dangling link counts as present: left, with a notice, no new link.
    let sandbox = Sandbox::empty();
    fs::create_dir_all(sandbox.home.join(".claude")).unwrap();
    let dangling = sandbox.home.join(".claude/skills");
    std::os::unix::fs::symlink("../gone", &dangling).unwrap();
    let out = sandbox.nabu(&["init"]);
    assert_init_ok(&out);
    assert_eq!(fs::read_link(&dangling).unwrap(), Path::new("../gone"));
    let lines: Vec<&str> = out.stdout.lines().collect();
    assert_eq!(
        lines[lines.len() - 2],
        "notice: [TMP]/home/.claude/skills is not a link to [TMP]/home/.agents/skills, left as is"
    );

    // A real folder: left with its content, one notice before the config line.
    let sandbox = Sandbox::empty();
    let real = sandbox.home.join(".claude/skills");
    fs::create_dir_all(real.join("mine")).unwrap();
    let out = sandbox.nabu(&["init"]);
    assert_init_ok(&out);
    assert!(real.join("mine").is_dir());
    assert!(!fs::symlink_metadata(&real).unwrap().is_symlink());
    let lines: Vec<&str> = out.stdout.lines().collect();
    assert_eq!(
        lines[lines.len() - 2],
        "notice: [TMP]/home/.claude/skills is not a link to [TMP]/home/.agents/skills, left as is"
    );
}

#[test]
fn init_global_stops_on_a_read_only_skills_folder() {
    let sandbox = Sandbox::empty();
    let skills = sandbox.home.join(".agents/skills");
    fs::create_dir_all(&skills).unwrap();
    fs::set_permissions(&skills, fs::Permissions::from_mode(0o555)).unwrap();
    let writable_anyway = fs::write(skills.join("probe"), "").is_ok();
    let out = sandbox.nabu(&["init"]);
    // Restored first: the temp folder cannot be cleaned up while it is read-only.
    fs::set_permissions(&skills, fs::Permissions::from_mode(0o755)).unwrap();
    if writable_anyway {
        return; // Running as root: permissions do not apply.
    }
    assert_eq!(out.status, 1);
    assert_eq!(out.stdout, "");
    assert!(
        out.stderr
            .starts_with("error: cannot write [TMP]/home/.agents/skills/"),
        "{}",
        out.stderr
    );
    assert!(
        out.stderr.contains("\nfix: rerun nabu init"),
        "{}",
        out.stderr
    );
}

#[test]
fn init_local_installs_into_the_repo_only() {
    let sandbox = init_repo();
    let out = sandbox.nabu(&["init", "--local"]);
    assert_init_ok(&out);
    assert_install_lines(
        &out,
        ".agents/skills",
        ".claude/agents",
        "config: nabu/config.toml written",
    );
    assert_installed(
        &sandbox.repo.join(".agents/skills"),
        &sandbox.repo.join(".claude/agents"),
    );
    assert_eq!(
        fs::read_link(sandbox.repo.join(".claude/skills")).unwrap(),
        Path::new("../.agents/skills")
    );
    // The link resolves.
    assert!(
        sandbox
            .repo
            .join(".claude/skills/nabu-build/SKILL.md")
            .is_file()
    );
    let list = sandbox.nabu(&["list"]);
    assert_eq!(list.status, 0, "{}", list.stderr);
    assert!(is_empty_dir(&sandbox.home), "init --local touched HOME");
    assert!(
        is_empty_dir(&sandbox.xdg),
        "init --local touched XDG_CONFIG_HOME"
    );
}

#[test]
fn init_local_rerun_replaces_the_installed_skills_from_a_subfolder() {
    let sandbox = init_repo();
    assert_init_ok(&sandbox.nabu(&["init", "--local"]));
    let skill = sandbox.repo.join(".agents/skills/nabu-build");
    fs::write(skill.join("SKILL.md"), "edited\n").unwrap();
    fs::write(skill.join("extra.md"), "stale\n").unwrap();
    let sub = sandbox.repo.join("src/deep");
    fs::create_dir_all(&sub).unwrap();
    let out = sandbox.nabu_in(&sub, &["init", "--local"]);
    assert_init_ok(&out);
    // Root-relative from a subfolder, and the link made by the first run raises no notice.
    assert_install_lines(
        &out,
        ".agents/skills",
        ".claude/agents",
        "config: nabu/config.toml kept",
    );
    assert_installed(
        &sandbox.repo.join(".agents/skills"),
        &sandbox.repo.join(".claude/agents"),
    );
    assert!(!sub.join(".agents").exists());
}

#[test]
fn init_local_leaves_a_foreign_claude_skills_link_with_a_notice() {
    let sandbox = init_repo();
    fs::create_dir_all(sandbox.repo.join("other")).unwrap();
    fs::create_dir_all(sandbox.repo.join(".claude")).unwrap();
    let link = sandbox.repo.join(".claude/skills");
    std::os::unix::fs::symlink("../other", &link).unwrap();
    let out = sandbox.nabu(&["init", "--local"]);
    assert_init_ok(&out);
    assert_eq!(fs::read_link(&link).unwrap(), Path::new("../other"));
    let lines: Vec<&str> = out.stdout.lines().collect();
    assert_eq!(
        lines[lines.len() - 2],
        "notice: .claude/skills is not a link to .agents/skills, left as is"
    );
}

#[test]
fn init_local_keeps_the_repo_config_even_when_malformed() {
    for text in ["context = \"mine\"\n", "schema = \n"] {
        let sandbox = init_repo();
        sandbox.write("nabu/config.toml", text);
        let out = sandbox.nabu(&["init", "--local"]);
        assert_init_ok(&out);
        assert_eq!(
            out.stdout.lines().last(),
            Some("config: nabu/config.toml kept")
        );
        let kept = fs::read_to_string(sandbox.repo.join("nabu/config.toml")).unwrap();
        assert_eq!(kept, text);
    }
}

#[test]
fn init_local_starter_config_leaves_the_global_layer_in_charge() {
    let sandbox = init_repo();
    sandbox.write_global("config.toml", "schema = \"other\"\n");
    sandbox.write_other_schema();
    assert_init_ok(&sandbox.nabu(&["init", "--local"]));
    let out = sandbox.nabu(&["new", "demo"]);
    assert_eq!(out.status, 0, "{}", out.stderr);
    let file = fs::read_to_string(sandbox.repo.join("nabu/tablets/demo/.nabu.toml")).unwrap();
    assert_eq!(file, "schema = \"other\"\n");
}

/// Snapshot of a refusal that leaves the repo and the home folder as they were.
fn assert_init_refused(sandbox: &Sandbox, args: &[&str]) -> String {
    let before = (
        sandbox.tree(),
        folder_tree(&sandbox.home),
        folder_tree(&sandbox.xdg),
    );
    let out = sandbox.nabu(args);
    assert_eq!(out.status, 1, "{}", out.stderr);
    let after = (
        sandbox.tree(),
        folder_tree(&sandbox.home),
        folder_tree(&sandbox.xdg),
    );
    assert_eq!(after, before, "refusal changed the tree");
    show(&out)
}

#[test]
fn init_local_refuses_outside_any_root() {
    let sandbox = Sandbox::empty();
    insta::assert_snapshot!(assert_init_refused(&sandbox, &["init", "--local"]));
}

#[test]
fn init_local_refuses_a_symlinked_skills_folder() {
    // The layout of this repo: `.agents/skills` links to the sources.
    let sandbox = init_repo();
    fs::create_dir_all(sandbox.repo.join("skills")).unwrap();
    fs::create_dir_all(sandbox.repo.join(".agents")).unwrap();
    std::os::unix::fs::symlink("../skills", sandbox.repo.join(".agents/skills")).unwrap();
    insta::assert_snapshot!(assert_init_refused(&sandbox, &["init", "--local"]));
}

#[test]
fn init_local_refuses_a_symlinked_agents_folder() {
    let sandbox = init_repo();
    fs::create_dir_all(sandbox.repo.join("agents")).unwrap();
    fs::create_dir_all(sandbox.repo.join(".claude")).unwrap();
    std::os::unix::fs::symlink("../agents", sandbox.repo.join(".claude/agents")).unwrap();
    insta::assert_snapshot!(assert_init_refused(&sandbox, &["init", "--local"]));
}

#[test]
fn init_local_refuses_a_symlinked_parent_folder() {
    // A linked `.claude` would send the agents outside the repo.
    let sandbox = init_repo();
    fs::create_dir_all(sandbox.base.join("elsewhere")).unwrap();
    std::os::unix::fs::symlink(sandbox.base.join("elsewhere"), sandbox.repo.join(".claude"))
        .unwrap();
    insta::assert_snapshot!(assert_init_refused(&sandbox, &["init", "--local"]));
    assert!(is_empty_dir(&sandbox.base.join("elsewhere")));
}

#[test]
fn init_global_installs_through_a_linked_dotfiles_parent() {
    let sandbox = Sandbox::empty();
    let dotfiles = sandbox.base.join("dotfiles-claude");
    fs::create_dir_all(&dotfiles).unwrap();
    std::os::unix::fs::symlink(&dotfiles, sandbox.home.join(".claude")).unwrap();
    assert_init_ok(&sandbox.nabu(&["init"]));
    assert!(dotfiles.join("agents/implementer.md").is_file());
}

#[test]
fn init_global_works_from_a_deleted_folder() {
    let sandbox = Sandbox::empty();
    let out = Command::new("sh")
        .args([
            "-c",
            "mkdir gone && cd gone && rmdir ../gone && exec \"$0\" init",
        ])
        .arg(env!("CARGO_BIN_EXE_nabu"))
        .current_dir(&sandbox.repo)
        .env_clear()
        .env("HOME", &sandbox.home)
        .env("XDG_CONFIG_HOME", &sandbox.xdg)
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        sandbox
            .home
            .join(".agents/skills/nabu-build/SKILL.md")
            .is_file()
    );
}

#[test]
fn init_global_refuses_a_symlinked_skills_folder() {
    let sandbox = Sandbox::empty();
    fs::create_dir_all(sandbox.base.join("wiki-skills")).unwrap();
    fs::create_dir_all(sandbox.home.join(".agents")).unwrap();
    std::os::unix::fs::symlink(
        sandbox.base.join("wiki-skills"),
        sandbox.home.join(".agents/skills"),
    )
    .unwrap();
    insta::assert_snapshot!(assert_init_refused(&sandbox, &["init"]));
}

#[test]
fn init_global_refuses_a_symlinked_agents_folder() {
    let sandbox = Sandbox::empty();
    fs::create_dir_all(sandbox.base.join("wiki-agents")).unwrap();
    fs::create_dir_all(sandbox.home.join(".claude")).unwrap();
    std::os::unix::fs::symlink(
        sandbox.base.join("wiki-agents"),
        sandbox.home.join(".claude/agents"),
    )
    .unwrap();
    insta::assert_snapshot!(assert_init_refused(&sandbox, &["init"]));
}

#[test]
fn init_global_refuses_without_home() {
    let sandbox = Sandbox::empty();
    let out = Command::new(env!("CARGO_BIN_EXE_nabu"))
        .args(["init"])
        .current_dir(&sandbox.repo)
        .env_clear()
        .env("XDG_CONFIG_HOME", &sandbox.xdg)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    assert!(is_empty_dir(&sandbox.xdg), "wrote without HOME");
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    insta::assert_snapshot!(stderr);

    // An empty HOME is unset too: joined paths would be relative and land in the current folder.
    let out = sandbox.nabu_env(&["init"], &[("HOME", "")]);
    assert_eq!(out.status, 1);
    assert_eq!(out.stdout, "");
    assert_eq!(out.stderr, stderr);
    assert!(is_empty_dir(&sandbox.repo), "wrote into the current folder");
    assert!(is_empty_dir(&sandbox.xdg), "wrote without HOME");
}

#[test]
fn init_global_and_local_conflict() {
    let sandbox = init_repo();
    let out = sandbox.nabu(&["init", "--global", "--local"]);
    assert_eq!(out.status, 2);
    assert!(out.stdout.is_empty());
    assert!(is_empty_dir(&sandbox.home));
    insta::assert_snapshot!(show(&out));
}
