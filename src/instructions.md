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
- `--format json` prints `passed`, `failed`, `files`, `results`, `uncovered` and `failures`. An `uncovered` entry is an object with the `rule` name, and with `files` when two rule files give the name.

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

- `alerts`: each with `path`, `line`, `column`, `end_line`, `end_column`, `level`, `rule`, `message`, `description`, `link` and `match`.  
  `line` and `column` name the first character of the match and `end_line` and `end_column` its last, so a match that wraps a line ends on a later line.  
  Columns count characters.
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

Every long option reads `HAKKIRI_<LONG>`: `HAKKIRI_CONFIG` names the config, `HAKKIRI_CACHE_DIR` names the folder that keeps fetched packages, `HAKKIRI_FAIL_LEVEL` sets the level, `HAKKIRI_COVERAGE` turns coverage on for `test`, and `HAKKIRI_FORMAT=json` asks for JSON.
`HAKKIRI_COLOR` and `HAKKIRI_QUIET` set the shared flags `--color` and `--quiet`, and `--no-color` has no variable.
Pretty output of `check` holds no colour, so `--color` changes nothing there. `--quiet` hides a successful pretty run.
A flag on the command line wins over its variable, and an empty variable counts as unset.

## The config

```yaml
level: suggestion
packages:
  - path: styles
vocab: [ProseStyle]
files:
  '*.{md,mdx,rs}': {styles: [ProseStyle]}
  'docs/**': {vocab: [Docs]}
  '*.rs': {ProseStyle.BareURL: off}
  CHANGELOG.md: {styles: []}
```

- The config is `.ctl/hakkiri.yaml` in the current directory, or the file `--config` names. Every `path` and glob is relative to the project root, the folder that holds `.ctl`, and a file outside it matches no glob. A path given outside the root draws a warning on standard error, and a run where no path lies under the root exits 2.
- A key the config does not define is an error that names the key and its line.
- `level` is the lowest level reported: `suggestion`, `warning` or `error`.
- `packages` lists folders of styles. Each `path` holds one folder per style, and each style folder holds one `.yml` file per rule, in the folder or under it. A folder that starts with `.` or `_` and a `*.test.yml` file are skipped, and so is a folder named `config`, where Vale keeps vocabularies and other settings.
  Two folders that hold one style name are refused. So are two files that give one rule name, and a name with a bracket, when a glob lists the style: a style no glob lists is not refused, as in Vale.
  A `url` entry holds the `url` of a zip, which starts with `https://` or `http://`, and its `sha256`, 64 lowercase hex digits. The first run downloads the zip, checks the hash, and unpacks it into the cache folder under that hash, and a later run reads it from there and asks for nothing.
  The zip holds one top folder, which holds a `styles` folder of styles or is a style itself, as in a Vale package, and the cache keeps it as the zip laid it out.
  A download is an error that names the `url`, and nothing is cached, when it is over 32 MiB or does not match its hash.
  The same holds when it is not a zip, names a file outside the package, holds a link or has no single top folder.
  The cache folder is `--cache-dir`, or `hakkiri` in the per-user cache folder: `~/Library/Caches` on macOS, and `$XDG_CACHE_HOME` or `~/.cache` elsewhere.
  A killed run leaves a folder named `.unpacking-` and a suffix in the cache. Delete it when no hakkiri run is using the cache.
  A Vale style folder loads as it is. `extends: Style.Rule` names a parent rule in any loaded package, `key+` adds to a list or mapping of the parent, and `key-` removes from one.
- `vocab` lists vocabularies by name, at the top level and in a glob of `files`. A vocabulary is the folder `config/vocabularies/<name>/` of a package.
  hakkiri reads every `accept.txt` and `reject.txt` below it, one term to a line, and skips a blank line and a line that starts with `# `. A term is plain text, with literal periods, or a pattern.
  The accepted terms of the top level hold back matches in `capitalization`, `conditional`, `existence`, `repetition`, `substitution` and `spelling` rules. A rule with `vocab: false` reads none, and nor does an `existence` rule with `nonword`.
  A glob's list holds back matches in `existence`, `repetition`, `spelling` and `substitution` rules in the files that glob matches, whatever the rule sets. A file takes the lists of every glob that matches it.
  An unknown vocabulary, a name in two packages and a term that is not a pattern are errors.
  The lists build rules, as in Vale. `Vale.Terms` reports an accepted term written in another case, and `Vale.Avoid` reports a rejected term, both at `error`. They belong to a style `Vale` that exists only while a top-level vocabulary holds a term. `Terms` matches a term without regard to case and keeps the escapes of a pattern term, so `data\Wset` keeps its `\W`, where Vale lowercases it to `\w`.
  List `Vale` in `styles` to run them, or set them by name. A vocabulary that a glob lists builds `Vale.<Name>.Terms` and `Vale.<Name>.Avoid`, which run on that glob's files with no setting.
