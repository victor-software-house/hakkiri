# Proposal

Queue row: `HKR-005` in [tasks.yaml][ledger].  
It is the first of three changes that ship the command line: this one adds `check`, [`test-command`][test] adds `test`, and [`release-lane`][release] ships the binary.

## Motivation

hakkiri reads Markdown, MDX and source comments and runs every check type, but its binary is empty (`fn main() {}`).  
`hakkiri check` is what the `lint:prose` task, CI and a pre-commit hook run, so no project can leave Vale until it exists.  
The project config, `.ctl/hakkiri.yaml`, is specified and not yet read.  
A project also needs its styles from somewhere before packages arrive in `HKR-006`.

## Changes

- `hakkiri check` lints the files it is given, or the current directory, in parallel, through the `files` globs of the project config, and prints one line per alert or a JSON report.
- Its exit status is 0 for a clean run, 1 when an alert reaches `--fail-level`, and 2 when anything could not be run.
- A `packages` entry may name a local `path` that holds style folders.
  `HKR-006` adds the entry that fetches by URL and hash.
- `hakkiri instructions` prints the operator instructions, and the repository gains its skill.
- The `vale-differences` spec lists where the command line differs from Vale.

## Capabilities

### New Capabilities

- `check-command`: which files `check` lints, what it prints, and how it exits.

### Modified Capabilities

- `project-config`: a package is a local folder or a download.
- `vale-differences`: the command line requirement is added, and one known gap.

## Impact

- New direct dependencies: `clap`, `anyhow` and `globset`; `serde_json` for tests.
  The design names why each is needed and what it costs in binary size.
- `Cargo.toml`, `src/main.rs` and new modules for the command, the config and the file walk.
- `README.md`, `AGENTS.md`, `src/instructions.md` and `skills/hakkiri/SKILL.md` move with the code.
- New rows `HKR-028` (`test`), `HKR-029` (release) and `HKR-030` (plain-text files).

[ledger]: ../../../tasks.yaml
[test]: ../test-command/proposal.md
[release]: ../release-lane/proposal.md
