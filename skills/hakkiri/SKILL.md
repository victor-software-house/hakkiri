---
name: hakkiri
description: >-
  Run the hakkiri prose linter on Markdown, MDX and source comments: hakkiri
  check, the project config .ctl/hakkiri.yaml, rule packages, alert levels and
  exit codes. Use when a repository has .ctl/hakkiri.yaml, when a lint:prose
  task or a pre-commit hook runs hakkiri, or when the user mentions hakkiri,
  prose lint alerts or ProseStyle.
license: AGPL-3.0-or-later
version: 0.0.0
compatibility: Requires a hakkiri binary on the path.
---

# hakkiri

`hakkiri check` lints Markdown, MDX and the comments of Rust, TypeScript and
JavaScript files under `.ctl/hakkiri.yaml`.
`hakkiri instructions` and `--help` are the contract of the installed version.
This skill is the workflow and does not repeat the whole grammar.

## Run a check

```sh
hakkiri check
hakkiri check docs README.md
hakkiri check --fail-level warning
hakkiri check --format json
```

With no path, `check` walks the current directory.
A walk skips `node_modules`, `.git` and package folders, and skips files that no language reads.
A file named on the command line that no language reads is a failure.

## Read the result

Each alert is `path:line:column: level Rule: message`, and the last line counts the alerts and the files.
Failures print as `failed <path>: <message>`.
Use `--format json` to read the same result as one object with `alerts`, `failures` and `files`.
JSON never holds an escape byte.

| Exit | Meaning |
|:--|:--|
| 0 | No reported alert reaches `--fail-level` |
| 1 | A reported alert reaches `--fail-level`, which defaults to `error` |
| 2 | The config or a rule is wrong, a file failed, or the command line is wrong |

## Flags and variables

Every long flag reads `HAKKIRI_<LONG>`. A flag wins over its variable, and an empty variable counts as unset.

| Flag | Variable | Meaning |
|:--|:--|:--|
| `--config` | `HAKKIRI_CONFIG` | The config file, `.ctl/hakkiri.yaml` by default |
| `--fail-level` | `HAKKIRI_FAIL_LEVEL` | The lowest level that exits 1: `suggestion`, `warning` or `error` |
| `--format` | `HAKKIRI_FORMAT` | `pretty` or `json` |
| `--color` | `HAKKIRI_COLOR` | `auto`, `always` or `never`; pretty output holds no colour |
| `--quiet` | `HAKKIRI_QUIET` | Hide successful pretty output |
| `--no-color` | none | The same as `--color never`, and it wins over `--color` |

## Fix a failing run

1. Exit 2 with a config error: the message names the file, the line and the key. Fix the config first, because no file was linted.
2. Exit 2 with `failed <path>`: that file could not be read or parsed. The other files were linted.
3. Exit 1: fix the alerts, or lower the rule's level under `files` when the rule is too strict for those files.
4. An alert that looks wrong is a rule fault. Set the rule `off` for those files in the config, and say why in the pull request.

The config keys are in `hakkiri instructions`.
