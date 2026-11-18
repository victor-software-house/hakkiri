# hakkiri operator instructions

`hakkiri check` lints Markdown, Markdown with components (MDX) and source comments.  
It reads the comments of Rust, TypeScript, JavaScript, Go, Swift, YAML, TOML and Shell files.  
The project config, `.ctl/hakkiri.yaml`, sets which rules run on which files, and `hakkiri test` runs the test cases of rule files.  
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
hakkiri check --code-span-size 0
hakkiri check --format json
```

### Paths and walks

Each path is a file or a directory.  
A directory is walked in sorted order, and with no path the current directory is walked.  
Every walk skips `node_modules`, `.git` and each folder that a `packages` entry points to.

Inside a git repository, a walk also skips what git ignores, as ripgrep does.  
That is what `.gitignore`, `.git/info/exclude` and the global git excludes ignore.  
`--no-gitignore`, `--gitignore=false` or `gitignore: false` in the config reads those files too.  
A file given on the command line is never skipped for being ignored.

A file given on the command line that no glob selects draws a warning on standard error, and is not linted.  
`--on-unmatched` or `on_unmatched` takes `warn`, the default, `error` or `ignore`.  
`error` fails the run with exit 2.  
A file that a glob selects but whose styles contain no rule draws nothing.  
Nor does a file that a walk found.

### Languages

A file has the language of its path, by its file name and then its extension.  
Otherwise it has the language of the interpreter on its first line, and else none.  
`LICENSE` and `notes.txt` are Text, `Cargo.toml` is TOML, and `README` has none.  
`bin/tool` that starts `#!/usr/bin/env bash` is Shell.

### Readers

A file is read by the reader of its `formats` entry, then by the reader of its language.  
The readers are Markdown, MDX, Rust, TypeScript, TSX (TypeScript with markup), JavaScript, Go, Swift, YAML, TOML, Shell and Text.  
So `.mts`, `.markdown`, `.yml`, `.bash` and a `bashrc` file have a reader.  
A script that starts `#!/usr/bin/env node` or `#!/bin/sh` has one too.

A file that a glob selects and that no reader reads is plain text, unless `on_unknown` is set to something else.  
Plain text is one block per run of lines between blank lines, read as written.  
A Text file is plain text whatever `on_unknown` is set to.

`--on-unknown` or `on_unknown` takes `text`, the default, `warn`, `error` or `ignore`.  
`warn` skips the file with a warning, and `error` fails the run with exit 2.  
A file that no glob selects draws no message.

### Comments in source files

A source, YAML, TOML or Shell file gives its comments only, read as Markdown.  
So a string, a key and a heredoc are never prose.

hakkiri leaves out a comment addressed to a tool, and the `#!` line of a script.  
Such comments are `//go:`, `eslint-disable`, `@ts-ignore`, `swiftlint:`, `yaml-language-server:`, `#:schema`, `shellcheck` and the like.

Some parts of a doc comment draw no alert, and the words around them keep their counts.  
They are a link in brackets in a Go or Rust doc comment, the name a Go doc comment opens with, and a JSDoc `{@link}`.

A line comment of a Go, YAML, TOML or Shell file keeps its indentation after the first space.  
So an indented example is code.

### Code span size

`--code-span-size` or `code_span_size` sets how many words a code span counts as.  
It reaches a word count and the metrics built on it.  
These are the `words`, `sentences` and `syllables` of a `metric` or `readability` rule, and the scores.

It is 1 by default.  
With 0 a code span counts as in Vale, and with 2 or more it counts as that many words.  
`HAKKIRI_CODE_SPAN_SIZE` is its variable.  
A rule that reads a match, such as `existence` or `occurrence`, ignores it.

### Bytes, order and failures

A plain-text file that contains a null (NUL) byte in its first 8192 bytes is binary.  
Each run skips such a file with no message.  
A byte that is not valid UTF-8, in a Latin-1 file for example, reads as one `?`.  
So the file is linted, and a column counts bytes, as in Vale.

Files lint in parallel.  
The report is sorted by path, then line, then column, so two runs print the same text.  
A file that cannot be read or parsed is a failure, and the other files still lint.

## Run the test cases

```sh
hakkiri test
hakkiri test styles test/package.test.yml
hakkiri test --coverage
hakkiri test --format json
```

### Cases

