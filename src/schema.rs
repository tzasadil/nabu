use crate::error::{Error, Result};
use crate::layers::Layers;
use crate::toml_file::parse_toml;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

/// Unknown keys are errors, missing required keys are errors.
/// The artifact graph is checked in `resolve`, not at parse time, because it needs the layers.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Schema {
    // Documentation only: no command prints these two. The schema is named by its folder.
    #[allow(dead_code)]
    pub name: String,
    #[allow(dead_code)]
    pub description: Option<String>,
    /// Template file that `new --stack` writes as `stack.md`.
    pub stack: Option<String>,
    pub artifacts: Vec<Artifact>,
    pub build: Build,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub id: String,
    pub generates: String,
    pub template: String,
    pub description: Option<String>,
    pub instruction: Option<String>,
    #[serde(default)]
    pub requires: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Build {
    pub requires: Vec<String>,
    /// File whose checkboxes count as progress.
    pub tracks: String,
    pub instruction: Option<String>,
}

#[derive(Debug)]
pub struct ResolvedSchema {
    pub schema: Schema,
    /// Where the winning `schema.toml` lives, as messages print it.
    pub path: String,
    /// Artifact id -> template text. Every artifact has one: `resolve` rejects the schema otherwise.
    pub templates: BTreeMap<String, String>,
    /// Set when the schema has a `stack` key. Its template is known to resolve.
    pub stack: Option<StackTemplate>,
}

#[derive(Debug)]
pub struct StackTemplate {
    /// File name `new --stack` writes in the tablet.
    pub file: String,
    pub text: String,
}

/// Takes the whole `schema.toml` from the nearest layer that has one.
/// `named_in` is the file that named the schema, so the unknown-schema fix points at the right file.
pub fn resolve(layers: &Layers, name: &str, named_in: &str) -> Result<ResolvedSchema> {
    let Some((path, text)) = layers.schema_file(name) else {
        let found: Vec<_> = layers.schema_names().into_iter().collect();
        return Err(Error::new(
            format!("unknown schema \"{name}\"; found: {}", found.join(", ")),
            format!("set schema = \"<name>\" in {named_in} to one of the found schemas"),
        ));
    };
    let schema: Schema = parse_toml(&path, text)?;
    let stack = schema
        .stack
        .as_deref()
        .map(|file| stack_template(layers, name, &path, file))
        .transpose()?;
    let templates = check_graph(layers, name, &path, &schema)?;
    Ok(ResolvedSchema {
        schema,
        path,
        templates,
        stack,
    })
}

/// Returns the template text per artifact id. Runs in `resolve` so no command can skip it.
fn check_graph(
    layers: &Layers,
    name: &str,
    path: &str,
    schema: &Schema,
) -> Result<BTreeMap<String, String>> {
    let mut ids = BTreeSet::new();
    for artifact in &schema.artifacts {
        // `instructions` prints `generates` as the path skills write to, so it must stay inside the folder.
        if !is_plain_file_name(&artifact.generates) {
            return Err(Error::new(
                format!(
                    "{path}: invalid `generates` value \"{}\" of artifact \"{}\"",
                    artifact.generates, artifact.id
                ),
                format!("set generates to a plain file name such as \"tasks.md\" in {path}"),
            ));
        }
        if !ids.insert(artifact.id.as_str()) {
            return Err(Error::new(
                format!("{path}: duplicate artifact id \"{}\"", artifact.id),
                format!("give every [[artifacts]] entry in {path} its own id"),
            ));
        }
    }
    let known = || ids.iter().copied().collect::<Vec<_>>().join(", ");
    let unknown_id = |owner: String, id: &str| {
        Error::new(
            format!(
                "{path}: {owner} names unknown id \"{id}\"; known ids: {}",
                known()
            ),
            format!("use a known id, or add the artifact, in {path}"),
        )
    };
    for artifact in &schema.artifacts {
        if let Some(id) = artifact
            .requires
            .iter()
            .find(|id| !ids.contains(id.as_str()))
        {
            return Err(unknown_id(
                format!("artifact \"{}\" requires", artifact.id),
                id,
            ));
        }
    }
    if let Some(id) = schema
        .build
        .requires
        .iter()
        .find(|id| !ids.contains(id.as_str()))
    {
        return Err(unknown_id("build.requires".to_string(), id));
    }
    let tracks = &schema.build.tracks;
    if !schema.artifacts.iter().any(|a| &a.generates == tracks) {
        let generated: Vec<_> = schema
            .artifacts
            .iter()
            .map(|a| a.generates.as_str())
            .collect();
        return Err(Error::new(
            format!(
                "{path}: build.tracks \"{tracks}\" is not generated by any artifact; generated files: {}",
                generated.join(", ")
            ),
            format!("set build.tracks in {path} to the `generates` value of an artifact"),
        ));
    }
    let mut templates = BTreeMap::new();
    for artifact in &schema.artifacts {
        let text = layers.template(name, &artifact.template).ok_or_else(|| {
            Error::new(
                format!(
                    "{path}: template \"{}\" of artifact \"{}\" is in no layer",
                    artifact.template, artifact.id
                ),
                format!(
                    "add schemas/{name}/templates/{} to a layer, or change `template` in {path}",
                    artifact.template
                ),
            )
        })?;
        templates.insert(artifact.id.clone(), text.to_string());
    }
    if let Some(cycle) = find_cycle(schema) {
        return Err(Error::new(
            format!("{path}: dependency cycle: {}", cycle.join(" -> ")),
            format!("remove one `requires` entry on the cycle in {path}"),
        ));
    }
    Ok(templates)
}

