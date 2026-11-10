# Design

## Context

The engine lints one file: `parse` gives a tree, and `lint` returns its alerts.
Rules load from a style folder with `load_style`.
Nothing yet reads `.ctl/hakkiri.yaml`, finds files, runs files in parallel, prints alerts or sets an exit status.
The [rule-format design][format] fixed the config's keys, and the [bootstrap design][bootstrap] fixed `check` and `test` as release 1.

## Goals / Non-Goals

Goals:

- A command the `lint:prose` task, CI and a pre-commit hook can run today in place of `vale --output=line`.
- Behaviour that matches Vale's command line where nothing argues for a difference, and a scenario for each difference.
- Presentation owned by ctl-core, so the line form and the JSON come from one result.

Non-goals:

- Fetching packages (`HKR-006`), vocabularies (`HKR-019`), the importer (`HKR-007`) and `hakkiri test` (`HKR-028`).
- Linting plain-text files, which the tree does not read yet (`HKR-030`).
- A language server and any editor protocol.

## Decisions

### 1. One binary on ctl-core's App

`check`, `test` and `instructions` are subcommands of one parser on `ctl_core::App`, with `#[command(env_prefix = "HAKKIRI")]`.
Every long flag reads `HAKKIRI_<LONG>`, shared flags included, as ctl's rule 7 asks.
ctl-core owns `--format`, `--color` and `--quiet`, the help and the JSON error envelope.
A Vale flag that `check` lacks, such as `--glob`, `--filter`, `--ext`, `--sort`, `--relative` and `--no-global`, is unknown to the parser, and hakkiri has no stdin mode.
The handler returns data and never prints.

Alternative: `clap` and hand-written output.
Lost: ctl-core already owns presentation for the other ctl CLIs, and JSON that never holds ANSI is its contract.

### 2. Paths are files or directories

An argument is a file, linted as given, or a directory, walked in sorted order.
No argument means the current directory.
The walk skips `node_modules`, `.git` and each folder a `packages` entry names, because a style folder holds rules and not prose.
Vale skips the first two and its styles path ([the walk][v-walk], [the ignored names][v-ignore]).
A dot directory is otherwise walked, so `.github` prose is linted when a glob matches it.

Alternative: honour `.gitignore` through the `ignore` crate.
Lost: Vale does not, the `lint:prose` task already lists `git ls-files`, and a config glob such as `'target/**': {styles: []}` states the same skip in the file that owns it.

### 3. Globs and paths are relative to the current directory

A `files` glob matches the path as written relative to the current directory, with `/` separators.
`*` crosses `/`, so `'*.rs'` matches `src/a.rs`, as in Vale, and `**` and `{a,b}` work.
A `packages` `path` is relative to the current directory too.
The project-config spec already reads the config from the current directory.

Alternative: walk up to find `.ctl/`.
Lost: a run from a subdirectory would lint other files than a run from the root, and the spec fixes the current directory.

### 4. A package is a folder of style folders

A `packages` entry holds a `path` to a directory whose subfolders are styles, each with one `.yml` file per rule.
The style name is the subfolder, and `styles: [ProseStyle]` in a glob selects it.
`HKR-006` adds the entry that holds `url` and `sha256`, which unpacks to the same shape.
Two entries that hold one style name are refused, and the error names both.

A local path is needed beyond this change: a style repository runs `hakkiri test` over its own folder.

Alternative: a top-level `styles_path` key, Vale's `StylesPath`.
Lost: it adds a second place that holds styles beside `packages`, and a project would carry both after `HKR-006`.

### 5. A language is read by extension

`.md` and `.mdx`, `.rs`, `.ts`, `.tsx` and `.js` select Markdown, MDX, Rust, TypeScript, TSX and JavaScript.
A directory walk skips any other file with no message, so a `'*'` glob does not fail on an image.
A file named as an argument that no language reads is an error that names it.

Vale reads any other file as plain text, so this is a known gap, not a difference ([row `HKR-030`][ledger]).

### 6. Files lint in parallel, output stays ordered