In a rule file, cases go under `tests`, and a `*.test.yml` file is a list of cases.  
`check` loads the cases and never runs them.

A case has `name` (required, unique in its file), `about`, `input`, `format` (an extension, `md` by default), `rule`, `want`, `contains` and `absent`.  
A case needs one of `want`, `contains` and `absent`.  
`contains` and `absent` take a string or a list.

A `format` that no reader reads is plain text.  
A case that runs through the config reads `input.<format>` as a file, by the config's `formats` entry first.  
Such a case follows `on_unknown` for that format.  
`error` fails the case, and `warn` and `ignore` give it no alerts.

A case sees its alerts as lines, `line:column:Rule:message`.  
`want` equals them after both are trimmed, and an empty `want` means no alerts.

A case in a rule file loads that rule alone: no config, every level.  
So does a case with `rule`, a path relative to its test file.  
Any other case lints `input` as `input.<format>` through the project config.

### Rule names

A rule's name is its style folder, the folders under it and its file name up to the first dot, joined by dots.  
So `Team/Sub/Bar.yml` is `Team.Sub.Bar`, and `Team/Terms.custom.yml` is `Team.Terms`.

`test` finds the style folder from the packages of the config.  
When no package contains the file, it uses the folder that contains the file.  
The style folder takes its name from the path as given.  
So a style folder that is a symlink keeps the name of the link, and `Foo.yml` run from inside `Team` is `Team.Foo`.

### Paths and coverage

A path is a file or a directory.  
A walk skips dot folders other than the one given, and `node_modules`.  
It reads a `.yml` file that is not a test file only when the file contains `extends` and `tests`.  
`--coverage` lists the rule files under the paths, and fails when a rule flags nothing in any case.

### Test output

A failed case prints as `failed <file>: <name>`, its reason, its `about`, and the alerts it got.  
A `-` and `+` diff against `want` follows.  
The last line counts the files, the cases that passed and the cases that failed.

`--format json` prints `passed`, `failed`, `files`, `results`, `uncovered` and `failures`.  
An `uncovered` entry is an object with the `rule` name.  
It also has `files` when two rule files give the name.

`hakkiri test` exits 0 when every case passes and no rule is uncovered.  
It exits 1 when a case fails or a rule is uncovered.  
Exit 2 means a case could not run, a file of cases could not be read, or no case was found.  
Before it exits, it reports every other case.

## Output

Each alert is one line, then one count line:

```text
docs/guide.md:12:5: warning ProseStyle.Filler: Remove 'very'.
1 alert in 3 files
```

A failure prints as `failed <path>: <message>` after the alerts.  
Pretty output has no colour, because tools read these lines.  
`--format json` prints one object:

- `alerts`: each with `path`, `line`, `column`, `end_line`, `end_column`, `level`, `rule`, `message`, `description`, `link` and `match`.  
  `line` and `column` point to the first character of the match, and `end_line` and `end_column` to its last.  
  So a match that wraps a line ends on a later line.  
  Columns count characters.
- `failures`: each with `path` or `config`, and `message`.
- `files`: how many files were linted.

JSON never contains an escape byte, whatever `CLICOLOR_FORCE` or `FORCE_COLOR` are set to.  
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

Every long option reads `HAKKIRI_<LONG>`.

- `HAKKIRI_CONFIG` gives the config.
- `HAKKIRI_CACHE_DIR` gives the folder that keeps fetched packages.
- `HAKKIRI_FAIL_LEVEL` sets the level.
- `HAKKIRI_COVERAGE` turns coverage on for `test`.
- `HAKKIRI_FORMAT=json` selects JSON.
- `HAKKIRI_COLOR` and `HAKKIRI_QUIET` set the shared flags `--color` and `--quiet`, and `--no-color` has no variable.
- `HAKKIRI_GITIGNORE` takes `true` or `false`, so `HAKKIRI_GITIGNORE=false` reads the files git ignores.  
  `--no-gitignore` has no variable.
- `HAKKIRI_ON_UNMATCHED` takes `warn`, `error` or `ignore`.
- `HAKKIRI_ON_UNKNOWN` takes `text`, `warn`, `error` or `ignore`.

Pretty output of `check` has no colour, so `--color` changes nothing there, and `--quiet` hides a successful pretty run.  
A flag on the command line wins over its variable, and an empty variable counts as unset.  
The config sets `fail_level`, `gitignore` and `on_unmatched` too.  
The order is the flag, then the variable, then the config, then the default.

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

