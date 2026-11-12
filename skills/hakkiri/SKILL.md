---
name: hakkiri
description: >-
  Run the hakkiri prose linter on Markdown, MDX and source comments: hakkiri
  check, hakkiri test, the project config .ctl/hakkiri.yaml, rule packages,
  rule test cases, alert levels and exit codes. Use when a repository has .ctl/hakkiri.yaml, when a lint:prose
  task or a pre-commit hook runs hakkiri, or when the user mentions hakkiri,
  prose lint alerts or ProseStyle.
license: AGPL-3.0-or-later
version: 0.0.6
compatibility: Requires a hakkiri binary on the path.
---

# hakkiri

`hakkiri check` lints Markdown, MDX and the comments of Rust, TypeScript and
JavaScript files under `.ctl/hakkiri.yaml`.
`hakkiri test` runs the test cases of rule files.
`hakkiri instructions` and `--help` are the contract of the installed version.
This skill is the workflow and does not repeat the whole grammar.

## Run a check

```sh
hakkiri check
hakkiri check docs README.md
hakkiri check --fail-level warning
hakkiri check --no-gitignore
hakkiri check --on-unmatched error README.md
hakkiri check --on-unknown warn
hakkiri check --format json
```

With no path, `check` walks the current directory.
A walk skips `node_modules`, `.git` and package folders, and what git ignores in a git repository.
`--no-gitignore` reads the ignored files, and a file named on the command line is read whether git ignores it or not.
A file named on the command line that no glob selects draws a warning, and `--on-unmatched error` fails the run for it.
A file that a glob selects and that no reader reads is linted as plain text, and a binary file, with a NUL byte or bytes that are not UTF-8 at its start, is skipped.
`formats` in the config maps an extension, a file name or a glob to a reader, and `--on-unknown` turns the plain-text default into a warning, a failure or nothing.

## Run the test cases

```sh
hakkiri test
hakkiri test --coverage
hakkiri test styles/Team/Filler.yml
```

A rule file holds cases under `tests`, and a `*.test.yml` file is a list of cases.
Each case has a `name`, an `input`, and one of `want`, `contains` and `absent`.
A failed case prints as `failed <file>: <name>` with its reason and the alerts it got.
`--coverage` also fails for a rule that no case makes flag anything.
`test` exits 0 when every case passes, 1 when a case fails or a rule is uncovered, and 2 when a case could not run or no case was found.

## Read the result

Each alert is `path:line:column: level Rule: message`, and the last line counts the alerts and the files.
Failures print as `failed <path>: <message>`.
Use `--format json` to read the same result as one object with `alerts`, `failures` and `files`.
JSON never holds an escape byte.

| Exit | Meaning |
|:--|:--|
| 0 | No reported alert reaches `--fail-level` |
| 1 | A reported alert reaches `--fail-level`, which defaults to `error` |
| 2 | The config or a rule is wrong, a file failed, no path lies under the project root, or the command line is wrong |

## Read the tags of a sequence rule

A `sequence` rule reads part-of-speech tags, and hakkiri's are not Vale's.
It tags with the model of prose at `b33e60d` and corrects a possessive `'s`, a modal or a name in capitals, a semicolon, and a verb after a noun phrase.
So a `sequence` rule can flag other words than in Vale.
A code span is no word of a `sequence` run, so a rule of a noun and a verb flags no code name, but the noun that a preposition attaches the span to, as in `the notice for `cmd-7` says`.
`hakkiri instructions` lists the difference.

## Flags and variables

Every long flag reads `HAKKIRI_<LONG>`. A flag wins over its variable, and an empty variable counts as unset.
`--fail-level`, `--gitignore`, `--on-unmatched` and `--on-unknown` are also keys of the project config, `fail_level`, `gitignore`, `on_unmatched` and `on_unknown`, which the variable wins over.

| Flag | Variable | Meaning |
|:--|:--|:--|
| `--cache-dir` | `HAKKIRI_CACHE_DIR` | The folder that keeps fetched packages, the per-user cache folder `hakkiri` by default |
| `--config` | `HAKKIRI_CONFIG` | The config file, `.ctl/hakkiri.yaml` by default |
| `--coverage` | `HAKKIRI_COVERAGE` | With `test`, fail when a rule flags nothing in any case |
| `--fail-level` | `HAKKIRI_FAIL_LEVEL` | The lowest level that exits 1: `suggestion`, `warning` or `error` |
| `--gitignore` | `HAKKIRI_GITIGNORE` | `true`, the default, or `false` to read what git ignores; `--no-gitignore` is the same as `--gitignore=false` and has no variable |
| `--on-unmatched` | `HAKKIRI_ON_UNMATCHED` | What to do with a named file no glob selects: `warn`, the default, `error` or `ignore` |
| `--on-unknown` | `HAKKIRI_ON_UNKNOWN` | What to do with a selected file no reader reads: `text`, the default, `warn`, `error` or `ignore` |
| `--format` | `HAKKIRI_FORMAT` | `pretty` or `json` |
| `--color` | `HAKKIRI_COLOR` | `auto`, `always` or `never`; pretty output holds no colour |
| `--quiet` | `HAKKIRI_QUIET` | Hide successful pretty output |
| `--no-color` | none | The same as `--color never`, and it wins over `--color` |

## Fix a failing run

1. Exit 2 with a config error: the message names the file, the line and the key. Fix the config first, because no file was linted.
   A message that names a package `url` means the download failed a check: the server did not answer, the zip does not match its `sha256`, or it is not a package.
   Check the `url` and the `sha256`, and set `--cache-dir` when the default cache folder is read-only.
   A message that names `block_ignores` or `token_ignores` means a pattern does not compile, or the value is not a list of patterns.
   A message that names a `vocab` entry means no package holds that vocabulary, two packages hold it, or a term in its lists is not a pattern.
   Alerts from `Vale.Terms` and `Vale.Avoid` come from a vocabulary: fix the word, or move the term between `accept.txt` and `reject.txt`.
   A message that names `<package>/.ctl/hakkiri.yaml` means a package ships settings with that fault: ask the package's owner, or pin another version of it.
   A folder named `.unpacking-` and a suffix in the cache is left by a killed run: delete it when no hakkiri run is using the cache.
2. Exit 2 with `failed <path>`: that file could not be read or parsed. The other files were linted.
   Exit 2 with `no path given lies under the project root`: the config names another root than the files you gave. Check `--config` and `HAKKIRI_CONFIG`.
   Exit 2 with a `failed` line that names a rule file and a rule: the rule is a Vale script rule or a `doc()` scope, which hakkiri cannot run. The rest of the style ran. Set the rule `off` under `files` to leave it out.
3. `test` exits 1 with `failed <file>: <name>`: the rule or the case is wrong. With `uncovered <Rule>`, add a case that makes the rule flag something.
4. Exit 1: fix the alerts, or lower the rule's level under `files` when the rule is too strict for those files.
5. An alert that looks wrong is a rule fault. Set the rule `off` for those files in the config, and say why in the pull request.
   To skip one stretch of a Markdown file, put `<!-- hakkiri Style.Rule: off -->` before it and `<!-- hakkiri Style.Rule: on -->` after it. `<!-- hakkiri off -->` and `<!-- hakkiri on -->` skip every rule.

The config keys are in `hakkiri instructions`.
