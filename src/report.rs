//! Artifact state and task progress of a tablet. `status` and `list` only render these values.

use crate::error::Result;
use crate::layers::Layers;
use crate::layers::read_file;
use crate::schema::{self, Artifact, ResolvedSchema};
use crate::tablets::{Slice, Tablet, find_tablet, list_slices, read_tablet_file, stack_file};
use std::collections::HashSet;
use std::path::Path;

#[derive(Debug, PartialEq)]
pub enum ArtifactState {
    Done,
    Ready,
    /// The unfinished `requires` ids, in the artifact's `requires` order.
    Blocked(Vec<String>),
}

#[derive(Debug, PartialEq)]
pub struct ArtifactStatus {
    pub id: String,
    pub state: ArtifactState,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Progress {
    pub checked: usize,
    pub total: usize,
}

impl Progress {
    /// Zero tasks is never complete => an unwritten plan is not ready to archive.
    pub fn is_complete(self) -> bool {
        self.total > 0 && self.checked == self.total
    }

    fn add(self, other: Progress) -> Progress {
        Progress {
            checked: self.checked + other.checked,
            total: self.total + other.total,
        }
    }
}

/// One folder that holds artifacts: a plain tablet or one slice.
#[derive(Debug)]
pub struct FolderReport {
    pub artifacts: Vec<ArtifactStatus>,
    pub progress: Progress,
    /// Unchecked task lines of the tracks file, verbatim. Empty unless the tracks file is done,
    /// matching progress `0/0`.
    pub pending: Vec<String>,
}

#[derive(Debug)]
pub struct SliceReport {
    pub folder: String,
    pub report: FolderReport,
}

#[derive(Debug)]
pub enum Layout {
    Plain(FolderReport),
    /// A stacked tablet ignores artifact files at its own root: only slices carry artifacts.
    Stacked {
        stack_file: String,
        stack_done: bool,
        slices: Vec<SliceReport>,
    },
}

#[derive(Debug)]
pub struct TabletReport {
    pub name: String,
    pub schema: String,
    pub layout: Layout,
}

impl TabletReport {
    pub fn progress(&self) -> Progress {
        match &self.layout {
            Layout::Plain(folder) => folder.progress,
            Layout::Stacked { slices, .. } => {
                slices.iter().fold(Progress::default(), |sum, slice| {
                    sum.add(slice.report.progress)
                })
            }
        }
    }

    pub fn ready_to_archive(&self) -> bool {
        match &self.layout {
            Layout::Plain(folder) => folder.progress.is_complete(),
            Layout::Stacked { slices, .. } => {
                !slices.is_empty() && slices.iter().all(|s| s.report.progress.is_complete())
            }
        }
    }
}

/// A tablet with its schema resolved and graph-checked, before any artifact is read.
/// `status` looks up the requested slice between this step and `read`, so a typo fails fast.
pub struct LoadedTablet {
    pub tablet: Tablet,
    pub schema_name: String,
    pub resolved: ResolvedSchema,
}

/// Fails on a bad `.nabu.toml`, schema, or graph. Reads no artifact file.
pub fn load(root: &Path, layers: &Layers, name: &str) -> Result<LoadedTablet> {
    let tablet = find_tablet(root, name)?;
    let tablet_file = read_tablet_file(&tablet)?;
    let resolved = schema::resolve(
        layers,
        &tablet_file.schema,
        &format!("{}/.nabu.toml", tablet.shown()),
    )?;
    Ok(LoadedTablet {
        tablet,
        schema_name: tablet_file.schema,
        resolved,
    })
}

/// Reads every artifact and tracks file of the tablet. Any read failure other than "not found" is an error.
pub fn read(loaded: &LoadedTablet) -> Result<TabletReport> {
    let LoadedTablet {
        tablet,
        schema_name,
        resolved,
    } = loaded;
    let layout = match stack_file(tablet, resolved)? {
        None => Layout::Plain(read_folder(&tablet.dir, &tablet.shown(), resolved)?),
        Some(stack_file) => {
            let slices = list_slices(tablet)?
                .into_iter()
                .map(|Slice { folder, .. }| {
                    let report = read_folder(
                        &tablet.dir.join(&folder),
                        &format!("{}/{folder}", tablet.shown()),
                        resolved,
                    )?;
                    Ok(SliceReport { folder, report })
                })
                .collect::<Result<_>>()?;
            Layout::Stacked {
                stack_file: stack_file.to_string(),
                stack_done: stack_done(tablet, resolved)?,
                slices,
            }
        }
    };
    Ok(TabletReport {
        name: tablet.name.clone(),
        schema: schema_name.clone(),
        layout,
    })
}

/// The stack file is done by the same rule as an artifact. `false` when the tablet has none.
/// One owner for `status` and `instructions stack`, so they never disagree.
pub fn stack_done(tablet: &Tablet, resolved: &ResolvedSchema) -> Result<bool> {
    let Some(file) = stack_file(tablet, resolved)? else {
        return Ok(false);
    };
    let template = &resolved
        .stack
        .as_ref()
        .expect("a stack file name implies a stack template")
        .text;
    let text = read_file(
        &tablet.dir.join(file),
        &format!("{}/{file}", tablet.shown()),
    )?;
    Ok(is_done(text.as_deref(), template))
}

/// One artifact's file as found on disk.
struct ArtifactFile<'a> {
    artifact: &'a Artifact,
    text: Option<String>,
    done: bool,
}