### Config file and root

The config is `.ctl/hakkiri.yaml` in the current directory, or the file given by `--config`.  
Every `path` and glob is relative to the project root, the folder that contains `.ctl`.

A file outside the root matches no glob, and a path given outside the root draws a warning on standard error.  
When no path lies under the root, the run exits 2.

### Glob syntax

A glob reads as gitignore reads one.  
`*`, `?` and a class stay inside one folder, and `**` spans folders.  
With no `/`, a glob matches the file name in any folder.  
A leading `/` ties a glob to the root, and a trailing `/` matches every file in the folder.  
So `docs/*.md` matches `docs/a.md` and not `docs/x/a.md`.

### Top-level keys

A key the config does not define is an error that shows the key and its line.  
`level` is the lowest level reported: `suggestion`, `warning` or `error`.

`fail_level`, `gitignore`, `on_unmatched` and `on_unknown` set the options of the same names for `check`.  
Only the project's config may set them, and a package's settings file that sets them is an error.

### `code_span_size`

`code_span_size` is a whole number of words from 0 to 255.  
It is a key of the project config, of a package's settings file and of a glob under `files`.  
It resolves by the settings cascade.  
The top level of a package's file ranks lowest, then the project's top level, then each matching glob by its rank.

The project may force it with `!override`.  
`HAKKIRI_CODE_SPAN_SIZE` and then `--code-span-size` beat them all.  
A value outside 0 to 255, or a word or a list under a glob, is an error with its line.

### `formats`

`formats` maps an extension, a file name or a glob to a reader, given by any name, alias or extension of the language it reads.  
The languages are Markdown, MDX, Rust, TypeScript, TSX, JavaScript, Go, Swift, YAML, TOML, Shell and Text.  
So `md`, `markdown` and `Markdown` are one reader, and `text` is plain text.

A key that contains `*`, `?`, `[`, `{` or `/` is a glob.

An entry for the file's extension wins first.  
Next comes the first key in text order that is the file's name, or a glob that matches its name or path.  
An entry beats the language of the file, so `md: text` reads Markdown files as text.

For a scope such as `text.md`, a file that an entry maps is in two languages: its own and its reader's.  
With `txt: md`, `text.txt` and `text.md` both select text in `a.txt`.  
Vale compares the scope with the file's own extension only, so `text.md` selects nothing there.  
A case that runs through the config is in the same two languages.

### `packages`

`packages` lists folders of styles.  
Each `path` contains one folder per style.  
Each style folder contains one `.yml` file per rule, in the folder or under it.

hakkiri skips a folder that starts with `.` or `_`, and a `*.test.yml` file.  
A folder called `config` is skipped too, because Vale keeps vocabularies and other settings there.  
A folder called `Vale` is skipped with one warning, because the built-in style `Vale` wins, as in Vale.

Two folders with one style name are rejected when a glob lists the style.  
So are two files that give one rule name, and a name with a bracket.  
A style that no glob lists is not rejected, as in Vale.

A Vale style folder loads as it is.  
`extends: Style.Rule` points to a parent rule in any loaded package.  
`key+` adds to a list or mapping of the parent, and `key-` removes from one.

#### Packages from a `url`

A `url` entry has the `url` of a zip, which starts with `https://` or `http://`.  
It also has the zip's `sha256`, 64 lowercase hex digits.  
The first run downloads the zip, checks the hash, and unpacks it into the cache folder under that hash.  
A later run reads it from there and downloads nothing.

The zip has one top folder, as in a Vale package.  
That folder contains a `styles` folder of styles, or is a style itself.  
The cache keeps it as the zip laid it out.

A download is an error that shows the `url`, and nothing is cached, in these cases:

- it is over 32 MiB, or does not match its hash;
- it is not a zip;
- it has an entry outside the package, or contains a link;
- it has no single top folder.

The cache folder is `--cache-dir`, or `hakkiri` in the per-user cache folder.  
That is `~/Library/Caches` on macOS, and `$XDG_CACHE_HOME` or `~/.cache` elsewhere.  
A killed run leaves a folder called `.unpacking-` and a suffix in the cache.  
Delete it when no hakkiri run is using the cache.

### `vocab`

