# hakkiri

A prose linter for Markdown, MDX and source comments, built on ctl-core.
This file is the development contract.
The operator contract is `hakkiri instructions` (`src/instructions.md`) and `skills/hakkiri/SKILL.md`.

## Decisions

### 1. Vale rule files, own config

hakkiri reads Vale rule files as they are, and its own `.ctl/hakkiri.yaml`, not `.vale.ini`.
A rule selects text with Vale's `scope`.
That means its lowercase dotted names, such as `sentence` and `text.comment.doc`, and its operators `~`, `&` and lists.
Its checks are Vale's check types except `script`.
A rate is a `metric` whose formula reads `occurrences`.

A Vale style folder loads with no conversion, and a rule may extend another rule, as in Vale.
A rule hakkiri cannot run, such as a `script` rule, is reported, and the rest of its style runs.
This goes on until the config turns the rule `off`.
`hakkiri import vale` will convert a `.vale.ini` into the project config (row `HKR-035`).

### 2. Structure first

A rule selects text by its scope in the document tree, and a pattern only narrows that selection.
Some rules need a regular expression to find whether text is in code, a link or a comment.
Such a rule shows that a scope is missing.

### 3. Language profiles

Each source language has a profile that maps its comments and doc comments into the tree.
Every comment reads as Markdown, plain `//` comments included.
TSDoc and JSDoc keep their tags.
String literals are never prose.
hakkiri ships Rust, TypeScript, JavaScript, Go, Swift, YAML, TOML, Shell, Markdown and MDX.

Some languages are read only by their comments.
Each of them is one dialect in `src/comments.rs`: its grammar, its comment nodes, its marker and its tool directives.
A file, a fenced code block and a comment have a language from langdex, the table of languages.
A node has its own language in `Node.language` when it has one.

A reader is one of the twelve parsers.
`Language` maps each reader to the langdex language it reads, by name.
A file goes to the reader of its langdex language, and no table lists extensions.
So `.yml`, `.bash`, a `#!/bin/sh` script and a `bashrc` file go to the YAML and Shell readers.

A fenced code block of Rust, TypeScript, TSX (TypeScript with markup), JavaScript, Go, Swift, YAML, TOML or Shell is parsed by that grammar.

`src/fence.rs` puts each comment where it is in the Markdown or MDX file, with the language of the fence.
Each line of the fence's code must end the line of the file it came from, or the fence gives no nodes.

Some blocks stay gaps.
These are a fence of Markdown, MDX or text, a fence with no info string, a fence of another language, and an indented block.
The text of a comment in a source file reads its fences as gaps.

### 4. Matching

hakkiri uses the `regex` crate by default.
It uses `fancy-regex` with a backtrack limit only for a pattern that needs lookaround.
A limit reached is an error alert for that file, never a panic.

### 5. Packages

Rules ship as versioned packages that a project pins by version and hash.
hakkiri fetches and caches them, and there is no separate sync step.
A package entry is a local `path` to a folder of style folders, or a `url` and its `sha256`.

A `url` points to a zip that hakkiri downloads on the first run and checks against the hash.
hakkiri then unpacks it whole into `<cache>/<sha256>/`.
The cache is the per-user cache folder, or `--cache-dir`.
A later run reads that folder with no network.

The zip is a Vale package, kept as it is laid out.
It has one top folder, which contains a `styles` folder or is a style.
So `HKR-007` and `HKR-008` find its files where Vale puts them.

#### Package settings

A package may ship `.ctl/hakkiri.yaml` in its top folder, beside `styles`, in the format of the project config (operator, 2026-11-11).
hakkiri reads the settings of each package under the project's own.
Vale does the same with a package's `.vale.ini`.

The globs that match a file apply as a cascade (operator, 2026-11-13).
Settings of different keys merge, and for one key the highest rank wins.
Rank is specificity first, then origin: each package in load order, and the project last.
Then comes the order written, and each file's top level ranks below every glob.

A glob with `{…}` groups ranks, for each file, as the most specific of its alternatives that matches that file (row `HKR-072`).