A pool of `available_parallelism` threads takes files from a shared index, so no queue crate is needed.
Each file's alerts are kept with its path, and the report sorts by path, then line, then column, so two runs print the same text.
A file that cannot be read or parsed is recorded as a failure, and the other files still lint.
Vale reports such documents after the alerts of the rest ([exit][v-exit]).

### 7. Output is a line per alert, or JSON

The pretty form prints one line per alert and then a count:

```text
docs/guide.md:12:5: warning ProseStyle.Filler: Remove 'very'.
1 alert in 3 files
```

The level word takes the colour of its level when colour is on.
`--format json` prints one object: `alerts`, each with `path`, `line`, `column`, `end_column`, `level`, `rule`, `message`, `description`, `link` and `match`; `failures`, each with `path` or `config` and `message`; and `files`, the count of files linted.
Both forms are one result that ctl-core renders, so they cannot differ.

Vale's `--output=line` prints `path:line:col:Rule:message`, without the level, and its JSON is a map from path to alerts ([the JSON][v-json]).
The level is added because `--fail-level` decides the exit by it.

Alternative: a third `--format line`.
Lost: ctl-core's two formats are shared by every ctl CLI, and the pretty form of a linter is already a line per finding.

### 8. Exit status follows Vale's, with a level flag

The exit status is 2 when anything could not run: a config or rule error, or a file that failed.
Otherwise it is 1 when a reported alert reaches `--fail-level`, which is `suggestion`, `warning` or `error` and defaults to `error`, and 0 when none does.
The config's `level` hides alerts below it first, so `--fail-level` sees only what is reported.
This is Vale's order, 2 before 1 ([exit][v-exit]); `--fail-level` takes the place of `--no-exit` and of a template that greps its output.

A config or rule error stops the run before any file lints, prints to stderr, and exits 2, where Vale's handler also exits 2.
A usage error exits 2 through ctl-core, as clap does.

Alternative: ctl-core's default of 1 for a handler error.
Lost: a caller could not tell a failing check from a broken setup.

### 9. Dependencies

| Crate | Why | Alternative lost |
|:--|:--|:--|
| `clap` 4, `derive` and `env` | The derive macros expand to `clap::` paths, so a program names it, as ctl-core's prelude says. ctl-core's `app` feature pulls it in, so the graph gains no crate. | None. |
| `anyhow` | `App::run` takes a handler that returns `anyhow::Result`; ctl-core does not re-export it, and its `app` feature pulls it in. | A local error type that implements the same trait. |
| `globset` | `files` globs need `**` and `{a,b}`, and its `*` crosses `/` by default. | A hand-written matcher. |
| `serde_json`, dev only | Tests read the JSON output. | Comparing strings. |

ctl-core gains the `app` feature, and `fixture` in dev for temporary project folders.
The change states the release binary size before and after, measured on a box, because CI cannot.

### 10. Instructions are a file

`src/instructions.md` holds the operator instructions and `hakkiri instructions` prints it with `include_str!`.
`skills/hakkiri/SKILL.md` holds the usage skill.
A test compares the command's output with the file, so the two cannot drift.

Alternative: render both from ctl-core templates.
Lost: it needs `minijinja` and a surface build for one command, and the text stays short.

## Risks / Trade-offs

- [A walk of a large tree reads every file name once] → A glob match runs one compiled set; a project that needs less lists its files, as the task does.
- [Exit 2 for a broken setup differs from ctl-core's default] → The scenarios and the skill state the three codes, and a test runs each.
- [The count line is an extra line] → A caller that parses lines reads `--format json`; the exit status says clean or not.

[format]: ../archive/2026-11-10-rule-format/design.md
[bootstrap]: ../archive/2026-11-09-bootstrap-hakkiri/design.md
[ledger]: ../../../tasks.yaml
[v-walk]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/lint.go#L283-L333
[v-ignore]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/util.go#L14-L16
[v-exit]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/cmd/vale/main.go#L186-L198
[v-json]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/cmd/vale/json.go#L46-L53
