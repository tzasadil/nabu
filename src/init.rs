//! `nabu init`: installs the skills and agents embedded in the binary, so they always match the CLI.
//! Runs before the root and the layers load: global init needs no root, and a malformed config
//! must never block the rerun that repairs an install.
//! Unix only: scripts get the executable bit and `.claude/skills` is a symlink.

use crate::error::{Error, Result};
use crate::root;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};

include!(concat!(env!("OUT_DIR"), "/embedded.rs"));

const STARTER_CONFIG: &str = include_str!("starter-config.toml");

/// Where to install. Global takes the environment as values, so `main` owns every env read.
pub enum Scope {
    Global {
        home: Option<OsString>,
        xdg_config_home: Option<OsString>,
    },
    Local {
        cwd: PathBuf,
    },
}

/// Where one install lands. Global and local differ only in these paths.
struct Plan {
    skills: PathBuf,
    /// Where `.claude/skills` should link, and what the link says.
    claude_skills: PathBuf,
    claude_skills_target: PathBuf,
    agents: PathBuf,
    config: PathBuf,
    /// Folders that must not be links: init would write or delete through them.
    no_links: Vec<PathBuf>,
    /// Printed paths are relative to this folder (local) or absolute (none).
    shown_from: Option<PathBuf>,
}

impl Plan {
    fn global(home: Option<OsString>, xdg_config_home: Option<OsString>) -> Result<Self> {
        // Checked before any write: a half install under a guessed folder is worse than none.
        let home = home.filter(|home| !home.is_empty()).ok_or_else(|| {
            Error::new(
                "HOME is not set",
                "set HOME to your home folder and rerun nabu init",
            )
        })?;
        let config = root::global_dir(xdg_config_home, Some(home.clone()))
            .expect("HOME is set")
            .join("config.toml");
        let home = PathBuf::from(home);
        let skills = home.join(".agents/skills");
        let agents = home.join(".claude/agents");
        Ok(Self {
            // Only the leaves: a user-managed `~/.agents` or `~/.claude` link (dotfiles) is intended.
            no_links: vec![skills.clone(), agents.clone()],
            skills,
            claude_skills: home.join(".claude/skills"),
            claude_skills_target: home.join(".agents/skills"),
            agents,
            config,
            shown_from: None,
        })
    }

    fn local(root: &Path) -> Self {
        Self {
            // Parents first, so the error names the link that leads out of the repo.
            no_links: [
                ".agents",
                ".claude",
                "nabu",
                ".agents/skills",
                ".claude/agents",
            ]
            .map(|rel| root.join(rel))
            .to_vec(),
            skills: root.join(".agents/skills"),
            claude_skills: root.join(".claude/skills"),
            claude_skills_target: PathBuf::from("../.agents/skills"),
            agents: root.join(".claude/agents"),
            config: root.join("nabu/config.toml"),
            shown_from: Some(root.to_path_buf()),
        }
    }

    fn show(&self, path: &Path) -> String {
        let shown = match &self.shown_from {
            Some(root) => path.strip_prefix(root).unwrap_or(path),
            None => path,
        };
        shown.display().to_string()
    }
}

/// Installs every embedded skill and agent plus a starter config when missing.
/// Returns one line per installed path, then notices, then the config line.
pub fn run(scope: Scope) -> Result<String> {
    let plan = match scope {
        Scope::Global {
            home,
            xdg_config_home,
        } => Plan::global(home, xdg_config_home)?,
        Scope::Local { cwd } => Plan::local(&root::find_root(&cwd)?),
    };
    refuse_links(&plan)?;
    install(&plan, EMBEDDED)
}

/// Writing through a link would change the folder it points at. In this repo both
/// `.agents/skills` and `.claude/agents` link to the sources, which init must never overwrite.
fn refuse_links(plan: &Plan) -> Result<()> {
    for folder in &plan.no_links {
        let is_link = fs::symlink_metadata(folder).is_ok_and(|meta| meta.is_symlink());
        if is_link {
            let shown = plan.show(folder);
            return Err(Error::new(
                format!("{shown} is a symlink, and init would write through it"),
                format!("replace {shown} with a real folder, or run nabu init elsewhere"),
            ));
        }
    }
    Ok(())
}

fn install(plan: &Plan, files: &[(&str, &[u8])]) -> Result<String> {
    let mut lines = Vec::new();
    for name in skill_names(files) {
        let target = plan.skills.join(name);
        remove_existing(&target)?;
        let prefix = format!("skills/{name}/");
        for (rel, bytes) in files.iter().filter(|(rel, _)| rel.starts_with(&prefix)) {
            write_file(&target.join(&rel[prefix.len()..]), bytes)?;
        }
        lines.push(plan.show(&target));
    }
    let notice = link_skills(plan)?;
    for (rel, bytes) in files {
        if let Some(file) = rel.strip_prefix("agents/") {
            let target = plan.agents.join(file);
            remove_existing(&target)?;
            write_file(&target, bytes)?;
            lines.push(plan.show(&target));
        }
    }
    lines.extend(notice);
    lines.push(write_starter_config(plan)?);
    Ok(lines.join("\n"))
}

