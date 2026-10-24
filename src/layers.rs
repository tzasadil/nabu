use crate::error::{Error, Result};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::Path;

/// Layer-relative path (`schemas/specless/schema.toml`) -> file text.
/// Built-in, global and repo layers all load into this shape => one loader, no interface.
type Files = BTreeMap<String, String>;

#[derive(Debug)]
pub struct Layer {
    /// Shown in messages: `defaults/`, `nabu/` (relative to the root) or an absolute path.
    prefix: String,
    files: Files,
}

impl Layer {
    pub fn new<'a>(prefix: &str, files: impl IntoIterator<Item = (&'a str, &'a str)>) -> Self {
        Self {
            prefix: prefix.to_string(),
            files: files
                .into_iter()
                .map(|(path, text)| (path.to_string(), text.to_string()))
                .collect(),
        }
    }

    pub fn file(&self, rel: &str) -> Option<&str> {
        self.files.get(rel).map(String::as_str)
    }

    /// Path of a layer file as messages print it.
    pub fn path(&self, rel: &str) -> String {
        format!("{}{rel}", self.prefix)
    }
}

/// Every file under `defaults/`. An explicit list instead of an embedding crate;
/// the `every_default_file_is_embedded` test fails when a file is missing here.
const BUILTIN: &[(&str, &str)] = &[
    ("config.toml", include_str!("../defaults/config.toml")),
    (
        "schemas/specless/schema.toml",
        include_str!("../defaults/schemas/specless/schema.toml"),
    ),
    (
        "schemas/specless/templates/backlog.md",
        include_str!("../defaults/schemas/specless/templates/backlog.md"),
    ),
    (
        "schemas/specless/templates/blueprint.md",
        include_str!("../defaults/schemas/specless/templates/blueprint.md"),
    ),
    (
        "schemas/specless/templates/stack.md",
        include_str!("../defaults/schemas/specless/templates/stack.md"),
    ),
    (
        "schemas/specless/templates/tasks.md",
        include_str!("../defaults/schemas/specless/templates/tasks.md"),
    ),
];

/// Layers in precedence order, farthest first: built-in, global, repo.
#[derive(Debug)]
pub struct Layers(Vec<Layer>);

impl Layers {
    #[cfg(test)]
    pub fn from_layers(farthest_first: Vec<Layer>) -> Self {
        Self(farthest_first)
    }

    /// A missing global or repo folder is skipped.
    pub fn load(root: &Path, global_dir: Option<&Path>) -> Result<Self> {
        let mut layers = vec![Layer::new("defaults/", BUILTIN.iter().copied())];
        if let Some(dir) = global_dir {
            let prefix = format!("{}/", dir.display());
            layers.extend(read_layer(dir, &prefix)?);
        }
        layers.extend(read_layer(&root.join("nabu"), "nabu/")?);
        Ok(Self(layers))
    }

    pub fn farthest_first(&self) -> impl Iterator<Item = &Layer> {
        self.0.iter()
    }

    pub fn nearest_first(&self) -> impl Iterator<Item = &Layer> {
        self.0.iter().rev()
    }

    /// Resolved per file: a repo can override one template without copying `schema.toml`.
    pub fn template(&self, schema: &str, file: &str) -> Option<&str> {
        let rel = format!("schemas/{schema}/templates/{file}");
        self.nearest_first().find_map(|layer| layer.file(&rel))
    }

    /// The whole `schema.toml` of the nearest layer that has one: its printed path and text.
    pub fn schema_file(&self, schema: &str) -> Option<(String, &str)> {
        let rel = format!("schemas/{schema}/schema.toml");
        self.nearest_first()
            .find_map(|layer| Some((layer.path(&rel), layer.file(&rel)?)))
    }

    /// Names of schemas that have a `schema.toml` in any layer.
    pub fn schema_names(&self) -> BTreeSet<String> {
        self.farthest_first()
            .flat_map(|layer| layer.files.keys())
            .filter_map(|path| path.strip_prefix("schemas/")?.strip_suffix("/schema.toml"))
            .map(str::to_string)
            .collect()
    }
}

/// Reads only the files the layout defines: `config.toml`, `schemas/<name>/schema.toml` and
/// `schemas/<name>/templates/*.md`. Anything else (`.DS_Store`, swap files) may be binary,
/// so reading it would break every command.
fn read_layer(dir: &Path, prefix: &str) -> Result<Option<Layer>> {
    if !dir.is_dir() {
        return Ok(None);
    }
    let mut files = Files::new();
    let mut load = |rel: String| -> Result<()> {
        if let Some(text) = read_file(&dir.join(&rel), &format!("{prefix}{rel}"))? {
            files.insert(rel, text);
        }
        Ok(())
    };
    load("config.toml".to_string())?;
    for schema in entry_names(
        &dir.join("schemas"),
        &format!("{prefix}schemas"),
        EntryKind::Dir,
    )? {
        load(format!("schemas/{schema}/schema.toml"))?;
        let templates = format!("schemas/{schema}/templates");
        for file in entry_names(
            &dir.join(&templates),
            &format!("{prefix}{templates}"),
            EntryKind::File,
        )? {
            if file.ends_with(".md") {
                load(format!("{templates}/{file}"))?;
            }
        }
    }
    Ok(Some(Layer {
        prefix: prefix.to_string(),
        files,
    }))
}

