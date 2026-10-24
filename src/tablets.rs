use crate::error::{Error, Result};
use crate::schema::ResolvedSchema;
use crate::toml_file::parse_toml;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// `.nabu.toml` of a tablet. Slices have none and inherit the tablet's schema.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TabletFile {
    pub schema: String,
}

#[derive(Debug, Clone, Copy)]
pub enum NameKind {
    Tablet,
    Slice,
}

/// Kebab-case. A slice starts with a letter => a slice name never looks like
/// `<N>` or `<N>-<name>`, so slice lookup is never ambiguous. Tablets may start
/// with a digit because existing change names do (`42-empty-list-notice`).
pub fn validate_name(kind: NameKind, name: &str) -> Result<()> {
    if is_valid_name(kind, name) {
        return Ok(());
    }
    let (label, rule) = match kind {
        NameKind::Tablet => ("tablet", "start with a letter or digit"),
        NameKind::Slice => ("slice", "start with a letter"),
    };
    let suggestion = suggest_name(kind, name);
    Err(Error::new(
        format!(
            "invalid {label} name \"{name}\": use lowercase letters, digits and hyphens, and {rule}"
        ),
        format!("use a name like \"{suggestion}\""),
    ))
}

fn is_valid_name(kind: NameKind, name: &str) -> bool {
    let mut chars = name.chars();
    let first_ok = match (kind, chars.next()) {
        (NameKind::Tablet, Some(c)) => c.is_ascii_lowercase() || c.is_ascii_digit(),
        (NameKind::Slice, Some(c)) => c.is_ascii_lowercase(),
        (_, None) => false,
    };
    first_ok && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

fn suggest_name(kind: NameKind, name: &str) -> String {
    let words: Vec<String> = name
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_ascii_lowercase)
        .collect();
    let slug = words.join("-");
    if is_valid_name(kind, &slug) {
        slug
    } else {
        "my-name".to_string()
    }
}

/// Splits `<tablet>` or `<tablet>/<slice>`. Callers validate the names:
/// `new` needs valid ones, `status` accepts any existing folder.
/// `command` only shapes the fix, so each command suggests itself.
pub fn split_target<'a>(target: &'a str, command: &str) -> Result<(&'a str, Option<&'a str>)> {
    let parts: Vec<&str> = target.split('/').collect();
    match parts.as_slice() {
        [tablet] => Ok((tablet, None)),
        [tablet, slice] => Ok((tablet, Some(slice))),
        _ => Err(Error::new(
            format!("invalid target \"{target}\": expected <tablet> or <tablet>/<slice>"),
            format!("nabu {command} <tablet>/<slice>"),
        )),
    }
}

#[derive(Debug)]
pub struct Tablet {
    pub name: String,
    pub dir: PathBuf,
}

impl Tablet {
    /// The folder as messages print it (relative to the root).
    pub fn shown(&self) -> String {
        format!("nabu/tablets/{}", self.name)
    }
}

/// The stack file name when the tablet is stacked: the schema has a `stack` key and the file exists.
/// `None` for a plain tablet. Only "not found" means plain: a stack file that is not a regular
/// file, or cannot be read, is an error => its slices never vanish silently.
pub fn stack_file<'a>(tablet: &Tablet, resolved: &'a ResolvedSchema) -> Result<Option<&'a str>> {
    let Some(stack) = &resolved.stack else {
        return Ok(None);
    };
    let shown = format!("{}/{}", tablet.shown(), stack.file);
    match fs::metadata(tablet.dir.join(&stack.file)) {
        Ok(meta) if meta.is_file() => Ok(Some(&stack.file)),
        Ok(_) => Err(Error::new(
            format!("{shown} is not a regular file"),
            format!("replace {shown} with the stack file"),
        )),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(Error::new(
            format!("cannot read {shown}: {err}"),
            format!("check the permissions of {shown}"),
        )),
    }
}