/// The ids on the first cycle found, first id repeated at the end. Depth-first, in declaration order.
fn find_cycle(schema: &Schema) -> Option<Vec<&str>> {
    enum Mark {
        OnPath,
        Done,
    }
    fn visit<'a>(
        schema: &'a Schema,
        id: &'a str,
        marks: &mut BTreeMap<&'a str, Mark>,
        path: &mut Vec<&'a str>,
    ) -> Option<Vec<&'a str>> {
        match marks.get(id) {
            Some(Mark::Done) => return None,
            Some(Mark::OnPath) => {
                let start = path.iter().position(|seen| *seen == id)?;
                let mut cycle = path[start..].to_vec();
                cycle.push(id);
                return Some(cycle);
            }
            None => {}
        }
        marks.insert(id, Mark::OnPath);
        path.push(id);
        if let Some(artifact) = schema.artifacts.iter().find(|a| a.id == id) {
            for dependency in &artifact.requires {
                if let Some(cycle) = visit(schema, dependency, marks, path) {
                    return Some(cycle);
                }
            }
        }
        path.pop();
        marks.insert(id, Mark::Done);
        None
    }
    let mut marks = BTreeMap::new();
    let mut path = Vec::new();
    schema
        .artifacts
        .iter()
        .find_map(|artifact| visit(schema, &artifact.id, &mut marks, &mut path))
}

/// No folder part and no leading dot: the name cannot leave its folder or hit `.nabu.toml`.
fn is_plain_file_name(name: &str) -> bool {
    !(name.is_empty() || name.contains(['/', '\\']) || name.starts_with('.'))
}

