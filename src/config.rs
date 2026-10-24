use crate::error::{Error, Result};
use crate::layers::Layers;
use crate::schema::ResolvedSchema;
use crate::toml_file::parse_toml;
use serde::Deserialize;
use std::collections::BTreeMap;

/// Unknown keys are errors => typos fail loudly instead of being ignored.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConfig {
    schema: Option<String>,
    context: Option<String>,
    #[serde(default)]
    rules: BTreeMap<String, Vec<String>>,
}

/// `config.toml` merged over all layers.
#[derive(Debug, PartialEq)]
pub struct Config {
    /// Nearest layer that sets it wins.
    pub schema: String,
    /// Every layer's text, built-in first, then global, then repo.
    pub context: Vec<String>,
    /// Lists concatenated per artifact id, global first.
    pub rules: BTreeMap<String, Vec<String>>,
    /// Rules key -> the config files that set it. Merging loses the file, and the unknown-key
    /// error must name it. The key set is the same as `rules`.
    rule_sources: BTreeMap<String, Vec<String>>,
}

impl Config {
    pub fn load(layers: &Layers) -> Result<Self> {
        let mut schema = None;
        let mut context = Vec::new();
        let mut rules: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut rule_sources: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for layer in layers.farthest_first() {
            let Some(text) = layer.file("config.toml") else {
                continue;
            };
            let raw: RawConfig = parse_toml(&layer.path("config.toml"), text)?;
            schema = raw.schema.or(schema);
            context.extend(raw.context);
            for (artifact, list) in raw.rules {
                rule_sources
                    .entry(artifact.clone())
                    .or_default()
                    .push(layer.path("config.toml"));
                rules.entry(artifact).or_default().extend(list);
            }
        }
        let schema = schema.ok_or_else(|| {
            Error::new(
                "no config sets `schema`",
                "add schema = \"specless\" to nabu/config.toml",
            )
        })?;
        Ok(Self {
            schema,
            context,
            rules,
            rule_sources,
        })
    }

    /// A key that is no artifact id of the tablet's schema would never be served, so a typo fails loudly.
    /// Per tablet, not at load: tablets may use different schemas.
    pub fn check_rules_keys(&self, schema_name: &str, resolved: &ResolvedSchema) -> Result<()> {
        let ids: Vec<&str> = resolved
            .schema
            .artifacts
            .iter()
            .map(|artifact| artifact.id.as_str())
            .collect();
        let Some((key, files)) = self
            .rule_sources
            .iter()
            .find(|(key, _)| !ids.contains(&key.as_str()))
        else {
            return Ok(());
        };
        let files = files.join(", ");
        Err(Error::new(
            format!(
                "{files} sets [rules] \"{key}\", which is no artifact of schema \"{schema_name}\" ({}); artifacts: {}",
                resolved.path,
                ids.join(", ")
            ),
            format!("rename \"{key}\" to an artifact id, or remove it, in {files}"),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layers::Layer;
    use crate::schema;

    fn layers(configs: &[&str]) -> Layers {
        Layers::from_layers(
            configs
                .iter()
                .enumerate()
                .map(|(i, text)| Layer::new(&format!("l{i}/"), [("config.toml", *text)]))
                .collect(),
        )
    }

    #[test]
    fn context_and_rules_append_farthest_first() {
        let config = Config::load(&layers(&[
            r#"schema = "a"
context = "built-in"
[rules]
tasks = ["1"]
"#,
            r#"context = "global"
[rules]
tasks = ["2"]
blueprint = ["g"]
"#,
            r#"context = "repo"
[rules]
tasks = ["3", "4"]
"#,
        ]))
        .unwrap();
        assert_eq!(config.context, ["built-in", "global", "repo"]);
        assert_eq!(config.rules["tasks"], ["1", "2", "3", "4"]);
        assert_eq!(config.rules["blueprint"], ["g"]);
    }

    fn check_rules(configs: &[&str]) -> Result<()> {
        let resolved = schema::resolve(
            &Layers::from_layers(vec![Layer::new(
                "nabu/",
                [
                    ("schemas/s/schema.toml", SCHEMA),
                    ("schemas/s/templates/t.md", "t"),
                ],
            )]),
            "s",
            "nabu/config.toml",
        )
        .unwrap();
        Config::load(&layers(configs))
            .unwrap()
            .check_rules_keys("s", &resolved)
    }

    const SCHEMA: &str = r#"name = "s"
[[artifacts]]
id = "a"
generates = "a.md"
template = "t.md"
[build]
requires = []
tracks = "a.md"
"#;

    #[test]
    fn rules_keys_that_are_artifact_ids_pass() {
        check_rules(&[
            "schema = \"s\"\n[rules]\na = [\"x\"]\n",
            "[rules]\na = []\n",
        ])
        .unwrap();
        check_rules(&["schema = \"s\""]).unwrap();
    }

    #[test]
    fn unknown_rules_key_names_every_file_that_sets_it() {
        let err = check_rules(&[
            "schema = \"s\"\n[rules]\na = [\"x\"]\nghost = [\"1\"]\n",
            "[rules]\nother = []\n",
            "[rules]\nghost = [\"2\"]\n",
        ])
        .unwrap_err()
        .to_string();
        assert!(err.contains("l0/config.toml, l2/config.toml"), "{err}");
        assert!(!err.contains("l1/config.toml"), "{err}");
        assert!(err.contains("\"ghost\""), "{err}");
        assert!(
            err.contains("schema \"s\" (nabu/schemas/s/schema.toml)"),
            "{err}"
        );
        assert!(err.contains("artifacts: a"), "{err}");
    }

    #[test]
    fn schema_nearest_wins() {
        let config = Config::load(&layers(&[
            "schema = \"a\"",
            "schema = \"b\"",
            "context = \"no schema here\"",
        ]))
        .unwrap();
        assert_eq!(config.schema, "b");
    }

    #[test]
    fn unknown_key_names_the_file() {
        let err = Config::load(&layers(&["schema = \"a\"", "shema = \"b\""]))
            .unwrap_err()
            .to_string();
        assert!(err.contains("l1/config.toml"), "{err}");
        assert!(err.contains("shema"), "{err}");
    }

    #[test]
    fn missing_schema_is_an_error_with_a_fix() {
        let err = Config::load(&layers(&["context = \"x\""]))
            .unwrap_err()
            .to_string();
        assert!(err.contains("no config sets `schema`"), "{err}");
        assert!(err.contains("fix: add schema"), "{err}");
    }
}