/// `stack_file` for commands that need slices: a plain tablet is a refusal.
/// One owner for `new` and `status`, so they never disagree about which tablets have slices.
pub fn require_stack_file<'a>(tablet: &Tablet, resolved: &'a ResolvedSchema) -> Result<&'a str> {
    if let Some(file) = stack_file(tablet, resolved)? {
        return Ok(file);
    }
    Err(match &resolved.stack {
        None => Error::new(
            format!(
                "tablet \"{}\" is not stacked: {} has no `stack` key",
                tablet.name, resolved.path
            ),
            format!("add stack = \"stack.md\" to {}", resolved.path),
        ),
        Some(stack) => Error::new(
            format!(
                "tablet \"{}\" is not stacked: {}/{} is missing",
                tablet.name,
                tablet.shown(),
                stack.file
            ),
            format!("add {}/{}", tablet.shown(), stack.file),
        ),
    })
}

pub fn tablets_dir(root: &Path) -> PathBuf {
    root.join("nabu/tablets")
}

/// Archived tablets live here, as messages print it. Never searched by lookup.
pub const ARCHIVE_DIR: &str = "nabu/archive";

/// Comma list for "existing ..." messages; `none` when empty.
pub fn listing<'a>(names: impl IntoIterator<Item = &'a str>) -> String {
    let names: Vec<&str> = names.into_iter().collect();
    if names.is_empty() {
        "none".to_string()
    } else {
        names.join(", ")
    }
}

/// Active tablets only: `nabu/archive/` is never searched and never listed.
pub fn list_tablets(root: &Path) -> Result<Vec<String>> {
    let dir = tablets_dir(root);
    let mut names = subfolders(&dir, "nabu/tablets")?;
    // Hidden folders are never tablets: the name rule forbids a leading dot.
    names.retain(|name| !name.starts_with('.'));
    Ok(names)
}

/// The error lists existing tablets so a typo shows the right name at once.
pub fn find_tablet(root: &Path, name: &str) -> Result<Tablet> {
    let dir = tablets_dir(root).join(name);
    // `list_tablets` hides dot names, and `.`/`..` would resolve to folders that are not tablets.
    if !name.is_empty() && !name.starts_with('.') && dir.is_dir() {
        return Ok(Tablet {
            name: name.to_string(),
            dir,
        });
    }
    let existing = list_tablets(root)?;
    Err(Error::new(
        format!(
            "unknown tablet \"{name}\"; existing tablets: {}",
            listing(existing.iter().map(String::as_str))
        ),
        format!("nabu new {name}"),
    ))
}

/// Error paths are relative to the root, like every other printed path.
pub fn read_tablet_file(tablet: &Tablet) -> Result<TabletFile> {
    let path = tablet.dir.join(".nabu.toml");
    let shown = format!("nabu/tablets/{}/.nabu.toml", tablet.name);
    let text = fs::read_to_string(&path).map_err(|err| {
        Error::new(
            format!("cannot read {shown}: {err}"),
            format!("create {shown} containing schema = \"<name>\""),
        )
    })?;
    parse_toml(&shown, &text)
}

#[derive(Debug, PartialEq)]
pub struct Slice {
    pub number: u64,
    pub name: String,
    /// Folder name, `<N>-<name>`.
    pub folder: String,
}

/// Subfolders named `<N>-<slice>` that hold a `.nabu.toml` marker. Other subfolders are not slices.
/// Only a missing marker means "not a slice": an unreadable or non-file marker is an error,
/// because dropping the slice would hide its tasks from `status` and could fake "ready to archive".
pub fn list_slices(tablet: &Tablet) -> Result<Vec<Slice>> {
    let shown = tablet.shown();
    let mut slices = Vec::new();
    for folder in subfolders(&tablet.dir, &shown)? {
        let Some((number, name)) = split_slice_folder(&folder) else {
            continue;
        };
        let marker = tablet.dir.join(&folder).join(".nabu.toml");
        if has_marker(&marker, &format!("{shown}/{folder}/.nabu.toml"))? {
            slices.push(Slice {
                number,
                name: name.to_string(),
                folder,
            });
        }
    }
    slices.sort_by_key(|slice| slice.number);
    Ok(slices)
}

fn has_marker(path: &Path, shown: &str) -> Result<bool> {
    match fs::metadata(path) {
        Ok(meta) if meta.is_file() => Ok(true),
        Ok(_) => Err(Error::new(
            format!("{shown} is not a regular file"),
            format!("replace {shown} with an empty file"),
        )),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(err) => Err(Error::new(
            format!("cannot read {shown}: {err}"),
            format!("check the permissions of {shown}"),
        )),
    }
}