fn stack_template(layers: &Layers, name: &str, path: &str, file: &str) -> Result<StackTemplate> {
    // The file is written inside the tablet folder, so it must be a plain name.
    // Checked here, before `new` writes anything, to avoid a half-created tablet.
    if !is_plain_file_name(file) {
        return Err(Error::new(
            format!("{path}: invalid `stack` value \"{file}\""),
            format!("set stack to a plain file name such as \"stack.md\" in {path}"),
        ));
    }
    let text = layers.template(name, file).ok_or_else(|| {
        Error::new(
            format!("{path}: stack template \"{file}\" is in no layer"),
            format!(
                "add schemas/{name}/templates/{file} to a layer, or remove `stack` from {path}"
            ),
        )
    })?;
    Ok(StackTemplate {
        file: file.to_string(),
        text: text.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layers::Layer;

    fn builtin() -> Layers {
        Layers::load(std::path::Path::new("/nonexistent"), None).unwrap()
    }

    const MINIMAL: &str = r#"
name = "specless"
stack = "stack.md"
artifacts = []
[build]
requires = []
tracks = "tasks.md"
"#;

    #[test]
    fn built_in_schema_loads_with_zero_config() {
        let resolved = resolve(&builtin(), "specless", "nabu/config.toml").unwrap();
        let ids: Vec<_> = resolved
            .schema
            .artifacts
            .iter()
            .map(|a| a.id.as_str())
            .collect();
        assert_eq!(ids, ["blueprint", "backlog", "tasks"]);
        let tasks = &resolved.schema.artifacts[2];
        assert_eq!(tasks.requires, ["blueprint"]);
        assert_eq!(resolved.schema.build.tracks, "tasks.md");
        assert_eq!(resolved.path, "defaults/schemas/specless/schema.toml");
        assert!(resolved.stack.unwrap().text.starts_with("# Stack:"));
    }

    #[test]
    fn repo_override_wins_whole_file_and_templates_resolve_per_file() {
        let fixture =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/override");
        let layers = Layers::load(&fixture, None).unwrap();
        let resolved = resolve(&layers, "specless", "nabu/config.toml").unwrap();
        assert_eq!(resolved.path, "nabu/schemas/specless/schema.toml");
        assert!(resolved.stack.is_none());
        assert!(
            layers
                .template("specless", "blueprint.md")
                .unwrap()
                .contains("## Intent")
        );
        assert!(
            layers
                .template("specless", "tasks.md")
                .unwrap()
                .starts_with("# TODO")
        );
    }

    #[test]
    fn unknown_schema_lists_found_ones() {
        let err = resolve(&builtin(), "nope", "nabu/tablets/t/.nabu.toml")
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("unknown schema \"nope\"; found: specless"),
            "{err}"
        );
        assert!(
            err.ends_with("in nabu/tablets/t/.nabu.toml to one of the found schemas"),
            "{err}"
        );
    }

    #[test]
    fn missing_required_key_names_the_file() {
        let layers = Layers::from_layers(vec![Layer::new(
            "nabu/",
            [("schemas/s/schema.toml", "name = \"s\"\nartifacts = []\n")],
        )]);
        let err = resolve(&layers, "s", "nabu/config.toml")
            .unwrap_err()
            .to_string();
        assert!(err.contains("nabu/schemas/s/schema.toml"), "{err}");
        assert!(err.contains("build"), "{err}");
    }

    #[test]
    fn unresolvable_stack_template_names_file_and_template() {
        let layers = Layers::from_layers(vec![Layer::new(
            "nabu/",
            [("schemas/specless/schema.toml", MINIMAL)],
        )]);
        let err = resolve(&layers, "specless", "nabu/config.toml")
            .unwrap_err()
            .to_string();
        assert!(err.contains("nabu/schemas/specless/schema.toml"), "{err}");
        assert!(err.contains("\"stack.md\""), "{err}");
    }

    #[test]
    fn stack_value_must_be_a_plain_file_name() {
        for bad in ["sub/x.md", "sub\\x.md", "../x.md", ".hidden.md", ""] {
            let schema = MINIMAL.replace("stack.md", &bad.replace('\\', "\\\\"));
            let layers = Layers::from_layers(vec![Layer::new(
                "nabu/",
                [
                    ("schemas/specless/schema.toml", schema.as_str()),
                    ("schemas/specless/templates/sub/x.md", "x"),
                ],
            )]);
            let err = resolve(&layers, "specless", "nabu/config.toml")
                .unwrap_err()
                .to_string();
            assert!(err.contains("invalid `stack` value"), "{bad:?}: {err}");
            assert!(err.contains("nabu/schemas/specless/schema.toml"), "{err}");
        }
    }

    #[test]
    fn generates_must_be_a_plain_file_name() {
        for bad in ["sub/a.md", "sub\\a.md", "../a.md", ".hidden.md", ""] {
            let broken = artifact("a", &[]).replace("\"a.md\"", &format!("{bad:?}"));
            let err = resolve_graph(&[broken], BUILD).unwrap_err().to_string();
            assert!(err.contains("invalid `generates` value"), "{bad:?}: {err}");
            assert!(err.contains("nabu/schemas/s/schema.toml"), "{err}");
            assert!(err.contains(&format!("\"{bad}\"")), "{bad:?}: {err}");
        }
    }

    fn artifact(id: &str, requires: &[&str]) -> String {
        let requires: Vec<_> = requires.iter().map(|id| format!("\"{id}\"")).collect();
        format!(
            "[[artifacts]]\nid = \"{id}\"\ngenerates = \"{id}.md\"\ntemplate = \"t.md\"\nrequires = [{}]\n",
            requires.join(", ")
        )
    }

    /// Resolves schema `s` from one repo layer that holds the template `t.md`.
    fn resolve_graph(artifacts: &[String], build: &str) -> Result<ResolvedSchema> {
        let schema = format!("name = \"s\"\n{}\n[build]\n{build}", artifacts.join("\n"));
        let layers = Layers::from_layers(vec![Layer::new(
            "nabu/",
            [
                ("schemas/s/schema.toml", schema.as_str()),
                ("schemas/s/templates/t.md", "template"),
            ],
        )]);
        resolve(&layers, "s", "nabu/config.toml")
    }

    const BUILD: &str = "requires = [\"a\"]\ntracks = \"a.md\"\n";

    #[test]
    fn valid_graph_resolves_templates_per_artifact() {
        let resolved = resolve_graph(&[artifact("a", &[]), artifact("b", &["a"])], BUILD).unwrap();
        assert_eq!(resolved.templates["b"], "template");
    }

    #[test]
    fn duplicate_artifact_id_is_an_error() {
        let err = resolve_graph(&[artifact("a", &[]), artifact("a", &[])], BUILD)
            .unwrap_err()
            .to_string();
        assert!(err.contains("nabu/schemas/s/schema.toml"), "{err}");
        assert!(err.contains("duplicate artifact id \"a\""), "{err}");
    }

    #[test]
    fn unknown_requires_names_the_artifact_and_the_id() {
        let err = resolve_graph(&[artifact("a", &["ghost"])], BUILD)
            .unwrap_err()
            .to_string();
        assert!(err.contains("nabu/schemas/s/schema.toml"), "{err}");
        assert!(
            err.contains("artifact \"a\" requires names unknown id \"ghost\""),
            "{err}"
        );
        assert!(err.contains("known ids: a"), "{err}");
    }

    #[test]
    fn unknown_build_requires_is_an_error() {
        let err = resolve_graph(
            &[artifact("a", &[])],
            "requires = [\"ghost\"]\ntracks = \"a.md\"\n",
        )
        .unwrap_err()
        .to_string();
        assert!(
            err.contains("build.requires names unknown id \"ghost\""),
            "{err}"
        );
    }

    #[test]
    fn tracks_must_be_generated_by_an_artifact() {
        let err = resolve_graph(
            &[artifact("a", &[])],
            "requires = []\ntracks = \"other.md\"\n",
        )
        .unwrap_err()
        .to_string();
        assert!(
            err.contains("build.tracks \"other.md\" is not generated"),
            "{err}"
        );
        assert!(err.contains("generated files: a.md"), "{err}");
    }

    #[test]
    fn template_in_no_layer_names_artifact_and_template() {
        let mut broken = artifact("a", &[]);
        broken = broken.replace("t.md\"", "missing.md\"");
        let err = resolve_graph(&[broken], BUILD).unwrap_err().to_string();
        assert!(
            err.contains("template \"missing.md\" of artifact \"a\" is in no layer"),
            "{err}"
        );
        assert!(err.contains("schemas/s/templates/missing.md"), "{err}");
    }

    #[test]
    fn cycle_names_the_ids_on_it_and_not_those_leading_in() {
        let err = resolve_graph(
            &[
                artifact("a", &["b"]),
                artifact("b", &["c"]),
                artifact("c", &["d"]),
                artifact("d", &["b"]),
            ],
            BUILD,
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("nabu/schemas/s/schema.toml"), "{err}");
        assert!(err.contains("dependency cycle: b -> c -> d -> b"), "{err}");
    }

    #[test]
    fn self_requirement_is_a_cycle() {
        let err = resolve_graph(&[artifact("a", &["a"])], BUILD)
            .unwrap_err()
            .to_string();
        assert!(err.contains("dependency cycle: a -> a"), "{err}");
    }

    #[test]
    fn shared_dependency_is_not_a_cycle() {
        let artifacts = [
            artifact("a", &[]),
            artifact("b", &["a"]),
            artifact("c", &["a", "b"]),
        ];
        assert!(resolve_graph(&artifacts, BUILD).is_ok());
    }
}
