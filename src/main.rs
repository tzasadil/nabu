mod archive;
mod config;
mod error;
mod init;
mod instructions;
mod layers;
mod list;
mod migrate;
mod new;
mod prompt;
mod report;
mod root;
#[cfg(test)]
mod runtime_state;
mod schema;
mod status;
mod table;
mod tablets;
#[cfg(test)]
mod test_util;
mod toml_file;

use clap::{Parser, Subcommand};
use config::Config;
use error::Result;
use layers::Layers;
use std::process::ExitCode;

#[derive(Parser)]
#[command(version, about = "Spec-driven planning folders for jj repos")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create a tablet, or a slice in a stacked tablet
    New {
        /// `<tablet>` or `<tablet>/<slice>`
        target: String,
        /// Also write the schema's stack template as `stack.md`
        #[arg(long)]
        stack: bool,
    },
    /// Show artifact states and task progress of a tablet or one slice
    Status {
        /// `<tablet>` or `<tablet>/<slice>`
        target: String,
    },
    /// List active tablets with task progress
    List,
    /// Print the prompt text for one step: an artifact, `build`, or `stack`
    Instructions {
        /// An artifact id of the tablet's schema, or the word `build` or `stack`
        what: String,
        /// `<tablet>` or `<tablet>/<slice>`; for `stack`, `<tablet>`
        target: String,
    },
    /// Move a finished tablet to `nabu/archive/YYYY-MM-DD-<tablet>/`
    Archive {
        /// `<tablet>`; the whole tablet moves, stack and slices included
        target: String,
    },
    /// Print the prompt that tells an agent how to convert a spec folder to `nabu/`
    Migrate,
    /// Install the skills and agents built into this binary
    Init {
        /// Install into the home folder (the default)
        #[arg(long, conflicts_with = "local")]
        global: bool,
        /// Install into the current repo only
        #[arg(long)]
        local: bool,
    },
}

/// What a command prints on stdout and whether it counts as failed.
/// Separate from `Err`: `list` prints every row and still exits 1 when a row is an error.
pub struct Outcome {
    pub text: String,
    pub failed: bool,
}

impl Outcome {
    pub fn ok(text: String) -> Self {
        Self {
            text,
            failed: false,
        }
    }
}

fn current_dir() -> Result<std::path::PathBuf> {
    std::env::current_dir().map_err(|err| {
        error::Error::new(
            format!("cannot read the current folder: {err}"),
            "run from an existing folder",
        )
    })
}

fn run(cli: Cli) -> Result<Outcome> {
    // Before the root and the layers: global init needs no root, and a malformed config must not block the repair.
    // Global init also skips the current folder, so it works from a deleted one.
    if let Command::Init { local, .. } = cli.command {
        let scope = if local {
            init::Scope::Local {
                cwd: current_dir()?,
            }
        } else {
            init::Scope::Global {
                home: std::env::var_os("HOME"),
                xdg_config_home: std::env::var_os("XDG_CONFIG_HOME"),
            }
        };
        return init::run(scope).map(Outcome::ok);
    }
    let root = root::find_root(&current_dir()?)?;
    let global = root::global_dir(
        std::env::var_os("XDG_CONFIG_HOME"),
        std::env::var_os("HOME"),
    );
    // Before the layers load: a broken `nabu/` left by a failed attempt must not block the retry.
    if matches!(cli.command, Command::Migrate) {
        return Ok(Outcome::ok(migrate::run(&root)));
    }
    let layers = Layers::load(&root, global.as_deref())?;
    // Parsed before dispatch so a malformed config fails every command, not only those that read it.
    let config = Config::load(&layers)?;
    match cli.command {
        Command::New { target, stack } => {
            new::run(&root, &layers, &config, &target, stack).map(Outcome::ok)
        }
        Command::Status { target } => status::run(&root, &layers, &target).map(Outcome::ok),
        Command::List => list::run(&root, &layers),
        Command::Instructions { what, target } => {
            instructions::run(&root, &layers, &config, &what, &target).map(Outcome::ok)
        }
        Command::Archive { target } => archive::run(&root, &target).map(Outcome::ok),
        Command::Migrate | Command::Init { .. } => {
            unreachable!("dispatched before the layers load")
        }
    }
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(outcome) => {
            println!("{}", outcome.text);
            if outcome.failed {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
    }
}