Only the project forces a value, with the YAML tag `!override`.
ctl-core's `Tagged` reads the tag, and the value applies after every value without the tag.
A package's file that has this tag, or any other tag, is a fault.

A `styles` list adds to a package's list under the same glob text.
Under another glob text it replaces that list by rank, where Vale applies its sections in the order written.
A package's `packages` are fetched, pinned by `url` and `sha256`, to a depth of 10.

#### Vocabularies

A package may store vocabularies in `config/vocabularies/<name>/` (operator, 2026-11-11).
The `vocab` key lists them at the top level and in a glob of `files`.
The top level reaches every check that reads accepted terms.
A glob's list reaches the files it matches, as Vale's `Vocab` does.

The lists build `Vale.Terms` and `Vale.Avoid` in a built-in style `Vale`.
A glob's vocabulary builds `Vale.<Name>.Terms` and `Vale.<Name>.Avoid` for its files.
A package folder called `Vale` is skipped with one warning, as Vale passes over it.
So the built-in style wins, and the other styles of the package lint.

#### The style Vale

The style `Vale` always exists and contains `Vale.Repetition`.
It also contains `Vale.Spelling` when a glob uses the style.
`vale_repetition` and `vale_spelling` read each rule from Vale's definition as text (rows `HKR-038` and `HKR-026`).

A glob uses the style when it lists `Vale` in `styles`, lists a rule that starts with `Vale.`, or sets `params` for one.
`Named::add_layer` finds it before the packages load.
So a dictionary that cannot load fails no run of a config that spells nothing.

`Vale.Spelling` is the one rule written for Vale in `rule/package.rs`.
It reads `config/dictionaries` and `config/ignore` of every package, where a rule file reads its own.
`read_lists` rejects an entry that a link sends outside the package, as `find` does.
It reads each real folder once, so a link loop ends.

Some hakkiri configs list `Vale` while their Vale config turns spelling off.
Such a config sets `Vale.Spelling: off` too, or the fixture gives alerts that Vale does not.
Repetition compares words by simple case folding, as Go's `EqualFold` does, never by lowercasing the whole word.

#### Parameters

A glob's `params` set fields of rules by name, as Vale's `Style.Rule[param] = value` does (row `HKR-059`).
`Rule::with_params` reads a rule again from the text it came from.
It lays the parameters over the rule's fields, as `extends` lays a child over its parent.
So each check accepts the fields it defines and the rule file's kinds.

A parameter belongs to its glob.
Vale instead applies it to every file and lets the last section win everywhere.
`Project::plan` picks the rule read with the parameters that add up for the file.
It picks from the sets that `build_variants` read when the config loaded.

#### Vocabulary files

A vocabulary folder contains `accept.txt`, `reject.txt`, `vocab.yml` and `vocab.yaml`, all read (row `HKR-041`).
`src/vocabulary/yaml.rs` parses a YAML file through ctl-core's input and checks each entry on its node.
So one run reports every wrong entry on Vale's line.

A plain YAML term is literal.
`affix` gives an affix file in the packages' `config/dictionaries` folders: the package that has the vocabulary first, never `$DICPATH`.
A key the file does not define is an error.
With `ignorecase` the period of a term is literal too, where Vale keeps it a wildcard.

#### Ignores and comments

A glob's `block_ignores` and `token_ignores` hide what their patterns match from the prose rules (row `HKR-043`).
Vale's `BlockIgnores` and `TokenIgnores` do the same.

hakkiri rewrites a Markdown or MDX file as Vale does, and parses the result.
It gives every node the bytes of the file it came from, so alerts point at the file as written.
A source file takes the ignores in the Markdown of each comment, where Vale applies none.

A comment turns rules off and on with Vale's reach (row `HKR-031`, operator, 2026-11-11).
In a Markdown file it is an HTML comment, and in an MDX file it is a `{/* */}` comment.
The forms are `hakkiri off` and `on`, `hakkiri Style.Rule: off` and `on`, a level, and Vale's own.

`src/control.rs` reads the comments and leaves the text between `off` and `on` out of the tree.
It gates each block where Vale reads it, and drops an alert that lies between two comments.

