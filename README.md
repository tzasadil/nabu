<p align="center"><img src="assets/logo.svg" width="128" alt="A reed stylus resting on a clay tablet with cuneiform wedges"></p>

# nabu

**Nabu** /ˈnɑːbuː/ (NAH-boo)

*noun.* The Mesopotamian god of writing, scribes, and wisdom. Son of Marduk. Keeper of the Tablet of Destinies, where he wrote down the fate of gods and people.

---

A small CLI for spec-driven development, built around jj and stacks. Heavily inspired by [OpenSpec](https://github.com/Fission-AI/OpenSpec).

Work is planned in **tablets**. A tablet is one folder that holds the artifacts for one piece of work (`blueprint.md`, `tasks.md`). The name avoids a clash with jj, where a "change" is a revision.

Tablets live in `nabu/tablets/<name>/`. Archived tablets move to `nabu/archive/YYYY-MM-DD-<name>/`.

## Install

The binary embeds `skills/` and `agents/`, so the installed copies always match the CLI. crates.io has another `nabu` => install from git or a checkout.

```sh
cargo install --git https://github.com/tzasadil/nabu   # or: cargo install --path .
nabu init            # same as --global
nabu init --local    # inside a jj repo
```

- `nabu init` installs into your home: skills in `~/.agents/skills/`, agents in `~/.claude/agents/`, and a link `~/.claude/skills` when missing. It writes a starter `~/.config/nabu/config.toml` when missing.
- `nabu init --local` installs into the current repo only: `.agents/skills/`, `.claude/agents/`, a link `.claude/skills`, and a starter `nabu/config.toml` when missing. Nothing outside the repo changes. The global config still applies.
- A rerun replaces every skill and agent nabu ships. Other skills and any existing config stay as they are.
- Init refuses to write through a symlinked skills or agents folder. This repo links them to its sources, so run `--local` in other repos.

## Goals

- **jj native.** Built for [jj](https://github.com/jj-vcs/jj). Git is not required.
- **Stacks first.** A tablet can hold a `stack.md`. The stack splits the tablet into slices, one subfolder per slice. The `stack.md` template lives with the other templates (`blueprint.md`, `tasks.md`).
- **Fully configurable.** Artifacts, templates, and rules come from config.
- **Global settings and templates.** Set them once in `~/.config/nabu/`. A repo overrides only what differs.
- **Self-learning.** `nabu-learn` mines past sessions for repeated corrections and friction. It turns them into architecture and reviewer rules, plus ranked process improvements. Every rule cites the sessions it came from.
- **Templates in one place.** Skills get the current template from `nabu instructions`. No skill holds a copy of the default template.

## Extras

- **Backlog.** Each tablet keeps a `backlog.md` of deferred findings. The backlog never widens scope and never counts toward progress.

## What nabu avoids

- Hundreds or thousands of lines of spec before implementation starts. Such specs fill up with implementation details => no human can review them. They cost time and tokens and add nothing.
- WHEN/THEN scenario catalogues that state a simple contract in a bloated form.
- Bloated framework skills that force a heavy workflow (for example TDD) onto every small task.
- Git only.
- Commits after every step.
- No support for stacks.
- Default templates copied into every skill => one template edit means editing every skill.

## License

MIT
