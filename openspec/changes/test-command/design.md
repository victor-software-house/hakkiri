# Design

## Context

[`check-command`][check-design] gives hakkiri a binary, a config reader and a result type that ctl-core renders.
The [rule-files spec][rule-files] fixes the fields of a case and when it passes.
Vale's `vale test` is the model: ProseStyle's [rules][prose-style-rules] already carry cases in its format.
This design reads Vale at 66bbb85 for what the spec leaves open.

## Goals / Non-Goals

Goals:

- Run every case that ProseStyle's cases hold, with the result `vale test` gives.
- A coverage check that fails when a rule fires in no case.
- A report a person reads and a caller parses.

Non-goals:

- Views, which hakkiri's own scopes replace.
- Watching files, and running cases in parallel: a case is a few lines.

## Decisions

### 1. Cases come from two places

A rule file's `tests` list holds cases for that rule alone.
A file named `*.test.yml` holds a list of cases that run through the project config, and a case may name one `rule` to isolate itself.
Vale reads both ([the loader][v-load]); ProseStyle holds its package cases in `test/package.test.yml`.

### 2. A case is a small record

A case has `name`, `about`, `input`, `format`, `rule`, `want`, `contains` and `absent`.
`name` is required and unique in its file.
`format` is an extension and defaults to `md`.
`contains` and `absent` take a string or a list.
A case with none of `want`, `contains` and `absent` is refused, because it passes whatever the rule does.
A field other than these is refused with its line, as every hakkiri file does.
`view` is one of them, since hakkiri has no Views.

### 3. A case sees alerts as lines

A case's alerts are one line each, `line:column:Rule:message`, in order of line and column.
`contains` and `absent` test the text for a substring, and `want` equals it after both are trimmed.
This is Vale's form ([run][v-run]), so ProseStyle's `want` and `contains` strings keep their meaning.

### 4. An isolated case loads one rule

A case in a rule's `tests`, or one with `rule`, loads that rule alone, from a path relative to its test file.
It reads no config, reports an alert of any level, and applies no `level` or `off` from a glob.
The rule keeps its project name: its style folder, the folders under it and its file name up to the first dot, joined by dots, so `Foo.v2.yml` in `Team` is `Team.Foo` (change `rule-names`).
The style loader and the `test` command derive the name in one function, so a coverage report and an alert agree.
The name comes from the path made absolute, with symlinks left alone.
Vale does the same: `vale test styles/Team/Foo.yml` names the rule `Team.Foo` when `styles/Team` is a symlink, and `Foo.yml` or `./Foo.yml` run from inside `Team` does too.
The fold is lexical on purpose, as Go's `filepath.Abs` is, and the rule's text, name and package all come from the folded path.
In a probe, a case with `rule: via/../Team/Foo.yml` through a symlink read `Team/Foo.yml` beside the case file; the symlink target's parent played no part.
Any other case lints `input` as a file named `input.<format>` in the current directory, through the whole config, so its globs, styles and levels apply.
Vale isolates the same way ([isolate][v-isolate]).

### 5. Finding cases

An argument is a file, read as given, or a directory, walked in sorted order, and no argument means the current directory.
The walk skips a dot directory except the root it was given, as Vale does ([find][v-find]).
It skips `node_modules` too, through the same test as the `check` walk, so one list of skipped folders serves both.
A `*.test.yml` file is read as cases, and any other `.yml` file only when it holds `extends` and `tests`, so `test` can walk a repository that holds other YAML.
No case found is an error.

### 6. Coverage names rules no case fires

With `--coverage`, hakkiri lists every rule file under the paths that holds `extends`, and fails when a rule produced no alert in any case.
A rule that matches nothing loads, runs and reports success, so a case that expects nothing from it proves nothing.
The walk for rules skips dot and underscore folders, as the style loader does ([coverage][v-coverage]).

### 7. The report and the exit status

The result is one value that ctl-core renders.
The pretty form prints, for each failed case, its file, name and reason, its `about`, and the alerts it got, with a diff against `want`, then a count line: `3 files, 41 passed, 1 failed`.
`--format json` prints `passed`, `failed`, `results` and `uncovered`.
The exit status is 0 when every case passes and no rule is uncovered, 1 when a case fails or a rule is uncovered, and 2 when a case could not run, after every other case has reported.
A case cannot run when its rule path is missing, its rule does not load, its file is not YAML, or it is refused by decision 2.
Vale's statuses are the same ([report][v-report]).

### 8. A rule file may hold `tests`

Every check type of the rule schema gains `tests`, read as the cases of decision 2.
`check` loads the field and never runs it.
A rule whose `tests` is malformed is refused at load, with the line.

Alternative: a rule file whose `tests` check never reads.
Lost: a mistake in a case would surface only in `test`, and the rule would load with the field unchecked.

### 9. No new dependency

Cases read through ctl-core's `input` reader, as rules do, so a case has a line number for every error.
The runner is sequential.

## Risks / Trade-offs

- [Two sources of cases double the discovery rules] → One walk finds both, and a scenario covers each source.
- [A case without a rule runs through the whole config, so a config change moves its result] → That is its purpose; an isolated case does not move.

[check-design]: ../archive/2026-11-10-check-command/design.md
[rule-files]: ../../specs/rule-files/spec.md
[prose-style-rules]: ../archive/2026-11-10-rule-format/prose-style-rules.md
[v-load]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/testsuite/suite.go#L117-L167
[v-run]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/testsuite/run.go#L93-L115
[v-isolate]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/testsuite/run.go#L261-L318
[v-find]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/testsuite/suite.go#L199-L252
[v-coverage]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/testsuite/coverage.go#L15-L112
[v-report]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/cmd/vale/test.go#L44-L111
