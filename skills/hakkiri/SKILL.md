---
name: hakkiri
description: >-
  Run the hakkiri prose linter on Markdown, MDX and source comments: hakkiri
  check, hakkiri test, the project config .ctl/hakkiri.yaml, rule packages,
  rule test cases, alert levels and exit codes. Use when a repository has .ctl/hakkiri.yaml, when a lint:prose
  task or a pre-commit hook runs hakkiri, or when the user mentions hakkiri,
  prose lint alerts or ProseStyle.
license: AGPL-3.0-or-later
version: 0.1.0
compatibility: Requires a hakkiri binary on the path.
---

# hakkiri

`hakkiri check` lints Markdown, MDX and the comments of Rust, TypeScript,
JavaScript, Go, Swift, YAML, TOML and Shell files under `.ctl/hakkiri.yaml`.
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
hakkiri check --code-span-size 0
hakkiri check --format json
```

With no path, `check` walks the current directory.
A walk skips `node_modules`, `.git` and package folders, and what git ignores in a git repository.
`--no-gitignore` reads the ignored files, and a file named on the command line is read whether git ignores it or not.
A file named on the command line that no glob selects draws a warning, and `--on-unmatched error` fails the run for it.
A file has the language of its path, or of the interpreter on its first line, and is read by the reader of its language: Markdown, MDX, Rust, TypeScript, TSX, JavaScript, Go, Swift, YAML, TOML, Shell or Text.
So `.yml`, `.bash` and a `#!/bin/sh` script have a reader.
A file that a glob selects and that no reader reads is linted as plain text, and a binary file is skipped. A byte that is not UTF-8, in a Latin-1 file for example, reads as one `?`, so a column still counts bytes.
The comments are all that a source, YAML, TOML or Shell file gives, read as Markdown, and a comment addressed to a tool, such as `eslint-disable` or `swiftlint:`, is left out.
`formats` in the config maps an extension, a file name or a glob to a reader, named by any name of its language, and `--on-unknown` turns the plain-text default into a warning, a failure or nothing.
A scope may end in a language, such as `text.comment.rs`, and the comments of a fenced code block in Rust, TypeScript, TSX, JavaScript, Go, Swift, YAML, TOML or Shell are linted as comments of that language.

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
It tags with the model of prose at `b33e60d` and corrects a possessive `'s`, a modal or a name in capitals, a semicolon, a verb after a noun phrase, and the verbs of saying and asking after a noun.
It also reads `check`, `prompt`, `thinking` and `fade` as nouns after a determiner.
It reads `prompt`, `round` and `split` as nouns in `the load split pull request`.
It reads `check` as a noun that ends a compound, as in `the peer user id check`.
So a `sequence` rule can flag other words than in Vale.
A code span is a noun in a `sequence` run, so a rule of a noun and a verb flags `` `flag` says `` as it flags `the flag says`, and a pattern for a lowercase word never takes the span.
An `existence` rule reads a code span as one `*`, as Vale masks it, so a match that runs across a span is a match, and one that starts or ends on a span is not.
`hakkiri instructions` lists the difference.

A `sentence` rule reads sentences that differ from Vale's in a few places.
A code span begins or ends a sentence next to a full stop, `vs.` ends none, a unit after a number ends one before a capital, and a numbered heading is one sentence.
`hakkiri instructions` lists them.

## Write a sequence rule with several forms

A `sequence` rule takes `forms`, a list of mappings that each hold `tokens`, where Vale takes one `tokens` list.
It matches where any form does and gives every alert the one rule name, so a check with three shapes is one rule.
Forms that match overlapping words give the match that starts first.
A rule holds `tokens` or `forms`, never both, and Vale stops with a panic on a rule with `forms`.
`hakkiri instructions` lists the difference.

## Flags and variables

Every long flag reads `HAKKIRI_<LONG>`. A flag wins over its variable, and an empty variable counts as unset.
`--fail-level`, `--gitignore`, `--on-unmatched` and `--on-unknown` are also keys of the project config, `fail_level`, `gitignore`, `on_unmatched` and `on_unknown`, which the variable wins over.
`--code-span-size` is also the key `code_span_size`, of the project config, a package's settings file and a glob under `files`, in words: a glob beats the top level, and the variable and the flag beat both. The project may force it with `!override`.