/// `dedup` drops only neighbors. It is enough because `build.rs` walks in sorted order, so each
/// skill's files sit together in `EMBEDDED`.
fn skill_names<'a>(files: &[(&'a str, &[u8])]) -> Vec<&'a str> {
    let mut names: Vec<&str> = files
        .iter()
        .filter_map(|(rel, _)| rel.strip_prefix("skills/")?.split('/').next())
        .collect();
    names.dedup();
    names
}

/// A symlink is unlinked, never followed: its target is not ours to delete.
/// No merge => files a skill dropped never linger.
fn remove_existing(target: &Path) -> Result<()> {
    let result = match fs::symlink_metadata(target) {
        Ok(meta) if meta.is_dir() => fs::remove_dir_all(target),
        Ok(_) => fs::remove_file(target),
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(err) => Err(err),
    };
    result.map_err(|err| io_error("remove", target, &err))
}

fn write_file(path: &Path, bytes: &[u8]) -> Result<()> {
    let write = || -> io::Result<()> {
        fs::create_dir_all(path.parent().expect("installed files have a folder"))?;
        fs::write(path, bytes)?;
        // The shebang marks the scripts. It also catches the sourced `lib.sh`, which is harmless.
        if bytes.starts_with(b"#!") {
            fs::set_permissions(path, fs::Permissions::from_mode(0o755))?;
        }
        Ok(())
    };
    write().map_err(|err| io_error("write", path, &err))
}

/// Only a missing `.claude/skills` becomes a link. A real folder may hold skills nabu must not
/// touch, so it stays (merging is in the backlog). The notice appears only when the existing entry
/// does not already lead to the installed skills.
fn link_skills(plan: &Plan) -> Result<Option<String>> {
    let link = &plan.claude_skills;
    match fs::symlink_metadata(link) {
        Ok(_) => {
            let same = matches!(
                (fs::canonicalize(link), fs::canonicalize(&plan.skills)),
                (Ok(a), Ok(b)) if a == b
            );
            Ok((!same).then(|| {
                format!(
                    "notice: {} is not a link to {}, left as is",
                    plan.show(link),
                    plan.show(&plan.skills)
                )
            }))
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            let create = || -> io::Result<()> {
                fs::create_dir_all(link.parent().expect("link has a folder"))?;
                symlink(&plan.claude_skills_target, link)
            };
            create().map_err(|err| io_error("link", link, &err))?;
            Ok(None)
        }
        Err(err) => Err(io_error("read", link, &err)),
    }
}

/// `create_new` makes the "only when missing" check atomic. A malformed config also counts as kept.
fn write_starter_config(plan: &Plan) -> Result<String> {
    let path = &plan.config;
    let created = || -> io::Result<bool> {
        fs::create_dir_all(path.parent().expect("config has a folder"))?;
        let file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path);
        match file {
            Ok(mut file) => {
                io::Write::write_all(&mut file, STARTER_CONFIG.as_bytes())?;
                Ok(true)
            }
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => Ok(false),
            Err(err) => Err(err),
        }
    };
    let state = if created().map_err(|err| io_error("write", path, &err))? {
        "written"
    } else {
        "kept"
    };
    Ok(format!("config: {} {state}", plan.show(path)))
}

/// Install is idempotent, so the first error stops the run and a rerun heals it.
fn io_error(verb: &str, path: &Path, err: &io::Error) -> Error {
    Error::new(
        format!("cannot {verb} {}: {err}", path.display()),
        "rerun nabu init",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime_state::is_runtime_state;
    use crate::test_util::walk_files;

    #[test]
    fn embedded_list_matches_the_folders_on_disk() {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut on_disk = Vec::new();
        for top in ["skills", "agents"] {
            walk_files(&manifest.join(top), top, &mut on_disk);
        }
        on_disk.retain(|rel| !is_runtime_state(rel));
        on_disk.sort();
        let mut embedded: Vec<&str> = EMBEDDED.iter().map(|(rel, _)| *rel).collect();
        embedded.sort();
        assert_eq!(on_disk, embedded);
        for (rel, bytes) in EMBEDDED {
            assert_eq!(&fs::read(manifest.join(rel)).unwrap(), bytes, "{rel}");
        }
    }

    #[test]
    fn a_symlinked_skill_file_is_embedded_as_its_target() {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let link = manifest.join("skills/nabu-verify/implementer-rules.md");
        assert!(fs::symlink_metadata(&link).unwrap().is_symlink());
        let target =
            fs::read(manifest.join("skills/with-implementer/implementer-rules.md")).unwrap();
        let (_, bytes) = EMBEDDED
            .iter()
            .find(|(rel, _)| *rel == "skills/nabu-verify/implementer-rules.md")
            .expect("the link is embedded");
        assert_eq!(*bytes, target.as_slice());
    }
}