fn split_slice_folder(folder: &str) -> Option<(u64, &str)> {
    let (number, name) = folder.split_once('-')?;
    if number.is_empty() || !number.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    is_valid_name(NameKind::Slice, name).then_some((number.parse().ok()?, name))
}

/// Accepts `<N>`, `<slice>` or `<N>-<slice>`.
pub fn find_slice<'a>(tablet: &Tablet, slices: &'a [Slice], key: &str) -> Result<&'a Slice> {
    let matches: Vec<&Slice> = if !key.is_empty() && key.chars().all(|c| c.is_ascii_digit()) {
        slices
            .iter()
            .filter(|slice| key.parse() == Ok(slice.number))
            .collect()
    } else if split_slice_folder(key).is_some() {
        slices.iter().filter(|slice| slice.folder == key).collect()
    } else {
        slices.iter().filter(|slice| slice.name == key).collect()
    };
    match matches.as_slice() {
        [slice] => Ok(slice),
        [] => Err(Error::new(
            format!(
                "unknown slice \"{key}\" in tablet \"{}\"; existing slices: {}",
                tablet.name,
                listing(slices.iter().map(|s| s.folder.as_str()))
            ),
            format!(
                "use {}/<N>, {}/<slice> or {}/<N>-<slice>",
                tablet.name, tablet.name, tablet.name
            ),
        )),
        many => Err(Error::new(
            format!(
                "slice \"{key}\" in tablet \"{}\" matches several folders: {}",
                tablet.name,
                listing(many.iter().map(|s| s.folder.as_str()))
            ),
            "rename the duplicate folders so every number and name is unique",
        )),
    }
}