- A package may hold `.ctl/hakkiri.yaml` in its top folder, the folder that holds `styles`, in the format of the config. A `path` entry that names a folder called `styles` has its top in the parent of that folder. When the parent is the project root or holds it, there is no top, so the project's own config is never read as a package's. For a zip the top is the top folder of the zip.
  The settings of each package apply before the project's own, in the order of `packages`, so the project overrides a package and a later package overrides an earlier one.
  The file holds `level`, `vocab`, `files` and `packages`. A `styles` list under the same glob text as a package's adds to its list, and under another glob it replaces it. `level` is the project's when it sets one.
  A written `vocab: []` drops the vocabularies the packages named, and a missing `vocab` keeps them. In a glob it drops the lists that earlier configs gave the same glob text.
  A package's `packages` hold a `url` and a `sha256`, never a `path`, and are fetched like the project's. A package named twice loads once, and a chain more than 10 packages deep is an error.
  A fault in the file is an error that names the package, the file and the line.
- `files` maps globs to settings. A file is linted only when a glob matches it. `*` crosses `/`, and `**` and `{a,b}` work.
  Every matching glob applies in the order written, and a later glob wins for the same key.
- `block_ignores` and `token_ignores` list patterns in a glob of `files`, as Vale's `BlockIgnores` and `TokenIgnores` do, and a package's settings file can hold them too.
  In a Markdown or MDX file, each match of a block pattern is read as a fenced code block that holds its first group, and each match of a token pattern as a code span that holds it. No rule that reads prose reports text inside them.
  A `scope: raw` rule still reads the file as written, and a `scope: code` rule reads each ignored token.
  The patterns run in order, block patterns first, each over the text the one before it left. A pattern with no group drops its match from the text, and a match of no bytes changes nothing.
  A file takes the patterns of every glob that matches it, a package's first, so a project adds to a package. A source file takes none, since Vale applies none to a comment.
  A pattern that does not compile is an error on its line.
- An HTML comment in a Markdown file turns rules off and on, as Vale's comments do.
  `<!-- hakkiri off -->` and `<!-- hakkiri on -->` leave out the text between them. `<!-- hakkiri Style.Rule: off -->` and `<!-- hakkiri Style.Rule: on -->` turn one rule off and on, and `<!-- hakkiri Style: off -->` and `on` a style.
  `<!-- hakkiri Style.Rule: suggestion -->`, `warning` or `error` gives the alerts of that rule after it that level, over a level the config sets, until a later comment names the rule. `on` returns the level of the config.
  Vale's forms work too: `<!-- vale off -->`, `<!-- vale on -->`, `<!-- vale Style.Rule = NO -->` and `YES` (or `off` and `on`), `<!-- vale Style = NO -->`, `<!-- vale Style.Rule["a","b"] = NO -->` for the matches `a` and `b`, and `<!-- vale styles = A, B -->` to run only those styles.
  A comment acts on what follows it. A comment in the middle of a paragraph, a list item, a heading or a table cell turns the rule off for that whole element.
  An alert between the comment that turns a rule off and the one that turns it on is left out.
  The words must be exactly as shown, with the spaces around `:` and `=`, and case matters. A comment in a code block or a code span is text. MDX files and source files read no comment control.
- `styles` lists the styles whose rules run. A later list replaces the earlier one. An empty list turns linting off for those files, and drops every rule setting before it.
- A rule name, such as `ProseStyle.BareURL`, sets that rule's level to `suggestion`, `warning` or `error`, turns it `on` at the level of its rule file, or turns it `off`.
  A level or `on` also runs the rule when no glob lists its style, as `Team.Terms: warning` does, and `off` on such a rule changes nothing.
  A level set once stays when a later glob turns the rule `off` and `on` again.
  A name that no loaded style holds is an error that names it and its line.
  A rule hakkiri cannot run, a `script` rule or one with a `doc()` scope, loads no rule, and the rest of its style runs.
  Each such rule in a style a glob lists, or that a level or `on` turns on, makes the run exit 2 with a `failed` line, until the config turns it `off`. In a style no glob lists it is ignored.

## Differences from Vale

`hakkiri test` refuses a case that sets `view`, reads no `.yaml` file, and prints its report in its own layout.

`hakkiri check` has no `--glob`, `--filter`, `--ext`, `--sort`, `--relative` or `--no-global`, and reads no standard input.
It prints the level in each line, and its JSON is one object instead of a map from path to alerts.
`--fail-level` takes the place of `--no-exit`.

hakkiri fetches a package pinned by `url` and `sha256` on the first run and keeps it in a cache folder outside the project, where Vale needs `vale sync` and copies the styles into `StylesPath`.
It refuses a download that does not match its hash, and Vale installs any zip.

hakkiri writes `on` and `off` where Vale writes `YES` and `NO`.
A key that names no rule, or a word that is neither a level, `on` nor `off`, is an error.
Vale ignores the key, and warns and turns the rule off for the word.

hakkiri runs no `script` rule, no `doc()` scope and no View, and reads no `.vale.ini` and no built-in `Vale` style.
