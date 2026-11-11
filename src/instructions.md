# hakkiri operator instructions

`hakkiri check` lints Markdown, MDX, and the comments of Rust, TypeScript and JavaScript files.
The project config, `.ctl/hakkiri.yaml`, says which rules run on which files.
`hakkiri test` runs the test cases of rule files.
`hakkiri instructions` and `--help` are the contract of the installed version.

## Commands

| Command | What it does |
|:--|:--|
| `check` | Lints files and directories, and prints one line per alert |
| `test` | Runs the test cases of rule files and `*.test.yml` files |
| `instructions` | Prints this text |

## Run a check

```sh
hakkiri check
hakkiri check docs README.md
hakkiri check --fail-level warning
hakkiri check --format json
```

- A path is a file or a directory. A directory is walked in sorted order, and with no path the current directory is walked.
- A walk skips `node_modules`, `.git` and each folder a `packages` entry names. It skips files that no language reads, with no message.
- A file named on the command line that no language reads is a failure.
- Languages come from the extension: `.md`, `.mdx`, `.rs`, `.ts`, `.tsx` and `.js`.
  Other files, plain text included, are not linted yet.
- Files lint in parallel. The report is sorted by path, then line, then column, so two runs print the same text.
- A file that cannot be read or parsed is a failure. The other files still lint.

## Run the test cases

```sh
hakkiri test
hakkiri test styles test/package.test.yml
hakkiri test --coverage
hakkiri test --format json
```

- A rule file holds cases under `tests`, and a `*.test.yml` file is a list of cases. `check` loads the cases and never runs them.
- A case has `name` (required, unique in its file), `about`, `input`, `format` (an extension, `md` by default), `rule`, `want`, `contains` and `absent`.
  A case needs one of `want`, `contains` and `absent`. `contains` and `absent` take a string or a list.
- A case sees its alerts as lines, `line:column:Rule:message`. `want` equals them after both are trimmed, and an empty `want` means no alerts.
- A case in a rule file, and a case with `rule` (a path relative to its test file), loads that rule alone: no config, every level.
  Any other case lints `input` as `input.<format>` through the project config.
- A rule is named by its style folder, the folders under it and its file name up to the first dot, joined by dots: `Team/Sub/Bar.yml` is `Team.Sub.Bar`, and `Team/Terms.custom.yml` is `Team.Terms`.
  `test` finds the style folder from the packages of the config, and from the folder that holds the file when no package holds it.
  The style folder is named as the path gives it, so a style folder that is a symlink keeps the name of the link, and `Foo.yml` run from inside `Team` is `Team.Foo`.
- A path is a file or a directory. A walk skips dot folders other than the one named, and `node_modules`. It reads a `.yml` file that is not a test file only when it holds `extends` and `tests`.
- `--coverage` lists the rule files under the paths and fails when a rule flags nothing in any case.
- A failed case prints as `failed <file>: <name>`, its reason, its `about`, and the alerts it got, with a `-` and `+` diff against `want`. The last line counts the files, the cases that passed and the cases that failed.
- `--format json` prints `passed`, `failed`, `files`, `results`, `uncovered` and `failures`.

`hakkiri test` exits 0 when every case passes and no rule is uncovered.
It exits 1 when a case fails or a rule is uncovered.
It exits 2 when a case could not run, a file of cases could not be read, or no case was found.
It reports every other case before it exits.

## Output

Each alert is one line, then one count line:

```text
docs/guide.md:12:5: warning ProseStyle.Filler: Remove 'very'.
1 alert in 3 files
```

A failure prints as `failed <path>: <message>` after the alerts.
Pretty output holds no colour, because tools read these lines.
`--format json` prints one object:

- `alerts`: each with `path`, `line`, `column`, `end_column`, `level`, `rule`, `message`, `description`, `link` and `match`.
- `failures`: each with `path` or `config`, and `message`.
- `files`: how many files were linted.

JSON never holds an escape byte, whatever `CLICOLOR_FORCE` or `FORCE_COLOR` say.
A parse error under `--format json` is JSON too.

## Exit status

| Code | When |
|:--|:--|
| 0 | No reported alert reaches `--fail-level` |
| 1 | A reported alert reaches `--fail-level`, which is `suggestion`, `warning` or `error` and defaults to `error` |
| 2 | The config or a rule is wrong, a file failed, no path lies under the project root, or the command line is wrong |

The config's `level` hides alerts below it first, so `--fail-level` sees only what is reported.
A broken config stops the run before any file lints, and prints to standard error.

## Variables

Every long option reads `HAKKIRI_<LONG>`: `HAKKIRI_CONFIG` names the config, `HAKKIRI_FAIL_LEVEL` sets the level, `HAKKIRI_COVERAGE` turns coverage on for `test`, and `HAKKIRI_FORMAT=json` asks for JSON.
`HAKKIRI_COLOR` and `HAKKIRI_QUIET` set the shared flags `--color` and `--quiet`, and `--no-color` has no variable.
Pretty output of `check` holds no colour, so `--color` changes nothing there. `--quiet` hides a successful pretty run.
A flag on the command line wins over its variable, and an empty variable counts as unset.

## The config

```yaml
level: suggestion
packages:
  - path: styles
files:
  '*.{md,mdx,rs}': {styles: [ProseStyle]}
  '*.rs': {ProseStyle.BareURL: off}
  CHANGELOG.md: {styles: []}
```

- The config is `.ctl/hakkiri.yaml` in the current directory, or the file `--config` names. Every `path` and glob is relative to the project root, the folder that holds `.ctl`, and a file outside it matches no glob. A path given outside the root draws a warning on standard error, and a run where no path lies under the root exits 2.
- A key the config does not define is an error that names the key and its line.
- `level` is the lowest level reported: `suggestion`, `warning` or `error`.
- `packages` lists folders of styles. Each `path` holds one folder per style, and each style folder holds one `.yml` file per rule, in the folder or under it. A folder that starts with `.` or `_` and a `*.test.yml` file are skipped.
  Two folders that hold one style name are refused. So are two files that give one rule name, and a name with a bracket, when a glob lists the style: a style no glob lists is not refused, as in Vale.
  A `url` entry is not downloaded yet.
- `files` maps globs to settings. A file is linted only when a glob matches it. `*` crosses `/`, and `**` and `{a,b}` work.
  Every matching glob applies in the order written, and a later glob wins for the same key.
- `styles` lists the styles whose rules run. An empty list turns linting off for those files.
- A rule name, such as `ProseStyle.BareURL`, sets that rule's level to `suggestion`, `warning` or `error`, or turns it `off`.
  A name that no loaded style holds is an error that names it and its line.

## Differences from Vale

`hakkiri test` refuses a case that sets `view`, reads no `.yaml` file, and prints its report in its own layout.

`hakkiri check` has no `--glob`, `--filter`, `--ext`, `--sort`, `--relative` or `--no-global`, and reads no standard input.
It prints the level in each line, and its JSON is one object, not a map from path to alerts.
`--fail-level` takes the place of `--no-exit`.