`vocab` lists vocabularies by name, at the top level and in a glob of `files`.  
A vocabulary is the folder `config/vocabularies/<name>/` of a package.  
hakkiri reads every `accept.txt`, `reject.txt`, `vocab.yml` and `vocab.yaml` below it.

A text list has one term to a line.  
hakkiri skips a blank line and a line that starts with `# `.  
A term is plain text, with literal periods, or a pattern.

#### YAML vocabularies

A YAML file contains `affix`, `accept` and `reject`.  
An entry is a term, a term with nothing after its colon, or a term mapped to options.  
The options are `regex`, `ignorecase` and `flags`.  
A YAML term is literal, with `C++` matching `C++`.

- `regex: true` keeps a pattern.
- `flags: DSG` adds the forms those affix flags give.
- `ignorecase: true` matches without regard to case.  
  A period in such a term is literal, where Vale reads it as any character.
- `regex` and `flags` do not mix.

`affix` gives `<name>.aff` in `config/dictionaries` of the package that has the vocabulary.  
After that package come the other packages, in order.  
With no `affix`, or with `en_US-web`, the built-in dictionary expands the flags.  
Vale also looks in `$DICPATH`, which hakkiri never reads.

A key the file does not define is an error, where Vale ignores it.  
Every wrong entry is reported with its file and line, where Vale stops at the first.

#### Accepted terms

The accepted terms of the top level apply to `capitalization`, `conditional`, `existence`, `repetition`, `substitution` and `spelling` rules.  
In those rules, a match of an accepted term is not reported.  
A rule with `vocab: false` reads no accepted terms, and nor does an `existence` rule with `nonword`.

A glob's list applies to `existence`, `repetition`, `spelling` and `substitution` rules in the files that glob matches.  
It applies whatever the rule sets.  
A file takes the lists of every glob that matches it, and a list the project tags `!override` replaces them.

An unknown vocabulary, a name in two packages, a term that is not a pattern and a wrong YAML entry are errors.

#### `Vale.Terms` and `Vale.Avoid`

The lists build rules, as in Vale.  
`Vale.Terms` reports an accepted term written in another case, at `error`.  
`Vale.Avoid` reports a rejected term, also at `error`.  
Both belong to the built-in style `Vale`, which contains them only while a top-level vocabulary has a term.

`Terms` matches a term without regard to case, and keeps the escapes of a pattern term.  
So `data\Wset` keeps its `\W`, where Vale lowercases it to `\w`.  
It also keeps the names of groups, the backreferences to them and the flags, such as `(?U)`.

A vocabulary that a glob lists builds `Vale.<Name>.Terms` and `Vale.<Name>.Avoid`.  
These run on that glob's files with no setting.

#### `Vale.Repetition`

The style `Vale` also contains `Vale.Repetition`, with Vale's defaults.  
It reports a word that repeats, such as `the the`, at `error`.

The rule ignores case and reads letters only.  
Two words in different sentences, or with a code span between them, are no repeat.  
Nor is a repeat that an accepted term covers.  
List `Vale` in `styles` to run these rules, or set them by name.

#### `Vale.Spelling`

The style `Vale` also contains `Vale.Spelling`.  
It reports a word that is not in the built-in dictionary, at `error`, with the message `Did you really mean '%s'?`.  
The accepted terms of a vocabulary pass.

For each package of the project, its `config/dictionaries` folder adds each dictionary in it, as in Vale.  
Every `.txt` file under its `config/ignore` folder adds the words it lists.  
A link under `config/ignore` that reaches outside the package is a fault.

A glob turns the rule off with `Vale.Spelling: off`.  
`params: {Vale.Spelling: {split: true}}` sets a field, such as `custom`, `append` or `dictionaries`.

hakkiri builds the rule only when a glob uses the style `Vale`.  
A glob uses it when it lists `Vale` in `styles`, lists a rule that starts with `Vale.`, or sets `params` for one.  
A dictionary that cannot load fails no run of a config that does none of these.

### Package settings

A package may ship `.ctl/hakkiri.yaml` in its top folder, in the format of the config.  
The top folder is the folder that contains `styles`.

For a `path` entry that points to a folder called `styles`, the top is the parent of that folder.  
When the parent is the project root or contains it, there is no top.  
So the project's own config is never read as a package's.  
For a zip, the top is the top folder of the zip.