enum EntryKind {
    Dir,
    File,
}

/// Names of non-hidden entries of one kind. A missing folder has none.
fn entry_names(dir: &Path, shown: &str, kind: EntryKind) -> Result<Vec<String>> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => return Err(read_error(shown, &err)),
    };
    let mut names = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|err| read_error(shown, &err))?;
        let wanted = match kind {
            EntryKind::Dir => entry.path().is_dir(),
            EntryKind::File => entry.path().is_file(),
        };
        let name = entry.file_name().to_string_lossy().to_string();
        if wanted && !name.starts_with('.') {
            names.push(name);
        }
    }
    Ok(names)
}

/// Only "not found" is `None`. Any other failure is an error naming the file.
pub(crate) fn read_file(path: &Path, shown: &str) -> Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(read_error(shown, &err)),
    }
}

fn read_error(shown: &str, err: &io::Error) -> Error {
    Error::new(
        format!("cannot read {shown}: {err}"),
        format!("check that {shown} is readable UTF-8 text"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::walk_files;

    #[test]
    fn every_default_file_is_embedded() {
        let mut on_disk = Vec::new();
        walk_files(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("defaults"),
            "",
            &mut on_disk,
        );
        on_disk.sort();
        let embedded: Vec<&str> = BUILTIN.iter().map(|(path, _)| *path).collect();
        assert_eq!(on_disk, embedded);
    }

    #[test]
    fn template_resolves_per_file_nearest_first() {
        let layers = Layers::from_layers(vec![
            Layer::new(
                "a/",
                [
                    ("schemas/s/templates/x.md", "built-in x"),
                    ("schemas/s/templates/y.md", "built-in y"),
                ],
            ),
            Layer::new("b/", [("schemas/s/templates/x.md", "repo x")]),
        ]);
        assert_eq!(layers.template("s", "x.md"), Some("repo x"));
        assert_eq!(layers.template("s", "y.md"), Some("built-in y"));
        assert_eq!(layers.template("s", "z.md"), None);
        assert_eq!(layers.template("other", "x.md"), None);
    }

    #[test]
    fn schema_names_span_all_layers() {
        let layers = Layers::from_layers(vec![
            Layer::new("a/", [("schemas/one/schema.toml", "")]),
            Layer::new(
                "b/",
                [
                    ("schemas/two/schema.toml", ""),
                    ("schemas/one/schema.toml", ""),
                    ("schemas/three/templates/x.md", ""),
                ],
            ),
        ]);
        let names: Vec<_> = layers.schema_names().into_iter().collect();
        assert_eq!(names, ["one", "two"]);
    }

    #[test]
    fn loads_repo_layer_and_skips_missing_ones() {
        let tmp = tempfile::tempdir().unwrap();
        let schema = tmp.path().join("nabu/schemas/s/templates");
        fs::create_dir_all(&schema).unwrap();
        fs::write(schema.join("x.md"), "x").unwrap();
        fs::write(tmp.path().join("nabu/config.toml"), "schema = \"s\"").unwrap();
        let missing_global = tmp.path().join("no-such-dir");
        let layers = Layers::load(tmp.path(), Some(&missing_global)).unwrap();
        assert_eq!(layers.farthest_first().count(), 2);
        assert_eq!(layers.template("s", "x.md"), Some("x"));
    }

    #[test]
    fn loads_only_the_layout_files() {
        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path().join("nabu");
        let templates = base.join("schemas/s/templates");
        fs::create_dir_all(&templates).unwrap();
        fs::write(base.join("schemas/s/schema.toml"), "s").unwrap();
        fs::write(templates.join("x.md"), "x").unwrap();
        // Binary or stray files must not be read at all.
        for stray in [
            base.join(".DS_Store"),
            base.join("schemas/.DS_Store"),
            base.join("schemas/s/notes.txt"),
            templates.join(".DS_Store"),
            templates.join("._x.md"),
            templates.join("y.txt"),
        ] {
            fs::write(stray, [0xff, 0xfe, 0x00]).unwrap();
        }
        let layers = Layers::load(tmp.path(), None).unwrap();
        let repo = layers.nearest_first().next().unwrap();
        let mut loaded: Vec<_> = repo.files.keys().cloned().collect();
        loaded.sort();
        assert_eq!(
            loaded,
            ["schemas/s/schema.toml", "schemas/s/templates/x.md"]
        );
    }
}
