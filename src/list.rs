use crate::Outcome;
use crate::error::Result;
use crate::layers::Layers;
use crate::report::{self, Layout};
use crate::table::{self, Row};
use crate::tablets::list_tablets;
use std::path::Path;

/// A broken tablet becomes its own `error` row, so one bad tablet never hides the others.
pub fn run(root: &Path, layers: &Layers) -> Result<Outcome> {
    let names = list_tablets(root)?;
    if names.is_empty() {
        return Ok(Outcome::ok("no active tablets".to_string()));
    }
    let results: Vec<_> = names
        .into_iter()
        .map(|name| {
            let row = row(root, layers, &name);
            (name, row)
        })
        .collect();
    let failed = results.iter().any(|(_, row)| row.is_err());
    let rows: Vec<Row> = results
        .into_iter()
        .map(|(name, row)| match row {
            Ok(cells) => Row::Cells(cells),
            Err(err) => Row::Tail {
                first: name,
                rest: format!("error  {}", err.summary()),
            },
        })
        .collect();
    Ok(Outcome {
        text: table::render(&rows, ""),
        failed,
    })
}

fn row(root: &Path, layers: &Layers, name: &str) -> Result<Vec<String>> {
    let tablet = report::read(&report::load(root, layers, name)?)?;
    let slices = match &tablet.layout {
        Layout::Plain(_) => String::new(),
        Layout::Stacked { slices, .. } => format!("{} slices", slices.len()),
    };
    let progress = tablet.progress();
    let mark = if tablet.ready_to_archive() {
        "ready to archive"
    } else {
        ""
    };
    Ok(vec![
        tablet.name,
        format!("{}/{} tasks", progress.checked, progress.total),
        slices,
        mark.to_string(),
    ])
}