At equal specificity, a setting of the project wins over a package's.  
A later package wins over an earlier one, in the order of `packages`.

The file contains `level`, `vocab`, `files`, `formats` and `packages`.  
`level` is the project's when it sets one.  
A `styles` list under the same glob text as a package's adds to that list.  
Under another glob, the list of higher rank replaces the other.

A written `vocab: []` drops the vocabularies the packages listed, and a missing `vocab` keeps them.  
In a glob it drops the lists that earlier configs gave the same glob text.

The `packages` of a package have a `url` and a `sha256`, never a `path`, and are fetched like the project's.  
A package listed twice loads once, and a chain more than 10 packages deep is an error.  
Any fault in the file is an error that shows the package, the file and the line.

### `files`

`files` maps globs to settings, and a file is linted only when a glob matches it.  
`*` crosses `/`, and `**` and `{a,b}` work.  
Every matching glob applies, and only settings of the same key compete.  
These keys are a rule's switch and its level, `styles`, each parameter of a rule, and `code_span_size`.

The globs apply from the lowest rank to the highest, and a higher rank wins for the same key.  
Rank is the specificity of the glob, then its origin, then the order written.

#### Specificity

Specificity is four counts, compared in order:

1. the parts between slashes made of literal characters only, such as `docs`;
2. the parts that contain a literal character and something else, such as `*.md`;
3. the parts that are `**`, where fewer is more specific;
4. the literal characters of the whole glob.

A literal character is one outside a `[ab]` class that is not `*` or `?`.  
A backslash with the character after it counts as one.  
So `docs/*` beats `*.mdx`, `docs/**/*.md` beats `*.md`, and `a*.md` beats `*.md`, wherever each is written.

A glob with `{a,b}` groups is read as its alternatives.  
For each file, it ranks as the most specific alternative that matches that file.  
So `{CHANGELOG.md,notes/**}` ranks as `CHANGELOG.md` for the changelog and as `notes/**` for a note, and beats `*.{md,mdx,rs}` for both.  
A glob may have 256 alternatives at most.

#### Origin and order

At equal specificity the project ranks above a package, and a later package above an earlier one.  
A later glob ranks above an earlier one in the same file.  
The top level of each config file, such as `code_span_size`, ranks below every glob in the same order.

#### `!override`

The project forces a value by tagging it `!override`, as in `'*.md': {Team.Filler: !override error}`.  
The tag goes on the value of a key directly under a glob, such as a rule's level, `styles` or `code_span_size`.  
It also goes on the top-level `code_span_size`.  
A forced `params` mapping forces every parameter in it, and a tag on one parameter alone is an error.

Forced values apply after every value without the tag, in rank order.  
A forced `styles` list is taken as written, and an empty one drops every rule setting without the tag.

A package's settings file with this tag, or any other tag, is an error that shows the file and the line of the key.  
Vale applies its sections in the order written, a later one winning, and has no way to force a value.

### `block_ignores` and `token_ignores`

`block_ignores` and `token_ignores` list patterns in a glob of `files`, as Vale's `BlockIgnores` and `TokenIgnores` do.  
A package's settings file can set them too.

In Markdown and MDX, a match of a block pattern reads as a fenced code block of its first group.  
A match of a token pattern reads as a code span.  
No rule that reads prose reports text inside them.  
A `scope: raw` rule still reads the file as written, and a `scope: code` rule reads each ignored token.

The patterns run in order, block patterns first, each over the text the one before it left.  
A pattern with no group drops its match from the text, and a match of no bytes changes nothing.  
A pattern that does not compile is an error on its line.

A file takes the patterns of every glob that matches it, in rank order, so a project adds to a package.  
A list the project tags `!override` replaces the lists without the tag.

Source files take the patterns too, in the Markdown of each comment, where Vale applies none.  
A `hakkiri test` case that runs through the config reads them as well.

### Comment controls

An HTML comment in a Markdown file turns rules off and on, as Vale's comments do.

- `<!-- hakkiri off -->` and `<!-- hakkiri on -->` leave out the text between them.
- `<!-- hakkiri Style.Rule: off -->` and `<!-- hakkiri Style.Rule: on -->` turn one rule off and on.
- `<!-- hakkiri Style: off -->` and `on` do the same for a style.
- `<!-- hakkiri Style.Rule: suggestion -->`, `warning` or `error` sets the level of that rule's alerts after it.  
  This beats a level the config sets, until a later comment for the same rule.  
  `on` returns the level of the config.

