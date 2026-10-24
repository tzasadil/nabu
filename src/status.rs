use crate::error::Result;
use crate::layers::Layers;
use crate::report::{self, ArtifactState, FolderReport, Layout, Progress, TabletReport};
use crate::table::{self, Row};
use crate::tablets::{find_slice, list_slices, require_stack_file, split_target};
use std::path::Path;

/// `target` is `<tablet>` or `<tablet>/<slice>`. A slice target prints only that slice's block.
/// Names are not validated: the lookup errors already list what exists.
pub fn run(root: &Path, layers: &Layers, target: &str) -> Result<String> {
    let (tablet_name, slice_key) = split_target(target, "status")?;
    let loaded = report::load(root, layers, tablet_name)?;
    let only_folder = match slice_key {
        None => None,
        Some(key) => {
            require_stack_file(&loaded.tablet, &loaded.resolved)?;
            let slices = list_slices(&loaded.tablet)?;
            Some(find_slice(&loaded.tablet, &slices, key)?.folder.clone())
        }
    };
    Ok(render(&report::read(&loaded)?, only_folder.as_deref()))
}

fn progress_label(progress: Progress, ready_to_archive: bool) -> String {
    let mut label = format!("{}/{} tasks", progress.checked, progress.total);
    if ready_to_archive {
        label += "  ready to archive";
    }
    label
}

fn render(tablet: &TabletReport, only_folder: Option<&str>) -> String {
    let label = progress_label(tablet.progress(), tablet.ready_to_archive());
    match &tablet.layout {
        Layout::Plain(folder) => format!(
            "{}  {}  {label}\n{}",
            tablet.name,
            tablet.schema,
            artifact_table(folder, "")
        ),
        Layout::Stacked {
            stack_file,
            stack_done,
            slices,
        } => {
            let stack_state = if *stack_done { "done" } else { "ready" };
            let mut blocks = vec![format!(
                "{}  stacked  {}  {label}\n{stack_file}  {stack_state}",
                tablet.name, tablet.schema
            )];
            for slice in slices
                .iter()
                .filter(|slice| only_folder.is_none_or(|folder| folder == slice.folder))
            {
                let report = &slice.report;
                blocks.push(format!(
                    "{}  {}\n{}",
                    slice.folder,
                    progress_label(report.progress, report.progress.is_complete()),
                    artifact_table(report, "  ")
                ));
            }
            blocks.join("\n\n")
        }
    }
}

fn artifact_table(folder: &FolderReport, indent: &str) -> String {
    let rows: Vec<Row> = folder
        .artifacts
        .iter()
        .map(|artifact| {
            Row::Cells(match &artifact.state {
                ArtifactState::Done => vec![artifact.id.clone(), "done".into()],
                ArtifactState::Ready => vec![artifact.id.clone(), "ready".into()],
                ArtifactState::Blocked(needs) => vec![
                    artifact.id.clone(),
                    "blocked".into(),
                    format!("needs {}", needs.join(", ")),
                ],
            })
        })
        .collect();
    table::render(&rows, indent)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::ArtifactStatus;

    #[test]
    fn blocked_line_lists_every_unfinished_id_after_needs() {
        let folder = FolderReport {
            artifacts: vec![
                ArtifactStatus {
                    id: "a".into(),
                    state: ArtifactState::Done,
                },
                ArtifactStatus {
                    id: "long-id".into(),
                    state: ArtifactState::Blocked(vec!["x".into(), "y".into()]),
                },
            ],
            progress: Progress::default(),
            pending: Vec::new(),
        };
        assert_eq!(
            artifact_table(&folder, "  "),
            "  a        done\n  long-id  blocked  needs x, y"
        );
    }
}