Vale reads a block when its element ends.
It reads it earlier where a tag that opens a block flushes the text so far.
Two such tags are a nested list in a tight item and a `div` inside a paragraph.
`Node::closes` and `Node::flushes` store those bytes, and the engine gates each alert at its own.

#### Math and front matter

Math reads as code by Pandoc's rules, as in Vale (row `HKR-048`).
`src/math.rs` finds each `$…$` span and `$$…$$` block.
It masks the file at the same length and parses it again.

Front matter values are nodes of the scope `frontmatter`, with their key as label (row `HKR-049`).
`src/frontmatter.rs` finds the block and reads its top-level values through the `front-matter` crate.
It uses one parser per format, in every form it finds.
It blanks the block out of the text before markdown-rs parses the body.

A block that no parser reads gives no value.
It fails the file with the line of the parser's problem, and the rest of the file is linted.
Vale lints none of that file.
The ignore rewrite leaves the front matter as written.

#### Paths and globs

Every `path` and `files` glob is relative to the project root, the folder that contains `.ctl`.
So a file matches the same glob however it is written on the command line.

`config::standard_glob` reads a glob by the rules of gitignore, for `files` and `formats` alike (operator, 2026-11-14).
`*` stays inside one folder, and a glob with no `/` matches the file name in any folder.
Vale matches the path as given, and the `vale-differences` spec records this difference.

### 6. Exit status

`hakkiri check` exits as Vale does.
It exits 2 when the config, a rule or a file failed.
Then it exits 1 when a reported alert reaches `--fail-level` (default `error`), and otherwise 0.
`--fail-level` takes the place of Vale's `--no-exit`.

`fail_level`, `gitignore`, `on_unmatched` and `on_unknown` are flags, variables and config keys.
hakkiri resolves them in that order, and then the default.
Only the project's config has the keys, and a package's settings file that has them is an error.

#### Walks and readers

A walk is the `ignore` crate's, with git rules only.
It skips what git ignores unless `gitignore` is false, and a file given by path is never filtered.

A file has the langdex language of its path: its file name, then its extension.
When the path gives none, hakkiri reads the interpreter on the first line, in at most the first 4096 bytes.
After that the file has no language.

A file is read by the reader its `formats` entry gives, then by the reader of its language, and then as the `on_unknown` setting picks.
Whatever `on_unknown` is set to, a Text file reads as plain text.
A selected file with no reader is plain text, unless `on_unknown` changes that.
Plain text is `Language::Text`, one `raw` root of paragraphs.

#### Code span size

`code_span_size` is the number of words a code span counts as.
It is 1 by default, and 0 counts a code span as Vale does.
It is a key of a package's file, the project and a glob.
After those come `HAKKIRI_CODE_SPAN_SIZE`, then `--code-span-size`, each beating the one before.

`Plan` stores the value of each file.
`Text::code_sized` reads the text of a `metric` or `readability` rule with it.
A rule that reads a match never sees it.

#### Formats

`formats` looks a file up as Vale's `formatFor` does: the extension, then the name or a glob, in key order.
It picks a reader by any langdex name of the language it reads, so `md` and `markdown` are one.

A node outside a fence is in the file's own language.
When `formats` chose the reader, the node is in the reader's language as well.
Vale keeps the file's own extension here.
The tree records the reader, and a scope may give either language.

#### Bytes and output

A plain-text file is binary, and skipped, when its first 8192 bytes contain a null (NUL) byte.
A byte that is not UTF-8 reads as one `?`, so a Latin-1 file is linted at the byte offsets Vale gives.
The pretty output is a line per alert with no colour and no frame, because tools read it.

### 7. Test cases

A rule file stores its cases under `tests`, and a `*.test.yml` file is a list of them.
`check` loads cases and never runs them.

`hakkiri test` loads one rule alone for a case in a rule file or a case with `rule`.
It lints any other case through the project config.
`--coverage` fails for a rule that flags nothing in any case.

### 8. Release scope

Release 1 is the CLI: `check` and `test`, run from a mise task, CI and a pre-commit hook.
Release 2 is the language server, on the same per-file engine.