Vale's forms work too:

- `<!-- vale off -->` and `<!-- vale on -->`;
- `<!-- vale Style.Rule = NO -->` and `YES`, or `off` and `on`;
- `<!-- vale Style = NO -->`;
- `<!-- vale Style.Rule["a","b"] = NO -->` for the matches `a` and `b`;
- `<!-- vale styles = A, B -->` to run only those styles.

A comment acts on what follows it.  
A comment inside a paragraph, a list item, a heading or a table cell turns the rule off for that whole element.  
An alert between the comment that turns a rule off and the one that turns it on is left out.

The words must be exactly as shown, with the spaces around `:` and `=`, and case matters.  
A comment in a code block or a code span is text.  
An MDX file reads `{/* hakkiri off */}` and `{/* vale off */}` in the same way, as a block of its own or inside a line.  
A source file reads no comment control.

### MDX expressions

An MDX file's `{…}` expressions, and its JSX (JavaScript XML) elements with no children, read as code, as in Vale.  
A `code` rule reads `{x}` and `<Tag a="x" />`, and no text rule does.

The tags of an element with children and a `{/* … */}` comment read as no node.  
An open and a close tag with nothing between them inside a line of text are no code.

### Math

Math in a Markdown or MDX file reads as code, as in Vale.  
So no text rule reports it, and a `code` rule reads it.

An inline span starts at a `$` that a non-space follows.  
It ends at a `$` that follows a non-space and has no digit next to it.  
So `$5 and $10` is money and `$x$` is math.  
A span may wrap over lines of a paragraph.

A display block opens at a line that starts with `$$`, even in the middle of a paragraph.  
It closes at a line that ends with `$$`, a blank line or the end of the paragraph.  
The text before and after reads as separate paragraphs.  
A line indented four columns or more past its container opens no block.

### Front matter

The top-level string values of a Markdown or MDX file's front matter are text, as in Vale.

- YAML is between `---` lines, after a `---yaml` line, or closed by `...`.
- TOML is between `+++` lines or after a `---toml` line.
- JSON is between `;;;` lines, after a `---json` line, or an object that opens with `{` on the first line.

The block is no body text, so no rule of the body reads it.  
A rule scoped `text`, `frontmatter`, `text.frontmatter`, `frontmatter.<key>` or `text.frontmatter.<key>` reads a value.  
A `paragraph` or `sentence` rule does not.

Some parts are not read: a key, a number, a boolean, a null, a date, a list and a nested mapping.  
Nor is a string below a TOML table, or a YAML alias.

A YAML key that is a list, a mapping or an alias gives no value, and drops none of the entries after it.  
Each key written twice keeps all of its values.  
A number or null key does not hide the values after it.

An anchor or a tag before a value changes nothing, and a block or token pattern leaves the front matter as written.  
A block that no parser reads fails the file with `front matter: line N: …` and exit 2.  
The rest of the file is linted, where Vale lints none of it.  
A `hakkiri test` case with such a block cannot run.

### `styles`

`styles` lists the styles whose rules run, and a list replaces the one of a lower rank.  
An empty list turns linting off for those files, and drops every rule setting of a lower rank.

### Rule settings

A rule name, such as `ProseStyle.BareURL`, sets that rule's level to `suggestion`, `warning` or `error`.  
It can also turn the rule `on` at the level of its rule file, or turn it `off`.  
A level or `on` also runs the rule when no glob lists its style, as `Team.Terms: warning` does.  
`off` on such a rule changes nothing.

A level stays when a glob of higher rank turns the rule `off` and `on` again.  
The switch and the level compete as two keys.  
A name that no loaded style contains is an error that shows the name and its line.

Some rules hakkiri cannot run: a `script` rule, or one with a `doc()` scope.  
Such a rule loads no rule, and the rest of its style runs.

It makes the run exit 2 with a `failed` line when a glob lists its style, or a level or `on` turns it on.  
This goes on until the config turns it `off`.  
In a style that no glob lists, it is ignored.

### `params`

