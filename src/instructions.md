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
hakkiri check --no-gitignore
hakkiri check --on-unmatched error README.md
hakkiri check --on-unknown warn
hakkiri check --format json
```

- A path is a file or a directory. A directory is walked in sorted order, and with no path the current directory is walked.
- A walk skips `node_modules`, `.git` and each folder a `packages` entry names.
- A walk also skips what `.gitignore`, `.git/info/exclude` and the global git excludes ignore, as ripgrep does, inside a git repository. `--no-gitignore`, `--gitignore=false` or `gitignore: false` in the config reads those files too. A file named on the command line is never skipped for being ignored.
- A file named on the command line that no glob selects draws a warning on standard error and is not linted. `--on-unmatched` or `on_unmatched` takes `warn`, the default, `error`, which fails the run with exit 2, or `ignore`. A file that a glob selects but whose styles hold no rule draws nothing, and nor does a file a walk found.
- Readers come from the extension: `.md`, `.mdx`, `.rs`, `.ts`, `.tsx` and `.js`, or from a `formats` entry. A file that a glob selects and that no reader reads is plain text: one block per run of lines between blank lines, read as written.
- `--on-unknown` or `on_unknown` takes `text`, the default, `warn`, which skips the file with a warning, `error`, which fails the run with exit 2, or `ignore`. A file that no glob selects draws no message.
- A plain-text file whose first 8192 bytes hold a NUL byte or are not valid UTF-8 is binary, and a run skips it with no message.
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
  A `format` that no reader reads is plain text, and a case that runs through the config reads `input.<format>` by the config's `formats` entry first.
  A case through the config follows `on_unknown` for such a format: `error` fails the case, and `warn` and `ignore` give it no alerts.
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
`HAKKIRI_GITIGNORE` takes `true` or `false`, so `HAKKIRI_GITIGNORE=false` reads the files git ignores, and `--no-gitignore` has no variable. `HAKKIRI_ON_UNMATCHED` takes `warn`, `error` or `ignore`, and `HAKKIRI_ON_UNKNOWN` takes `text`, `warn`, `error` or `ignore`.
A flag on the command line wins over its variable, and an empty variable counts as unset.
The config sets `fail_level`, `gitignore` and `on_unmatched` too, and the order is the flag, then the variable, then the config, then the default.

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
- `fail_level`, `gitignore`, `on_unmatched` and `on_unknown` set the options of the same names for `check`. Only the project's config holds them, and a package's settings file that does is an error.
- `formats` maps an extension, a file name or a glob to a reader: `md`, `mdx`, `rs`, `ts`, `tsx`, `js` or `text`. A key that holds `*`, `?`, `[`, `{` or `/` is a glob.
  An entry named by the file's extension wins, then the first key in text order that is the file's name or a glob that matches its name or path. An entry beats the extension table, so `md: text` reads Markdown files as text.
  A file that an entry maps has the extension of its reader in a scope such as `text.md`: with `txt: md`, that scope selects text in `a.txt`. Vale compares the scope with the file's own extension, so it selects nothing there. A case that runs through the config has the same extension.
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
  The lists build rules, as in Vale. `Vale.Terms` reports an accepted term written in another case, and `Vale.Avoid` reports a rejected term, both at `error`. They belong to a style `Vale` that exists only while a top-level vocabulary holds a term. `Terms` matches a term without regard to case and keeps the escapes of a pattern term, so `data\Wset` keeps its `\W`, where Vale lowercases it to `\w`. It also keeps the names of groups, the backreferences to them and the flags, such as `(?U)`.
  List `Vale` in `styles` to run them, or set them by name. A vocabulary that a glob lists builds `Vale.<Name>.Terms` and `Vale.<Name>.Avoid`, which run on that glob's files with no setting.
- A package may hold `.ctl/hakkiri.yaml` in its top folder, the folder that holds `styles`, in the format of the config. A `path` entry that names a folder called `styles` has its top in the parent of that folder. When the parent is the project root or holds it, there is no top, so the project's own config is never read as a package's. For a zip the top is the top folder of the zip.
  The settings of each package apply before the project's own, in the order of `packages`, so the project overrides a package and a later package overrides an earlier one.
  The file holds `level`, `vocab`, `files`, `formats` and `packages`. A `styles` list under the same glob text as a package's adds to its list, and under another glob it replaces it. `level` is the project's when it sets one.
  A written `vocab: []` drops the vocabularies the packages named, and a missing `vocab` keeps them. In a glob it drops the lists that earlier configs gave the same glob text.
  A package's `packages` hold a `url` and a `sha256`, never a `path`, and are fetched like the project's. A package named twice loads once, and a chain more than 10 packages deep is an error.
  A fault in the file is an error that names the package, the file and the line.
- `files` maps globs to settings. A file is linted only when a glob matches it. `*` crosses `/`, and `**` and `{a,b}` work.
  Every matching glob applies in the order written, and a later glob wins for the same key.
- `block_ignores` and `token_ignores` list patterns in a glob of `files`, as Vale's `BlockIgnores` and `TokenIgnores` do, and a package's settings file can hold them too.
  In a Markdown or MDX file, each match of a block pattern is read as a fenced code block that holds its first group, and each match of a token pattern as a code span that holds it. No rule that reads prose reports text inside them.
  A `scope: raw` rule still reads the file as written, and a `scope: code` rule reads each ignored token.
  The patterns run in order, block patterns first, each over the text the one before it left. A pattern with no group drops its match from the text, and a match of no bytes changes nothing.
  A file takes the patterns of every glob that matches it, a package's first, so a project adds to a package. A source file takes them too, in the Markdown of each comment, where Vale applies none. A `hakkiri test` case that runs through the config reads them too.
  A pattern that does not compile is an error on its line.
- An HTML comment in a Markdown file turns rules off and on, as Vale's comments do.
  `<!-- hakkiri off -->` and `<!-- hakkiri on -->` leave out the text between them. `<!-- hakkiri Style.Rule: off -->` and `<!-- hakkiri Style.Rule: on -->` turn one rule off and on, and `<!-- hakkiri Style: off -->` and `on` a style.
  `<!-- hakkiri Style.Rule: suggestion -->`, `warning` or `error` gives the alerts of that rule after it that level, over a level the config sets, until a later comment names the rule. `on` returns the level of the config.
  Vale's forms work too: `<!-- vale off -->`, `<!-- vale on -->`, `<!-- vale Style.Rule = NO -->` and `YES` (or `off` and `on`), `<!-- vale Style = NO -->`, `<!-- vale Style.Rule["a","b"] = NO -->` for the matches `a` and `b`, and `<!-- vale styles = A, B -->` to run only those styles.
  A comment acts on what follows it. A comment in the middle of a paragraph, a list item, a heading or a table cell turns the rule off for that whole element.
  An alert between the comment that turns a rule off and the one that turns it on is left out.
  The words must be exactly as shown, with the spaces around `:` and `=`, and case matters. A comment in a code block or a code span is text.
  An MDX file reads `{/* hakkiri off */}` and `{/* vale off */}` in the same way, as a block of its own or inside a line, and a source file reads no comment control.
- An MDX file's `{…}` expressions, and its JSX elements with no children, read as code, as in Vale: a `code` rule reads `{x}` and `<Tag a="x" />`, and no text rule does.
  The tags of an element with children and a `{/* … */}` comment read as no node, and an open and a close tag with nothing between them inside a line of text are no code.
- Math in a Markdown or MDX file reads as code, as in Vale, so no text rule reports it and a `code` rule reads it.
  An inline span runs from a `$` that a non-space follows to a `$` that follows a non-space and is no digit's neighbour: `$5 and $10` is money and `$x$` is math. A span may wrap over lines of a paragraph.
  A display block opens at a line that starts with `$$`, even in the middle of a paragraph, and closes at a line that ends with `$$`, a blank line or the end of the paragraph. The text before and after reads as separate paragraphs. A line indented four columns or more past its container opens no block.
- The top-level string values of a Markdown or MDX file's front matter are text, as in Vale: YAML between `---` lines, TOML between `+++` lines and JSON between `;;;` lines.
  A rule scoped `text`, `frontmatter`, `text.frontmatter`, `frontmatter.<key>` or `text.frontmatter.<key>` reads a value, and a `paragraph` or `sentence` rule does not.
  A key, a number, a boolean, a null, a date, a list and a nested mapping are not read, nor is a string below a TOML table, a YAML alias or a block that holds a key twice. An anchor or a tag before a value changes nothing. A block or token pattern leaves the front matter as written.
- `styles` lists the styles whose rules run. A later list replaces the earlier one. An empty list turns linting off for those files, and drops every rule setting before it.
- A rule name, such as `ProseStyle.BareURL`, sets that rule's level to `suggestion`, `warning` or `error`, turns it `on` at the level of its rule file, or turns it `off`.
  A level or `on` also runs the rule when no glob lists its style, as `Team.Terms: warning` does, and `off` on such a rule changes nothing.
  A level set once stays when a later glob turns the rule `off` and `on` again.
  A name that no loaded style holds is an error that names it and its line.
  A rule hakkiri cannot run, a `script` rule or one with a `doc()` scope, loads no rule, and the rest of its style runs.
  Each such rule in a style a glob lists, or that a level or `on` turns on, makes the run exit 2 with a `failed` line, until the config turns it `off`. In a style no glob lists it is ignored.

## Rule scopes

A rule's `scope` is Vale's: lowercase dotted names such as `text`, `heading.h1` and `sentence`, joined by `~`, `&` and lists.
`&` keeps the blocks that hold every name, so `sentence & paragraph` selects the sentences of paragraphs.

hakkiri adds two scopes that Vale lacks.
`title` selects the title of a Markdown link, of an image and of a link reference definition, and the `title` attribute of a tag in an HTML block.
`image` selects the alt text of an image.

A title is selected by what holds it too.
`title & link` selects the title of a link, `title & image` the title of an image, and only `title` selects the title of a definition.
In an HTML block, `title & link` selects the `title` attribute of an `a` tag and `title & image` that of an `img` tag.
`title & ~link` leaves out the titles of links, and `title & ~image` those of images.
`link` and `image` alone never select a title, and no other scope reads one, as in Vale, where only `raw` does.
A `title` attribute in a line of text is read by `raw` only.

## Differences from Vale

`hakkiri test` refuses a case that sets `view`, reads no `.yaml` file, and prints its report in its own layout.

`hakkiri check` has no `--glob`, `--filter`, `--ext`, `--sort`, `--relative` or `--no-global`, and reads no standard input.
It prints the level in each line, and its JSON is one object instead of a map from path to alerts.
`--fail-level` takes the place of `--no-exit`.

hakkiri fetches a package pinned by `url` and `sha256` on the first run and keeps it in a cache folder outside the project, where Vale needs `vale sync` and copies the styles into `StylesPath`.
It refuses a download that does not match its hash, and Vale installs any zip.

hakkiri reads a plain-text file as paragraphs and skips a binary file, where Vale reads a file with no reader as lines of one block.
It takes a project's `formats` entry over a package's, where Vale keeps the first value.

hakkiri writes `on` and `off` where Vale writes `YES` and `NO`.
A key that names no rule, or a word that is neither a level, `on` nor `off`, is an error.
Vale ignores the key, and warns and turns the rule off for the word.

hakkiri runs no `script` rule, no `doc()` scope and no View, and reads no `.vale.ini` and no built-in `Vale` style.

hakkiri tags a sentence with the model of prose at `b33e60d`, where Vale tags with the older model of prose v3.2.2.
It then corrects the tags of a possessive `'s`, a modal or a name in capitals, a semicolon, and a verb after a noun phrase.
A `sequence` rule can therefore flag other words than in Vale.
A title keeps the tags of the model alone.

hakkiri reads a code span as a word that no token of a `sequence` rule names, and a negated token holds at it.
Vale reads the span as a run of `*` that a tag or a pattern can match, so it flags `` `flag` says `` for a noun and a verb.
