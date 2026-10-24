//! `nabu instructions`: the prompt text skills feed to the model.
//! Only layout lives here. Templates, instructions, context, and rules come from the layers,
//! and states come from `report`, so no skill keeps its own copy.

use crate::config::Config;
use crate::error::{Error, Result};
use crate::layers::Layers;
use crate::prompt::{block, bullets, join_context, render};
use crate::report::{self, ArtifactState, FolderReport, Layout, LoadedTablet, TabletReport};
use crate::schema::Schema;
use crate::tablets::{
    find_slice, list_slices, listing, require_stack_file, split_target, stack_file,
};
use std::collections::HashSet;
use std::path::Path;

/// `what` is `build`, `stack`, or an artifact id. These two words shadow artifacts of that name.
pub fn run(
    root: &Path,
    layers: &Layers,
    config: &Config,
    what: &str,
    target: &str,
) -> Result<String> {
    match what {
        "build" => build(root, layers, config, target),
        "stack" => stack(root, layers, config, target),
        id => artifact(root, layers, config, id, target),
    }
}

/// The folder an artifact prompt is about: a plain tablet or one slice.
struct Folder {
    /// Root-relative path.
    shown: String,
    /// `<tablet>` or `<tablet>/<N>-<slice>`.
    label: String,
    slice_folder: Option<String>,
    /// Root-relative path of the stack file, for slices.
    stack_path: Option<String>,
}

/// `invocation` is what follows `nabu instructions` in the fix of a stacked-root refusal.
fn locate(loaded: &LoadedTablet, slice_key: Option<&str>, invocation: &str) -> Result<Folder> {
    let LoadedTablet {
        tablet, resolved, ..
    } = loaded;
    let shown = tablet.shown();
    let Some(key) = slice_key else {
        if stack_file(tablet, resolved)?.is_some() {
            let slices = list_slices(tablet)?;
            let fix = match slices.first() {
                Some(first) => format!(
                    "nabu instructions {invocation} {}/{}",
                    tablet.name, first.number
                ),
                None => format!("nabu new {}/<slice>", tablet.name),
            };
            return Err(Error::new(
                format!(
                    "tablet \"{}\" is stacked: its artifacts live in slices; slices: {}",
                    tablet.name,
                    listing(slices.iter().map(|s| s.folder.as_str()))
                ),
                fix,
            ));
        }
        return Ok(Folder {
            shown,
            label: tablet.name.clone(),
            slice_folder: None,
            stack_path: None,
        });
    };
    let stack = require_stack_file(tablet, resolved)?;
    let slices = list_slices(tablet)?;
    let slice = find_slice(tablet, &slices, key)?;
    Ok(Folder {
        label: format!("{}/{}", tablet.name, slice.folder),
        slice_folder: Some(slice.folder.clone()),
        stack_path: Some(format!("{shown}/{stack}")),
        shown: format!("{shown}/{}", slice.folder),
    })
}

/// The report of the folder `locate` chose: the plain tablet or the one slice.
fn folder_report(report: TabletReport, folder: &Folder) -> FolderReport {
    match report.layout {
        Layout::Plain(plain) => plain,
        Layout::Stacked { slices, .. } => slices
            .into_iter()
            .find(|slice| Some(&slice.folder) == folder.slice_folder.as_ref())
            .map(|slice| slice.report)
            .expect("`locate` found the slice in the same listing"),
    }
}

/// Done by the same rule `status` uses, so a prompt never disagrees with it.
fn is_done(report: &FolderReport, id: &str) -> bool {
    *state_of(report, id) == ArtifactState::Done
}

/// Ids come from the checked schema graph, and a report holds a status per schema artifact.
fn state_of<'a>(report: &'a FolderReport, id: &str) -> &'a ArtifactState {
    &report
        .artifacts
        .iter()
        .find(|status| status.id == id)
        .expect("every schema artifact has a status")
        .state
}

/// `list` for header values: comma-separated, `none` when empty.
fn list(items: &[String]) -> String {
    listing(items.iter().map(String::as_str))
}

/// `<id> (done|missing), ...`
fn requires_line(ids: &[String], report: &FolderReport) -> String {
    let items: Vec<String> = ids
        .iter()
        .map(|id| {
            let word = if is_done(report, id) {
                "done"
            } else {
                "missing"
            };
            format!("{id} ({word})")
        })
        .collect();
    list(&items)
}

/// The done artifacts in the transitive `requires` closure of `ids`, in schema order, plus the
/// stack file for a slice. The backlog stays out unless the schema requires it =>
/// deferred work never widens scope.
fn read_paths(schema: &Schema, report: &FolderReport, ids: &[String], folder: &Folder) -> String {
    let mut wanted: HashSet<&str> = HashSet::new();
    let mut todo: Vec<&str> = ids.iter().map(String::as_str).collect();
    while let Some(id) = todo.pop() {
        if !wanted.insert(id) {
            continue;
        }
        if let Some(artifact) = schema.artifacts.iter().find(|a| a.id == id) {
            todo.extend(artifact.requires.iter().map(String::as_str));
        }
    }
    let mut paths: Vec<String> = schema
        .artifacts
        .iter()
        .filter(|artifact| wanted.contains(artifact.id.as_str()) && is_done(report, &artifact.id))
        .map(|artifact| format!("{}/{}", folder.shown, artifact.generates))
        .collect();
    paths.extend(folder.stack_path.clone());
    list(&paths)
}