`params` sets fields of rules, as Vale's `Style.Rule[param] = value` does.  
It maps a rule name to a mapping from a field name to one value: a number, `true` or `false`, or text.  
`Vale.Repetition: {max: 1}` lets one repeat pass.  
The rule is read again with the field replaced.  
A value reads as the field's kind, so `metrics: Gunning Fog` is a list of one.

A parameter reaches the files of the glob that sets it.  
A glob of higher rank wins for the same parameter.  
An empty `styles` list drops the parameters of a lower rank.  
Vale applies a parameter to every file, whichever section sets it, and the last section wins everywhere.

These parameters are errors, each on its line, and all are reported at once:

- a level;
- a field that is a list, a mapping, the rule's identity or its prose: `tokens`, `forms`, `swap`, `exceptions`, `raw`, `either`, `filters`, `ignore`, `message`, `description`, `link`, `extends`, `name`, `path` and `tests`;
- a field the rule's check does not define;
- a value of the wrong kind, a flag that is not `true` or `false`, and a missing value;
- a rule that no loaded style contains, and any parameter of `Vale.Terms` or `Vale.Avoid`.

Vale accepts some of them with no message, and shows no line for others.

## Rule scopes

A rule's `scope` is Vale's.  
It is lowercase dotted names such as `text`, `heading.h1` and `sentence`, joined by `~`, `&` and lists.  
`&` keeps the blocks that have every name, so `sentence & paragraph` selects the sentences of paragraphs.

### `title` and `image`

hakkiri adds two scopes that Vale lacks.  
`image` selects the alt text of an image.  
`title` selects these titles:

- the title of a Markdown link, of an image and of a link reference definition;
- the `title` attribute of an HTML tag, in a block or in a line of text;
- the `title` attribute of a JSX element.

A title is selected by the element around it too.  
`title & link` selects the title of a link, and `title & image` the title of an image.  
Only `title` selects the title of a definition.  
In HTML, `title & link` selects the `title` attribute of an `a` tag, and `title & image` that of an `img` tag.

`title & ~link` leaves out the titles of links, and `title & ~image` those of images.  
Inside the text of a link, a tag's title belongs to the tag, so `title & link` leaves it out.  
`link` and `image` alone never select a title, and no other scope reads one.  
In Vale only `raw` reads a title, and `raw` still reads every title here.

### Languages in scopes

The last piece of a part can be a language, when the whole part is no scope and the rest of it is one.  
The language is given by any name, alias or extension of it in the GitHub Linguist table.

`raw.rs` and `raw.rust` select the `raw` text of a Rust file.  
`text.comment.js` selects the comments of `.js`, `.jsx`, `.mjs` and `.cjs` files.

A negated part with a language, such as `~heading.rs`, leaves out nodes only in that language.  
An item whose every part is for another language selects nothing.  
A piece that is no language is an error, and `text.comment.line` stays the scope of line comments.

A node is in the language of its file.  
When `formats` chose the reader, the node is also in the reader's language.  
So `text.txt` selects only a file whose language is Text.  
It never selects a `.rst` file or a `README` that is read as plain text.

## Fenced code blocks

A fenced code block takes the language of its info string, and `rs` and `rust` are one.  
For Rust, TypeScript, TSX or JavaScript, hakkiri parses the code of the fence with that grammar.

Each comment is read as Markdown at its place in the file, as in a source file.  
The nodes of that comment are in the language of the fence.  
So `text.comment.rs` selects the comments of Rust fences and no others.

Some fences give no text, as in Vale.  
These are a fence of Markdown, MDX or plain text, a fence with no info string, a fence of another language, and an indented block.

A fence whose code does not map line for line onto the file gives none either.  
One example is a fence with a tab after a block quote marker.  
A fence in a source file's comment is no code.

## Differences from Vale

### Commands and output

`hakkiri test` rejects a case that sets `view`, reads no `.yaml` file, and prints its report in its own layout.

`hakkiri check` has no `--glob`, `--filter`, `--ext`, `--sort`, `--relative` or `--no-global`.  
It reads no standard input.  
It prints the level in each line, and its JSON is one object instead of a map from path to alerts.  
`--fail-level` takes the place of `--no-exit`.

### Packages

hakkiri fetches a package pinned by `url` and `sha256` on the first run.  
It keeps the package in a cache folder outside the project.  
Vale needs `vale sync` instead, and copies the styles into `StylesPath`.  
hakkiri rejects a download that does not match its hash, and Vale installs any zip.

