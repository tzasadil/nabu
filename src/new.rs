use crate::config::Config;
use crate::error::{Error, Result};
use crate::layers::Layers;
use crate::schema::{self, ResolvedSchema, StackTemplate};
use crate::tablets::{
    NameKind, TabletFile, find_tablet, list_slices, read_tablet_file, require_stack_file,
    split_target, tablets_dir, validate_name,
};
use std::fs;
use std::path::Path;

/// Returns the printed result: the created path relative to the root.
/// Every refusal happens before the first write => a refused call leaves the tree unchanged.
pub fn run(
    root: &Path,
    layers: &Layers,
    config: &Config,
    target: &str,
    stack: bool,
) -> Result<String> {
    match split_target(target, "new")? {
        (tablet, None) => {
            validate_name(NameKind::Tablet, tablet)?;
            new_tablet(root, layers, config, tablet, stack)
        }
        (tablet, Some(slice)) => {
            validate_name(NameKind::Tablet, tablet)?;
            validate_name(NameKind::Slice, slice)?;
            if stack {
                return Err(Error::new(
                    "--stack applies to a tablet, not to a slice",
                    format!("nabu new {tablet}/{slice}"),
                ));
            }
            new_slice(root, layers, tablet, slice)
        }
    }
}

fn new_tablet(
    root: &Path,
    layers: &Layers,
    config: &Config,
    name: &str,
    stack: bool,
) -> Result<String> {
    let dir = tablets_dir(root).join(name);
    let shown = format!("nabu/tablets/{name}");
    if dir.exists() {
        return Err(Error::new(
            format!("tablet \"{name}\" already exists at {shown}"),
            "pick another name, or work in the existing tablet",
        ));
    }
    let resolved = schema::resolve(layers, &config.schema, "nabu/config.toml")?;
    let stack_template = if stack {
        Some(require_stack(&resolved)?)
    } else {
        None
    };

    let tablet_file = toml::to_string(&TabletFile {
        schema: config.schema.clone(),
    })
    .expect("a single string field always serializes");
    create_dir(&dir, &shown, true)?;
    write(
        &dir.join(".nabu.toml"),
        &format!("{shown}/.nabu.toml"),
        &tablet_file,
    )?;
    if let Some(stack) = stack_template {
        write(
            &dir.join(&stack.file),
            &format!("{shown}/{}", stack.file),
            &stack.text,
        )?;
    }
    Ok(shown)
}

fn require_stack(resolved: &ResolvedSchema) -> Result<&StackTemplate> {
    resolved.stack.as_ref().ok_or_else(|| {
        Error::new(
            format!(
                "{}: schema has no `stack` key, so it cannot make a stacked tablet",
                resolved.path
            ),
            format!("add stack = \"stack.md\" to {}", resolved.path),
        )
    })
}

fn new_slice(root: &Path, layers: &Layers, tablet_name: &str, slice: &str) -> Result<String> {
    let tablet = find_tablet(root, tablet_name)?;
    let tablet_file = read_tablet_file(&tablet)?;
    let shown_tablet = tablet.shown();
    let resolved = schema::resolve(
        layers,
        &tablet_file.schema,
        &format!("{shown_tablet}/.nabu.toml"),
    )?;

    require_stack_file(&tablet, &resolved)?;

    let slices = list_slices(&tablet)?;
    if let Some(used) = slices.iter().find(|existing| existing.name == slice) {
        return Err(Error::new(
            format!(
                "slice \"{slice}\" already exists in tablet \"{tablet_name}\" as {}",
                used.folder
            ),
            "pick another slice name",
        ));
    }
    let number = slices
        .iter()
        .map(|s| s.number)
        .max()
        .map_or(1, |max| max + 1);
    let folder = format!("{number}-{slice}");
    let dir = tablet.dir.join(&folder);
    let shown = format!("{shown_tablet}/{folder}");
    create_dir(&dir, &shown, false)?;
    // jj and git track files, not folders: the empty marker keeps a fresh slice alive
    // across revision switches and is what makes a folder a slice.
    write(&dir.join(".nabu.toml"), &format!("{shown}/.nabu.toml"), "")?;
    Ok(shown)
}

fn create_dir(dir: &Path, shown: &str, with_parents: bool) -> Result<()> {
    let made = if with_parents {
        fs::create_dir_all(dir)
    } else {
        fs::create_dir(dir)
    };
    made.map_err(|err| {
        Error::new(
            format!("cannot create {shown}: {err}"),
            "check the folder permissions and rerun",
        )
    })
}

fn write(path: &Path, shown: &str, text: &str) -> Result<()> {
    fs::write(path, text).map_err(|err| {
        Error::new(
            format!("cannot write {shown}: {err}"),
            "check the folder permissions and rerun",
        )
    })
}