/// `shown` is the folder as messages print it (relative to the root).
fn subfolders(dir: &Path, shown: &str) -> Result<Vec<String>> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => {
            return Err(Error::new(
                format!("cannot read {shown}: {err}"),
                format!("check the permissions of {shown}"),
            ));
        }
    };
    let read_error = |err: std::io::Error| {
        Error::new(
            format!("cannot read {shown}: {err}"),
            format!("check the permissions of {shown}"),
        )
    };
    let mut names = Vec::new();
    for entry in entries {
        let entry = entry.map_err(read_error)?;
        if !entry.path().is_dir() {
            continue;
        }
        let name = entry.file_name().into_string().map_err(|name| {
            Error::new(
                format!("{shown} holds a folder whose name is not UTF-8: {name:?}"),
                format!("rename or remove that folder in {shown}"),
            )
        })?;
        names.push(name);
    }
    names.sort();
    Ok(names)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tablet_with(folders: &[&str]) -> (tempfile::TempDir, Tablet) {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("nabu/tablets/t");
        for folder in folders {
            fs::create_dir_all(dir.join(folder)).unwrap();
            fs::write(dir.join(folder).join(".nabu.toml"), "").unwrap();
        }
        fs::create_dir_all(&dir).unwrap();
        let tablet = Tablet {
            name: "t".into(),
            dir,
        };
        (tmp, tablet)
    }

    #[test]
    fn name_rule_is_kebab_case_with_a_letter_first_for_slices() {
        for ok in ["a", "demo", "42-empty-list-notice", "a1-b2"] {
            assert!(validate_name(NameKind::Tablet, ok).is_ok(), "{ok}");
        }
        for bad in ["", "-a", "A", "a_b", "a b", "ä", "a.b"] {
            assert!(validate_name(NameKind::Tablet, bad).is_err(), "{bad:?}");
        }
        assert!(validate_name(NameKind::Slice, "first").is_ok());
        for bad in ["1", "1-first", "1first", "-x", "Up"] {
            assert!(validate_name(NameKind::Slice, bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn invalid_name_fix_is_a_valid_command() {
        let err = validate_name(NameKind::Tablet, "My Tablet_2")
            .unwrap_err()
            .to_string();
        assert!(
            err.ends_with("fix: use a name like \"my-tablet-2\""),
            "{err}"
        );
        let err = validate_name(NameKind::Slice, "???")
            .unwrap_err()
            .to_string();
        assert!(err.ends_with("fix: use a name like \"my-name\""), "{err}");
    }

    #[test]
    fn slices_are_numbered_folders_ordered_by_number() {
        let (_tmp, tablet) =
            tablet_with(&["10-last", "2-second", "1-first", "notes", "x-1", "3-Bad"]);
        let folders: Vec<_> = list_slices(&tablet)
            .unwrap()
            .into_iter()
            .map(|s| s.folder)
            .collect();
        assert_eq!(folders, ["1-first", "2-second", "10-last"]);
    }

    #[test]
    fn slice_shaped_folder_without_marker_is_not_a_slice() {
        let (_tmp, tablet) = tablet_with(&["1-first"]);
        fs::create_dir(tablet.dir.join("2-unmarked")).unwrap();
        let folders: Vec<_> = list_slices(&tablet)
            .unwrap()
            .into_iter()
            .map(|s| s.folder)
            .collect();
        assert_eq!(folders, ["1-first"]);
    }

    #[test]
    fn slice_lookup_forms() {
        let (_tmp, tablet) = tablet_with(&["1-first", "2-second"]);
        let slices = list_slices(&tablet).unwrap();
        for key in ["2", "02", "second", "2-second"] {
            assert_eq!(
                find_slice(&tablet, &slices, key).unwrap().folder,
                "2-second",
                "{key}"
            );
        }
    }

    #[test]
    fn unknown_slice_lists_slices() {
        let (_tmp, tablet) = tablet_with(&["1-first", "2-second"]);
        let slices = list_slices(&tablet).unwrap();
        for key in ["3", "third", "1-second"] {
            let err = find_slice(&tablet, &slices, key).unwrap_err().to_string();
            assert!(err.contains("existing slices: 1-first, 2-second"), "{err}");
        }
    }

    #[test]
    fn unknown_tablet_lists_tablets_without_archive() {
        let tmp = tempfile::tempdir().unwrap();
        for dir in [
            "nabu/tablets/b",
            "nabu/tablets/a",
            "nabu/archive/2026-01-01-gone",
        ] {
            fs::create_dir_all(tmp.path().join(dir)).unwrap();
        }
        let err = find_tablet(tmp.path(), "gone").unwrap_err().to_string();
        assert!(err.contains("existing tablets: a, b"), "{err}");
        assert!(!err.contains("2026"), "{err}");
    }

    #[test]
    fn marker_that_is_not_a_file_is_an_error_naming_it() {
        let (_tmp, tablet) = tablet_with(&["1-first"]);
        let marker = tablet.dir.join("1-first/.nabu.toml");
        fs::remove_file(&marker).unwrap();
        fs::create_dir(&marker).unwrap();
        let err = list_slices(&tablet).unwrap_err().to_string();
        assert!(err.contains("nabu/tablets/t/1-first/.nabu.toml"), "{err}");
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_folder_name_is_an_error_naming_the_parent() {
        use std::os::unix::ffi::OsStrExt;
        let (_tmp, tablet) = tablet_with(&["1-first"]);
        let odd = std::ffi::OsStr::from_bytes(b"2-\xff");
        fs::create_dir(tablet.dir.join(odd)).unwrap();
        let err = list_slices(&tablet).unwrap_err().to_string();
        assert!(err.contains("nabu/tablets/t holds a folder"), "{err}");
    }

    fn stacked_schema() -> ResolvedSchema {
        let layers = crate::layers::Layers::load(Path::new("/nonexistent"), None).unwrap();
        crate::schema::resolve(&layers, "specless", "nabu/config.toml").unwrap()
    }

    #[test]
    fn stack_file_is_none_only_when_there_is_no_key_or_no_file() {
        let (_tmp, tablet) = tablet_with(&[]);
        let resolved = stacked_schema();
        assert_eq!(stack_file(&tablet, &resolved).unwrap(), None);
        fs::write(tablet.dir.join("stack.md"), "x").unwrap();
        assert_eq!(stack_file(&tablet, &resolved).unwrap(), Some("stack.md"));
        let mut no_key = stacked_schema();
        no_key.stack = None;
        assert_eq!(stack_file(&tablet, &no_key).unwrap(), None);
    }

    #[test]
    fn stack_file_that_is_a_folder_is_an_error_not_a_plain_tablet() {
        let (_tmp, tablet) = tablet_with(&[]);
        fs::create_dir(tablet.dir.join("stack.md")).unwrap();
        let err = stack_file(&tablet, &stacked_schema())
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("nabu/tablets/t/stack.md is not a regular file"),
            "{err}"
        );
        assert!(require_stack_file(&tablet, &stacked_schema()).is_err());
    }
}
