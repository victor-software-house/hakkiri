# Proposal

Queue row: `HKR-030` in [tasks.yaml][ledger].
It follows [`gitignore-walk`][walk], which chose the files a run reads, and changes [check-command][check] and [project-config][config].

## Motivation

hakkiri reads Markdown, MDX and the comments of three source languages, and skips every other file a glob selects.
`hakkiri check notes.txt` is a failure, and a `*` glob lints less than it names.
Vale reads such a file as plain text, and its `[formats]` section maps an extension to a reader.
A project that checks `README`, `LICENSE`, `.txt` and `.toml` files with Vale cannot switch while hakkiri refuses them (operator, 2026-11-11).

## Changes

- A file that a glob selects and that no reader reads is plain text: one `raw` root over the file, with a paragraph for each run of lines between blank lines.
- `formats` maps an extension, a file name or a glob to a reader (`md`, `mdx`, `rs`, `ts`, `tsx`, `js`, `text`), as Vale's `[formats]` does, and a package's settings file may hold it.
- `on_unknown` (`--on-unknown`, `HAKKIRI_ON_UNKNOWN`) takes `text`, the default, `warn`, `error` or `ignore`, for a selected file that no reader reads.
- A binary file is skipped with no message.
- The refusal of a named file that no language reads goes, and so does the gap scenario for it in the `vale-differences` spec.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `check-command`: the reader of a file, plain text, and binary files.
- `project-config`: `formats` and `on_unknown` are keys.
- `package-settings`: a package's file may hold `formats`, and may not hold `on_unknown`.
- `vale-differences`: the plain-text gap closes, and three differences stay.

## Impact

- New: `src/plain.rs` and `src/formats.rs`.
- Changed: `Language`, the file finder, the config loader, the command line, `src/instructions.md`, the skill, `AGENTS.md` and `README.md`.
- No new dependency.

[ledger]: ../../../tasks.yaml
[walk]: ../gitignore-walk/proposal.md
[check]: ../../specs/check-command/spec.md
[config]: ../../specs/project-config/spec.md