/// `shown` is the folder as messages print it.
fn read_folder(dir: &Path, shown: &str, resolved: &ResolvedSchema) -> Result<FolderReport> {
    let files = resolved
        .schema
        .artifacts
        .iter()
        .map(|artifact| {
            let text = read_file(
                &dir.join(&artifact.generates),
                &format!("{shown}/{}", artifact.generates),
            )?;
            let done = is_done(text.as_deref(), &resolved.templates[&artifact.id]);
            Ok(ArtifactFile {
                artifact,
                text,
                done,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let done_ids: HashSet<&str> = files
        .iter()
        .filter(|file| file.done)
        .map(|file| file.artifact.id.as_str())
        .collect();
    let statuses = files
        .iter()
        .map(|file| {
            let state = if file.done {
                ArtifactState::Done
            } else {
                let needs: Vec<String> = file
                    .artifact
                    .requires
                    .iter()
                    .filter(|id| !done_ids.contains(id.as_str()))
                    .cloned()
                    .collect();
                if needs.is_empty() {
                    ArtifactState::Ready
                } else {
                    ArtifactState::Blocked(needs)
                }
            };
            ArtifactStatus {
                id: file.artifact.id.clone(),
                state,
            }
        })
        .collect();
    // Template placeholder tasks never count => only a done artifact contributes.
    // The graph check guarantees an artifact generates the tracked file.
    let (progress, pending) = files
        .iter()
        .find(|file| file.artifact.generates == resolved.schema.build.tracks)
        .filter(|file| file.done)
        .and_then(|file| file.text.as_deref())
        .map_or((Progress::default(), Vec::new()), scan_tasks);
    Ok(FolderReport {
        artifacts: statuses,
        progress,
        pending,
    })
}

/// An agent that only touches whitespace has not done the artifact.
fn normalize(text: &str) -> String {
    text.replace("\r\n", "\n")
        .replace('\r', "\n")
        .trim()
        .to_string()
}

fn is_done(text: Option<&str>, template: &str) -> bool {
    text.map(normalize)
        .is_some_and(|text| !text.is_empty() && text != normalize(template))
}

/// A task is a list item (`-`, `*`, `+`) with a `[ ]`, `[x]` or `[X]` box, at any indent.
/// One pass yields the counts and the unchecked lines, so `build` cannot disagree with `status`.
fn scan_tasks(text: &str) -> (Progress, Vec<String>) {
    let mut progress = Progress::default();
    let mut pending = Vec::new();
    for line in text.lines() {
        let Some(rest) = line.trim_start().strip_prefix(['-', '*', '+']) else {
            continue;
        };
        let Some(rest) = rest.strip_prefix(' ') else {
            continue;
        };
        match rest.get(..3) {
            Some("[ ]") => {
                progress.total += 1;
                pending.push(line.to_string());
            }
            Some("[x]" | "[X]") => {
                progress.total += 1;
                progress.checked += 1;
            }
            _ => {}
        }
    }
    (progress, pending)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    const SCHEMA: &str = r#"name = "s"
[[artifacts]]
id = "one"
generates = "one.md"
template = "one.md"
[[artifacts]]
id = "two"
generates = "two.md"
template = "two.md"
[[artifacts]]
id = "three"
generates = "three.md"
template = "three.md"
requires = ["two", "one", "three-b"]
[[artifacts]]
id = "three-b"
generates = "three-b.md"
template = "three-b.md"
requires = ["one"]
[build]
requires = []
tracks = "two.md"
"#;

    /// A folder of artifact files over schema `s`, whose templates all read "template".
    fn folder(files: &[(&str, &[u8])]) -> Result<FolderReport> {
        let layers = Layers::from_layers(vec![crate::layers::Layer::new(
            "nabu/",
            [
                ("schemas/s/schema.toml", SCHEMA),
                ("schemas/s/templates/one.md", "template\n"),
                ("schemas/s/templates/two.md", "# Tasks\n- [ ] placeholder\n"),
                ("schemas/s/templates/three.md", "template\n"),
                ("schemas/s/templates/three-b.md", "template\n"),
            ],
        )]);
        let resolved = schema::resolve(&layers, "s", "nabu/config.toml").unwrap();
        let tmp = tempfile::tempdir().unwrap();
        for (name, bytes) in files {
            fs::write(tmp.path().join(name), bytes).unwrap();
        }
        read_folder(tmp.path(), "nabu/tablets/t", &resolved)
    }

    fn states(files: &[(&str, &[u8])]) -> Vec<(String, ArtifactState)> {
        folder(files)
            .unwrap()
            .artifacts
            .into_iter()
            .map(|a| (a.id, a.state))
            .collect()
    }

    fn progress(tasks: &[u8]) -> (usize, usize) {
        let report = folder(&[("two.md", tasks)]).unwrap();
        (report.progress.checked, report.progress.total)
    }

    #[test]
    fn done_needs_content_that_differs_from_the_template() {
        let done = |bytes: &[u8]| states(&[("one.md", bytes)])[0].1 == ArtifactState::Done;
        assert!(done(b"edited"));
        assert!(!done(b""));
        assert!(!done(b" \n\t\n"));
        assert!(!done(b"template"));
        assert!(!done(b"\n  template\r\n\r\n  "));
        assert!(!done(b"template\r\n"));
        // Only the surrounding whitespace is ignored.
        assert!(done(b"temp late"));
    }

    #[test]
    fn missing_file_is_not_done() {
        assert_eq!(states(&[])[0].1, ArtifactState::Ready);
    }

    #[test]
    fn blocked_lists_only_unfinished_ids_in_requires_order() {
        let all = states(&[]);
        assert_eq!(
            all[2].1,
            ArtifactState::Blocked(vec!["two".into(), "one".into(), "three-b".into()])
        );
        let partial = states(&[("one.md", b"x")]);
        assert_eq!(
            partial[2].1,
            ArtifactState::Blocked(vec!["two".into(), "three-b".into()])
        );
        assert_eq!(partial[3].1, ArtifactState::Ready);
        let ready = states(&[("one.md", b"x"), ("two.md", b"x"), ("three-b.md", b"x")]);
        assert_eq!(ready[2].1, ArtifactState::Ready);
    }

    #[test]
    fn checkbox_forms_all_count() {
        let text = "- [ ] a\n* [x] b\n+ [X] c\n  - [ ] nested\n\t- [x] tabbed\n";
        assert_eq!(progress(text.as_bytes()), (3, 5));
    }

    #[test]
    fn lines_that_are_not_tasks_do_not_count() {
        let text =
            "[x] bare\n-[x] glued\n- [] empty\n- [y] other\n1. [x] numbered\n- [x]\n- text [x]\n";
        // Only the bare `- [x]` line without text is a task.
        assert_eq!(progress(text.as_bytes()), (1, 1));
    }

    #[test]
    fn pending_lines_are_the_unchecked_tasks_verbatim_in_file_order() {
        let text = "# T\n- [ ] a\n- [x] b\n  * [ ] nested\r\n- not a task\n";
        let report = folder(&[("two.md", text.as_bytes())]).unwrap();
        assert_eq!(report.pending, ["- [ ] a", "  * [ ] nested"]);
    }

    #[test]
    fn template_equal_tracks_file_has_no_pending_lines() {
        let report = folder(&[("two.md", b"# Tasks\n- [ ] placeholder\n")]).unwrap();
        assert!(report.pending.is_empty());
        assert!(folder(&[]).unwrap().pending.is_empty());
    }

    #[test]
    fn crlf_tasks_count() {
        assert_eq!(progress(b"- [x] a\r\n- [ ] b\r\n"), (1, 2));
    }

    #[test]
    fn missing_tracks_file_is_zero_of_zero() {
        assert_eq!(progress(b""), (0, 0));
        let report = folder(&[]).unwrap();
        assert_eq!(report.progress, Progress::default());
    }

    #[test]
    fn template_equal_tracks_file_is_zero_of_zero() {
        assert_eq!(progress(b"# Tasks\r\n- [ ] placeholder\r\n\r\n"), (0, 0));
    }

    #[test]
    fn non_utf8_tracks_file_is_an_error_naming_it() {
        let err = folder(&[("two.md", &[0xff, 0xfe, 0x00])])
            .unwrap_err()
            .to_string();
        assert!(err.contains("nabu/tablets/t/two.md"), "{err}");
    }

    #[test]
    fn folder_where_a_file_should_be_is_an_error() {
        let layers = Layers::load(Path::new("/nonexistent"), None).unwrap();
        let resolved = schema::resolve(&layers, "specless", "nabu/config.toml").unwrap();
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir(tmp.path().join("blueprint.md")).unwrap();
        let err = read_folder(tmp.path(), "nabu/tablets/t", &resolved)
            .unwrap_err()
            .to_string();
        assert!(err.contains("nabu/tablets/t/blueprint.md"), "{err}");
    }

    fn slice_with(checked: usize, total: usize) -> SliceReport {
        SliceReport {
            folder: "1-a".into(),
            report: FolderReport {
                artifacts: Vec::new(),
                progress: Progress { checked, total },
                pending: Vec::new(),
            },
        }
    }

    fn stacked(slices: Vec<SliceReport>) -> TabletReport {
        TabletReport {
            name: "t".into(),
            schema: "s".into(),
            layout: Layout::Stacked {
                stack_file: "stack.md".into(),
                stack_done: true,
                slices,
            },
        }
    }

    #[test]
    fn zero_tasks_is_never_ready_to_archive() {
        assert!(!Progress::default().is_complete());
        assert!(
            !Progress {
                checked: 0,
                total: 2
            }
            .is_complete()
        );
        assert!(
            Progress {
                checked: 2,
                total: 2
            }
            .is_complete()
        );
    }

    #[test]
    fn stacked_tablet_is_ready_only_when_every_slice_is_complete() {
        assert!(!stacked(vec![]).ready_to_archive());
        assert!(stacked(vec![slice_with(2, 2), slice_with(1, 1)]).ready_to_archive());
        assert!(!stacked(vec![slice_with(2, 2), slice_with(1, 2)]).ready_to_archive());
        assert!(!stacked(vec![slice_with(2, 2), slice_with(0, 0)]).ready_to_archive());
        let sum = stacked(vec![slice_with(2, 2), slice_with(1, 3)]).progress();
        assert_eq!((sum.checked, sum.total), (3, 5));
    }
}
