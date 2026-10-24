use crate::error::{Error, Result};
use crate::tablets::{ARCHIVE_DIR, find_tablet};
use jiff::Timestamp;
use jiff::tz::TimeZone;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

/// Returns the printed result: the new folder relative to the root.
/// Moves the whole tablet folder with one `fs::rename`, so a tablet is either fully active or fully
/// archived. Needs no schema: a tablet with a broken or unknown schema still archives.
pub fn run(root: &Path, target: &str) -> Result<String> {
    // Before lookup: `a/b` must not report "unknown tablet" for a slice path.
    if let Some((tablet, _)) = target.split_once('/') {
        return Err(Error::new(
            format!("cannot archive \"{target}\": archive moves whole tablets"),
            format!("nabu archive {tablet}"),
        ));
    }
    let tablet = find_tablet(root, target)?;
    let name = format!("{}-{}", today(), tablet.name);
    let shown = format!("{ARCHIVE_DIR}/{name}");

    // Created before the existence check on purpose: an existing target implies the archive folder
    // exists, so a refusal never changes the tree. It also makes a file named `nabu/archive` fail
    // here with a clear cause, instead of as a "not a directory" error on the target lookup.
    let archive_dir = root.join(ARCHIVE_DIR);
    fs::create_dir_all(&archive_dir).map_err(|err| {
        Error::new(
            format!("cannot create {ARCHIVE_DIR}: {err}"),
            format!("check that {ARCHIVE_DIR} is a folder you can write, then rerun"),
        )
    })?;

    let target_dir = archive_dir.join(&name);
    // Required check: `fs::rename` silently replaces an empty target folder on Linux.
    // `symlink_metadata` so a dangling symlink counts as existing too.
    match fs::symlink_metadata(&target_dir) {
        Ok(_) => {
            return Err(Error::new(
                format!("{shown} already exists"),
                format!("rename or remove {shown}, then rerun"),
            ));
        }
        Err(err) if err.kind() == ErrorKind::NotFound => {}
        Err(err) => {
            return Err(Error::new(
                format!("cannot check {shown}: {err}"),
                format!("check the permissions of {shown}"),
            ));
        }
    }

    // `nabu/` lives in one repo, so source and target share a file system and the rename is atomic.
    fs::rename(&tablet.dir, &target_dir).map_err(|err| {
        Error::new(
            format!("cannot move {} to {shown}: {err}", tablet.shown()),
            "check the folder permissions and rerun",
        )
    })?;
    Ok(shown)
}

/// ISO date in the system time zone. A UTC date would be off by one day for hours after local
/// midnight. Never blocks: an undetectable zone falls back to UTC with a warning on stderr.
fn today() -> String {
    let zone = TimeZone::try_system().unwrap_or_else(|_| {
        eprintln!("warning: system time zone unknown, using the UTC date; set TZ");
        TimeZone::UTC
    });
    Timestamp::now().to_zoned(zone).date().to_string()
}