### Files and languages

hakkiri reads a plain-text file as paragraphs, and skips a binary file.  
Vale reads a file with no reader as lines of one block.  
hakkiri reads the comments of a fenced code block in Rust, TypeScript, TSX or JavaScript.  
Vale reads no text inside a fence.

A scope part is a language of the Linguist table.  
Vale compares it with the extension after folding `.tsx`, `.mts` and `.cts` into `.ts`.  
So in Vale, `text.comment.ts` does not reach a `.tsx` file.

For a file with no extension, hakkiri reads the interpreter on its first line, where Vale lints none of it.  
It takes a project's `formats` entry over a package's, where Vale keeps the first value.

### Settings

hakkiri writes `on` and `off` where Vale writes `YES` and `NO`.  
A key that is no rule name is an error.  
So is a word that is neither a level, `on` nor `off`.  
Vale ignores the key, and for the word it warns and turns the rule off.

### Spelling and rules

For `Vale.Spelling`, hakkiri reads the `config/dictionaries` and `config/ignore` folders of each package.  
Vale reads one styles path.  
hakkiri reads no word in a code span of a code comment, which Vale reads when `custom` turns the filters off.  
It also reads no name after `@param`, which Vale reads.

hakkiri runs no `script` rule, no `doc()` scope and no View, and reads no `.vale.ini`.  
Its built-in style `Vale` contains `Vale.Repetition` and `Vale.Spelling`, as Vale's does.

### Sequence forms

hakkiri reads a `sequence` rule with `forms` in place of `tokens`.  
`forms` is a list of mappings that each contain `tokens`.  
The rule matches where any form does, and every alert carries the one rule name.  
Two forms that match overlapping words give the match that starts first, and the earlier form at a tie.

A rule has `tokens` or `forms`, never both.  
Vale stops with a panic when a rule has `forms` and no `tokens`.  
So a package that must load in both tools keeps one rule for each form.

### Tags

To tag a sentence, hakkiri uses the model of prose at `b33e60d`.  
Vale tags with the older model of prose v3.2.2.  
hakkiri then corrects these tags:

- a possessive `'s`, a modal or a name in capitals, and a semicolon;
- a verb after a noun phrase;
- the verbs of saying and asking after a noun.

The words in `the docs say` and `the flag asks for` read as verbs.  
The words in `for the asks that wait` and `a hook told of the file` read as a noun and a participle.  
`check`, `prompt`, `thinking` and `fade` read as nouns after a determiner.  
In `the load split pull request`, the words `prompt`, `round` and `split` read as nouns.

`check` reads as a noun that ends a compound, as in `the peer user id check`.  
A `sequence` rule can therefore flag other words than in Vale.  
A title keeps the tags of the model alone.

### Code spans

In a `sequence` rule, hakkiri reads a code span as a noun, and its tag is `NN`.  
The tagger reads a noun in its place.  
A pattern or a word list reads the span as the letter `X`.

A rule for a noun and a verb, such as `ThingsSpeakNoun`, flags `` `flag` says `` as it flags `the flag says`.  
A rule for four lowercase nouns counts no code span.  
Vale reads the span as a run of `*`, and a rule with a tag and no pattern flags `` `flag` says `` there too.

An `existence` rule reads a code span as one `*`, as Vale masks it.  
So a match that runs across a span is a match, and one that starts or ends on a span is not.  
The square brackets of an intra-doc link, as in ``[`Name`]``, belong to the span.

### Sentences

hakkiri splits sentences where Vale does not in a few places, and keeps whole what Vale splits in others.

- A full stop after a code span ends a sentence, and a code span begins one after a full stop.  
  So ``Use `a`. Then use `b`. `c` is last.`` is three sentences.
- A unit of measure after a number ends a sentence before a capital word, as in `92 ms. Tests`.
- `vs.` ends no sentence, before a capital or a number.  
  So `Rust 1.99 vs. Go 1.25 differ.` is one sentence, where Vale ends the first after `vs.`.
- A date such as `2026-09-29.` is no list number, so the sentence after it begins a new one.
- Vale reads the `1.` of a numbered heading or item as a sentence of its own.
- Vale keeps a version, a decimal or a task number whole before a capital, where hakkiri ends the sentence.  
  An example is `ctl-core 0.5.2. ctl-core 0.6.3 ships`.