### 9. Releases are GitHub Release tarballs only

`.github/workflows/release.yml` calls verctl's reusable release workflow.
`.ctl/ver.yaml` declares the package with `publisher: none`.
So a release is a tag, a GitHub Release and two tarballs with their checksums.

Each tarball builds natively on a free GitHub-hosted runner.
`linux-x64` builds on `ubuntu-26.04`, and `darwin-arm64` on `xcode-27`, the labels of verctl's built-in targets.
The `linux-x64` binary is a static musl build (operator, 2026-11-11).
So it starts on a machine with no glibc loader, NixOS without nix-ld included.

`mise run release:prepare` installs the Rust target before that build.
The C compiler for it is the `zig` that `mise.toml` pins, set as `CC_x86_64_unknown_linux_musl` in `mise.toml`.
So no apt package is installed.

The musl build links mimalloc as its global allocator (operator, 2026-11-11).
With musl's own allocator, a parallel `check` is about eight times slower.
No other build links it.

Each change that ships behaviour adds a `.changeset/` fragment.
Merging the Version PR is the release.
The README's install line and the skill's `version:` line are version pins, so never edit their numbers by hand.

### 10. Tags

The English tagger is the model of prose at `b33e60d`.
The corrections of `src/tagger/corrections.rs` apply after it, each a rule with its reason.
`tag_treebank` skips them, so a title is tagged as prose tags it.
A new correction needs a test and three measures.

The first measure is on the Georgetown University Multilayer (GUM) corpus of Universal Dependencies.
A second one is on the judged conflicts of our documents.
The third is on the alerts of a rule that reads tags.

A `sequence` rule reads a code span as a noun.
The tagger reads the word `name` in its place, and its tag is `NN`.
A pattern reads it as `X`, so a lowercase pattern rejects it.

An `existence` rule reads a code span as one `*`, and drops a match only when the match starts or ends on the span.

`nouns_of_technical_prose` reads three lists of nouns that the model tags as another part of speech.
There is one list for each place where a noun is certain.
Add a word to a list only with the sampled readings of that word in that place, and the alerts that need it.
Then run the GUM count.

### 11. Sequence forms

A `Sequence` contains `forms` and the rule's `exceptions`.
Each form is a `Form` with its own anchor and candidate starts.
`find_tagged` walks each form over the one tagged sentence, and `merged` joins the matches.

One form's matches stay as found, which keeps a `tokens` rule at Vale's alerts.
A match that overlaps one kept from another form is dropped.

### 12. Sentence splitting

`Model::split_code` takes the byte ranges of the code spans, and `split` is the call with none.

A code word is a sentence of its own after a full stop, never an abbreviation, and never an initial.
`UNIT_SYMBOLS`, `SI_PREFIXES` and `BINARY_PREFIXES` in `src/splitter/token.rs` cite their sources.
The unit rule needs a number before the unit.
`vs.` ends no sentence, and a date is no list number.

A change needs a case in `src/splitter/tests.rs`.
Where Vale differs, it also needs a case in `tests/sentence_differences.rs`.

## Workflow

- Work is queued in `tasks.yaml` and moved with qctl, so never edit the ledger by hand.
- Each queued slice is an OpenSpec change under `openspec/changes/`, archived into `openspec/specs/` when it merges.
- The repository is rendered from the shared Copier template, so change rendered files through a template update.

## Rules

- ctl's shared rules apply.
  Every change goes through a pull request, and `--no-verify` is never used.
  ctl-core owns presentation, every flag has a `HAKKIRI_` variable, and nothing local leaks into a tracked file.
- The pull request's CI verifies.
  Do not build or test before a push.
- A change that makes hakkiri differ from Vale, or removes a difference, updates [`openspec/specs/vale-differences/spec.md`](openspec/specs/vale-differences/spec.md) in the same pull request.
- A behaviour change moves code, tests, `src/instructions.md`, `skills/hakkiri/SKILL.md`, this file and its OpenSpec change together (the ctl bundle contract).
- Prose in this repository follows ProseStyle: plain words, short sentences, no dashes as punctuation.