fn state_word(state: &ArtifactState) -> &'static str {
    match state {
        ArtifactState::Done => "done",
        ArtifactState::Ready => "ready",
        ArtifactState::Blocked(_) => "blocked",
    }
}

fn artifact(
    root: &Path,
    layers: &Layers,
    config: &Config,
    id: &str,
    target: &str,
) -> Result<String> {
    let (tablet_name, slice_key) = split_target(target, &format!("instructions {id}"))?;
    let loaded = report::load(root, layers, tablet_name)?;
    config.check_rules_keys(&loaded.schema_name, &loaded.resolved)?;
    let schema = &loaded.resolved.schema;
    let Some(artifact) = schema.artifacts.iter().find(|a| a.id == id) else {
        let ids: Vec<&str> = schema.artifacts.iter().map(|a| a.id.as_str()).collect();
        return Err(Error::new(
            format!(
                "unknown artifact \"{id}\"; artifacts: {}, or the words build, stack",
                ids.join(", ")
            ),
            format!(
                "nabu instructions {} {target}",
                ids.first().unwrap_or(&"<artifact>")
            ),
        ));
    };
    let folder = locate(&loaded, slice_key, id)?;
    let report = folder_report(report::read(&loaded)?, &folder);
    let state = state_word(state_of(&report, id));
    let header = [
        ("artifact", id.to_string()),
        ("target", folder.label.clone()),
        ("write", format!("{}/{}", folder.shown, artifact.generates)),
        ("state", state.to_string()),
        ("requires", requires_line(&artifact.requires, &report)),
        (
            "read",
            read_paths(schema, &report, &artifact.requires, &folder),
        ),
    ];
    let rules = config.rules.get(id).map(|rules| bullets(rules));
    Ok(render(
        &header,
        &[
            block(
                "description",
                artifact.description.as_deref().unwrap_or_default(),
            ),
            block(
                "instruction",
                artifact.instruction.as_deref().unwrap_or_default(),
            ),
            block("context", &join_context(&config.context)),
            block("rules", &rules.unwrap_or_default()),
            block("template", &loaded.resolved.templates[id]),
        ],
    ))
}

fn build(root: &Path, layers: &Layers, config: &Config, target: &str) -> Result<String> {
    let (tablet_name, slice_key) = split_target(target, "instructions build")?;
    let loaded = report::load(root, layers, tablet_name)?;
    config.check_rules_keys(&loaded.schema_name, &loaded.resolved)?;
    let schema = &loaded.resolved.schema;
    let folder = locate(&loaded, slice_key, "build")?;
    let report = folder_report(report::read(&loaded)?, &folder);
    let requires = &schema.build.requires;
    let progress = report.progress;
    let state = if !requires.iter().all(|id| is_done(&report, id)) {
        "blocked"
    } else if progress.is_complete() {
        "all done"
    } else {
        "ready"
    };
    let header = [
        ("build", folder.label.clone()),
        ("state", state.to_string()),
        (
            "progress",
            format!("{}/{} tasks", progress.checked, progress.total),
        ),
        ("requires", requires_line(requires, &report)),
        (
            "tracks",
            format!("{}/{}", folder.shown, schema.build.tracks),
        ),
        ("read", read_paths(schema, &report, requires, &folder)),
    ];
    Ok(render(
        &header,
        &[
            block(
                "instruction",
                schema.build.instruction.as_deref().unwrap_or_default(),
            ),
            block("context", &join_context(&config.context)),
            block("pending", &report.pending.join("\n")),
        ],
    ))
}

fn stack(root: &Path, layers: &Layers, config: &Config, target: &str) -> Result<String> {
    let (tablet_name, slice_key) = split_target(target, "instructions stack")?;
    if slice_key.is_some() {
        return Err(Error::new(
            format!("\"{target}\" is a slice: the stack file belongs to the tablet"),
            format!("nabu instructions stack {tablet_name}"),
        ));
    }
    let loaded = report::load(root, layers, tablet_name)?;
    config.check_rules_keys(&loaded.schema_name, &loaded.resolved)?;
    let LoadedTablet {
        tablet, resolved, ..
    } = &loaded;
    let Some(template) = &resolved.stack else {
        return Err(Error::new(
            format!(
                "{}: schema has no `stack` key, so it has no stack template",
                resolved.path
            ),
            format!("add stack = \"stack.md\" to {}", resolved.path),
        ));
    };
    let shown = format!("{}/{}", tablet.shown(), template.file);
    let done = report::stack_done(tablet, resolved)?;
    let header = [
        ("stack", tablet.name.clone()),
        ("write", shown),
        ("state", if done { "done" } else { "ready" }.to_string()),
    ];
    Ok(render(
        &header,
        &[
            block("context", &join_context(&config.context)),
            block("template", &template.text),
        ],
    ))
}