| Flag | Variable | Meaning |
|:--|:--|:--|
| `--cache-dir` | `HAKKIRI_CACHE_DIR` | The folder that keeps fetched packages, the per-user cache folder `hakkiri` by default |
| `--config` | `HAKKIRI_CONFIG` | The config file, `.ctl/hakkiri.yaml` by default |
| `--coverage` | `HAKKIRI_COVERAGE` | With `test`, fail when a rule flags nothing in any case |
| `--fail-level` | `HAKKIRI_FAIL_LEVEL` | The lowest level that exits 1: `suggestion`, `warning` or `error` |
| `--gitignore` | `HAKKIRI_GITIGNORE` | `true`, the default, or `false` to read what git ignores; `--no-gitignore` is the same as `--gitignore=false` and has no variable |
| `--on-unmatched` | `HAKKIRI_ON_UNMATCHED` | What to do with a named file no glob selects: `warn`, the default, `error` or `ignore` |
| `--on-unknown` | `HAKKIRI_ON_UNKNOWN` | What to do with a selected file no reader reads: `text`, the default, `warn`, `error` or `ignore` |
| `--code-span-size` | `HAKKIRI_CODE_SPAN_SIZE` | How many words a code span counts as in a word count and the metrics built on words: 1 by default, 0 as Vale counts it, up to 255 |
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
   A message that names a `vocab.yml` or `vocab.yaml` file and a line means an entry there is wrong. The entry has more than one key, an option that does not exist, a pattern that does not compile, or an `affix` file that no package holds.
   A message that names a line under `params` means a rule's parameter is wrong. The field is not one a parameter sets, the rule's check does not define it, its value is of the wrong kind, a flag is not `true` or `false`, or the rule does not exist.
   Alerts from `Vale.Repetition` mean a word repeats, as in `the the`: fix the text, accept the phrase in a vocabulary, set `Vale.Repetition: off` under the glob, or let more repeats pass with `params: {Vale.Repetition: {max: 1}}`.
   Alerts from `Vale.Spelling` mean a word is not in the built-in dictionary. Fix the word, accept it in a vocabulary or in a `.txt` file under a package's `config/ignore` folder, add a dictionary to its `config/dictionaries` folder, or set `Vale.Spelling: off` under the glob. A link under `config/ignore` that reaches outside the package is a fault. A fault in a dictionary stops a run only when a glob lists `Vale` in `styles`, names a rule that starts with `Vale.`, or sets `params` for one.
   Alerts from `Vale.Terms` and `Vale.Avoid` come from a vocabulary: fix the word, or move the term between the accepted and the rejected list, in `accept.txt` and `reject.txt` or in `vocab.yml`.
   A message that names `<package>/.ctl/hakkiri.yaml` means a package ships settings with that fault: ask the package's owner, or pin another version of it.
   Globs that match one file all apply, and the more specific glob wins for the same key, whatever the order written. At equal specificity the project beats a package. A value the project tags `!override` beats every value without the tag. A message about `!override` or another tag means the tag is in a package's file or is not `!override`: only the project's config forces a value.
   A `warning` that a package's `styles/Vale` is skipped means the package holds a style named for the built-in one. Its rules never run, as in Vale; the other styles of the package lint.
   A folder named `.unpacking-` and a suffix in the cache is left by a killed run: delete it when no hakkiri run is using the cache.
2. Exit 2 with `failed <path>`: that file could not be read or parsed. The other files were linted.
   Exit 2 with `no path given lies under the project root`: the config names another root than the files you gave. Check `--config` and `HAKKIRI_CONFIG`.
   Exit 2 with a `failed` line that names a rule file and a rule: the rule is a Vale script rule or a `doc()` scope, which hakkiri cannot run. The rest of the style ran. Set the rule `off` under `files` to leave it out.
3. `test` exits 1 with `failed <file>: <name>`: the rule or the case is wrong. With `uncovered <Rule>`, add a case that makes the rule flag something.
4. Exit 1: fix the alerts, or lower the rule's level under `files` when the rule is too strict for those files.
5. An alert that looks wrong is a rule fault. Set the rule `off` for those files in the config, and say why in the pull request.
   To skip one stretch of a Markdown file, put `<!-- hakkiri Style.Rule: off -->` before it and `<!-- hakkiri Style.Rule: on -->` after it. `<!-- hakkiri off -->` and `<!-- hakkiri on -->` skip every rule.

The config keys are in `hakkiri instructions`.
