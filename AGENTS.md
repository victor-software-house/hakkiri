# hakkiri

A prose linter for Markdown, MDX, and source comments, built on ctl-core.  
This file is the development contract.  
The operator contract is `hakkiri instructions` (`src/instructions.md`) and `skills/hakkiri/SKILL.md`.

## Decisions

1. **Vale rule files, own config.**
   hakkiri reads Vale rule files as they are, and its own `.ctl/hakkiri.yaml`, not `.vale.ini`.  
   A rule selects text with Vale's `scope`: its lowercase dotted names, such as `sentence` and `text.comment.doc`, and its operators `~`, `&` and lists.  
   Its checks are Vale's check types except `script`; a rate is a `metric` whose formula reads `occurrences`.  
   A Vale style folder loads with no conversion: a rule may extend another rule, as in Vale.  
   A rule hakkiri cannot run, such as a `script` rule, is reported and the rest of its style runs, until the config turns it `off`.  
   `hakkiri import vale` will convert a `.vale.ini` into the project config (row `HKR-035`).
2. **Structure first.**
   A rule selects text by its scope in the document tree; a pattern only narrows that selection.  
   A rule that needs a regular expression to find whether text sits in code, a link or a comment shows a missing scope.
3. **Language profiles.**
   Each source language has a profile that maps its comments and doc comments into the tree.  
   Every comment reads as Markdown, plain `//` comments included; TSDoc and JSDoc keep their tags.  
   String literals are never prose.  
   hakkiri ships Rust, TypeScript, JavaScript, Go, Swift, YAML, TOML, Shell, Markdown and MDX.  
   A language that hakkiri reads only by its comments is one dialect in `src/comments.rs`: its grammar, its comment nodes, its marker and its tool directives.  
   A file, a fenced code block and a comment have a language from langdex, the table of languages, and a node has its own in `Node.language` when it has one.  
   A reader is one of the twelve parsers, and `Language` maps each to the langdex language it reads, by name.  
   A file reaches the reader of its langdex language, so `.yml`, `.bash`, a `#!/bin/sh` script and a `bashrc` file reach the YAML and Shell readers, and no table lists extensions.  
   A fenced code block of Rust, TypeScript, TSX, JavaScript, Go, Swift, YAML, TOML or Shell is parsed by that grammar, and `src/fence.rs` puts each comment in the tree at its place in the Markdown or MDX file, with the language of the fence.  
   Each line of the fence's code must end the line of the file it came from, or the fence gives no nodes.  
   A fence of Markdown, MDX or text, a fence with no info string, a fence of another language and an indented block stay gaps.  
   The text of a comment in a source file reads its fences as gaps.
4. **Matching.**
   The `regex` crate by default; `fancy-regex` with a backtrack limit only for a pattern that needs lookaround.  
   A limit reached is an error alert for that file, never a panic.
5. **Packages.**
   Rules ship as versioned packages that a project pins by version and hash.  
   hakkiri fetches and caches them; there is no separate sync step.  
   A package entry is a local `path` to a folder of style folders, or a `url` and its `sha256`.  
   A `url` names a zip that hakkiri downloads on the first run, checks against the hash, and unpacks whole into `<cache>/<sha256>/`, where the per-user cache folder or `--cache-dir` is the cache.  
   A later run reads that folder with no network.  
   The zip is a Vale package kept as it lays out: one top folder that holds a `styles` folder or is a style, so `HKR-007` and `HKR-008` find its files where Vale puts them.
   A package may ship `.ctl/hakkiri.yaml` in its top folder, beside `styles`, in the format of the project config (operator, 2026-11-11).  
   hakkiri reads the settings of each package under the project's own, as Vale reads a package's `.vale.ini` under the project's.  
   The globs that match a file apply as a cascade (operator, 2026-11-13): settings of different keys merge, and for one key the highest rank wins.  
   Rank is specificity, then origin (each package in load order, the project last), then the order written, and each file's top level ranks below every glob.  
   A glob with `{…}` groups ranks, for each file, as the most specific of its alternatives that matches that file (row `HKR-072`).  
   Only the project forces a value, with the YAML tag `!override`, read through ctl-core's `Tagged` and applied after every value without the tag. A package's file that holds it, or any other tag, is a fault.  
   A `styles` list adds to a package's under the same glob text and replaces it under another by rank, where Vale applies its sections in the order written.  
   A package's `packages` are fetched, pinned by `url` and `sha256`, to a depth of 10.  
   A package may hold vocabularies in `config/vocabularies/<name>/`, which the `vocab` key names at the top level and in a glob of `files` (operator, 2026-11-11).  
   The top level reaches every check that reads accepted terms, and a glob's list reaches the files it matches, as Vale's `Vocab` does.  
   The lists build `Vale.Terms` and `Vale.Avoid` in a built-in style `Vale`, and a glob's vocabulary builds `Vale.<Name>.Terms` and `Vale.<Name>.Avoid` for its files.  
   A package folder named `Vale` is skipped with one warning, as Vale passes over it, so the built-in style wins and the other styles of the package lint.  
   The style `Vale` always exists and holds `Vale.Repetition`, and `Vale.Spelling` when a glob uses the style, each read from Vale's definition as text by `vale_repetition` and `vale_spelling` (rows `HKR-038` and `HKR-026`).  
   A glob uses the style when it lists `Vale` in `styles`, names a rule that starts with `Vale.`, or sets `params` for one.  
   `Named::add_layer` finds it before the packages load, so a dictionary that cannot load fails no run of a config that spells nothing.  
   `Vale.Spelling` is the one rule named for Vale in `rule/package.rs`: it reads `config/dictionaries` and `config/ignore` of every package, where a rule file reads its own.  
   `read_lists` refuses an entry that a link sends outside the package, as `find` does, and reads each real folder once, so a link loop ends.  
   A hakkiri config that lists `Vale` and whose Vale config turns spelling off sets `Vale.Spelling: off` too, or the fixture gives alerts that Vale does not.  
   Repetition compares words by simple case folding, as Go's `EqualFold` does, never by lowercasing the whole word.  
   A glob's `params` set fields of rules by name, as Vale's `Style.Rule[param] = value` does (row `HKR-059`).  
   `Rule::with_params` reads a rule again from the text it came from, with the parameters laid over its fields as `extends` lays a child over its parent, so each check accepts the fields it defines and the rule file's kinds.  
   A parameter belongs to its glob, where Vale applies it to every file and lets the last section win everywhere.  
   `Project::plan` picks the rule read with the parameters that add up for the file, from the sets `build_variants` read when the config loaded.  
   A vocabulary folder holds `accept.txt`, `reject.txt`, `vocab.yml` and `vocab.yaml`, all read (row `HKR-041`).  
   `src/vocabulary/yaml.rs` parses a YAML file through ctl-core's input and checks each entry on its node, so one run reports every wrong entry on Vale's line.  
   A plain YAML term is literal, `affix` names an affix file in the packages' `config/dictionaries` folders (the holder first, never `$DICPATH`), and a key the file does not define is an error.  
   With `ignorecase` the period of a term is literal too, where Vale keeps it a wildcard.  
   A glob's `block_ignores` and `token_ignores` hide what their patterns match from the prose rules, as Vale's `BlockIgnores` and `TokenIgnores` do (row `HKR-043`).  
   hakkiri rewrites a Markdown or MDX file as Vale does, parses the result, and gives every node the bytes of the file it came from, so alerts point at the file as written.  
   A source file takes them in the Markdown of each comment, where Vale applies none.  
   An HTML comment of a Markdown file, or a `{/* */}` comment of an MDX file, turns rules off and on with Vale's reach (row `HKR-031`, operator, 2026-11-11).  
   The forms are `hakkiri off` and `on`, `hakkiri Style.Rule: off` and `on`, a level, and Vale's own.  
   `src/control.rs` reads the comments, leaves the text between `off` and `on` out of the tree, gates each block where Vale reads it, and drops an alert that lies between two comments.  
   Vale reads a block when its element ends, or earlier where a tag that opens a block flushes the text so far: a nested list in a tight item, a `div` inside a paragraph.  
   `Node::closes` and `Node::flushes` hold those bytes, and the engine gates each alert at its own.  
   Math reads as code by Pandoc's rules, as in Vale: `src/math.rs` finds each `$…$` span and `$$…$$` block, masks the file at the same length, and parses it again (row `HKR-048`).  
   Front matter values are nodes of the scope `frontmatter`, with their key as label (row `HKR-049`).  
   `src/frontmatter.rs` takes the top-level strings of YAML, TOML and JSON from ctl-core's `Input::top_level`, which refuses nothing that YAML allows.  
   A block that no parser reads gives no value and fails the file with the line of the parser's problem, and the rest of the file is linted, where Vale lints none of it.  
   The ignore rewrite leaves the front matter as written.  
   Every `path` and `files` glob is relative to the project root, the folder that holds `.ctl`, so a file matches the same glob however it is named.  
   A glob reads by the rules of gitignore through `config::standard_glob`, for `files` and `formats` alike: `*` stays inside one folder, and a glob with no `/` matches the file name in any folder (operator, 2026-11-14).  
   Vale matches the path as given; the departure is in the `vale-differences` spec.
6. **Exit status.**
   `hakkiri check` exits as Vale does: 2 when the config, a rule or a file failed, then 1 when a reported alert reaches `--fail-level` (default `error`), then 0.  
   `--fail-level` takes the place of Vale's `--no-exit`.  
   `fail_level`, `gitignore`, `on_unmatched` and `on_unknown` are flags, variables and config keys, resolved in that order and then the default; only the project's config holds the keys, and a package's settings file that does is an error.  
   A walk is the `ignore` crate's, with git rules only: it skips what git ignores unless `gitignore` is false, and a named file is never filtered.  
   A file has the langdex language of its path: its file name, then its extension.  
   A path that names none leaves the interpreter on the first line, in at most the first 4096 bytes, and then the file has no language.  
   A file is read by the reader its `formats` entry gives, then by the reader of its language, then as `on_unknown` says.  
   A Text file reads as plain text whatever `on_unknown` says.  
   A selected file with no reader is plain text (`Language::Text`, one `raw` root of paragraphs), unless `on_unknown` changes that.  
   `code_span_size` is the words a code span counts as, 1 by default and 0 as Vale counts it.  
   It is a key of a package's file, the project and a glob, then `HAKKIRI_CODE_SPAN_SIZE`, then `--code-span-size`, each beating the one before.  
   `Plan` holds the value of each file, and `Text::code_sized` reads the text of a `metric` or `readability` rule with it.  
   A rule that reads a match never sees it.  
   `formats` looks a file up as Vale's `formatFor` does: the extension, then the name or a glob, in key order.  
   It names a reader by any langdex name of the language it reads, so `md` and `markdown` are one.  
   A node outside a fence is in the file's own language, and in its reader's when `formats` chose the reader, where Vale keeps the file's own extension.  
   The tree records the reader, and a scope may name either language.  
   A plain-text file is binary, and skipped, when its first 8192 bytes hold a NUL byte.  
   A byte that is not UTF-8 reads as one `?`, so a Latin-1 file is linted at the byte offsets Vale gives.  
   The pretty output is a line per alert with no colour and no frame, because tools read it.
7. **Test cases.**
   A rule file holds its cases under `tests`, and a `*.test.yml` file is a list of them. `check` loads cases and never runs them.  
   `hakkiri test` loads one rule alone for a case in a rule file or a case with `rule`, and lints any other case through the project config.  
   `--coverage` fails for a rule that flags nothing in any case.
8. **Release scope.**
   Release 1 is the CLI: `check` and `test`, run from a mise task, CI and a pre-commit hook.  
   Release 2 is the language server, on the same per-file engine.
9. **Releases are GitHub Release tarballs only.**
   `.github/workflows/release.yml` calls verctl's reusable release workflow, and `.ctl/ver.yaml` declares the package with `publisher: none`, so a release is a tag, a GitHub Release and two tarballs with their checksums.  
   Each tarball builds natively on a free GitHub-hosted runner: `linux-x64` on `ubuntu-26.04`, and `darwin-arm64` on `xcode-27`, the labels verctl's built-in targets name.  
   The `linux-x64` binary is a static musl build (operator, 2026-11-11), so it starts on a machine with no glibc loader, NixOS without nix-ld included.  
   `mise run release:prepare` installs the Rust target before that build, and the C compiler for it is the `zig` that `mise.toml` pins, set as `CC_x86_64_unknown_linux_musl` in `mise.toml`, so no apt package is installed.  
   The musl build links mimalloc as its global allocator (operator, 2026-11-11), because musl's own allocator makes a parallel `check` about eight times slower.  
   No other build links it.  
   Each change that ships behaviour adds a `.changeset/` fragment; merging the Version PR is the release.  
   The README's install line and the skill's `version:` line are version pins, so never edit their numbers by hand.
10. **Tags.**
    The English tagger is the model of prose at `b33e60d` with the corrections of `src/tagger/corrections.rs` applied after it, each a rule with its reason.
    `tag_treebank` skips them, so a title is tagged as prose tags it.
    A new correction needs a test, and a measure on UD English GUM, on the judged conflicts of our documents and on the alerts of a rule that reads tags.
    A `sequence` rule reads a code span as a noun: the tagger reads the word `name` in its place, its tag is `NN`, and a pattern reads it as `X`, so a lowercase pattern refuses it.
    An `existence` rule reads a code span as one `*` and drops a match only when it starts or ends on the span.
    `nouns_of_technical_prose` reads three lists of nouns that the model tags as another part of speech, one for each place a noun is certain.
    Add a word to a list only with the sampled readings of that word in that place and the alerts that need it, and run the GUM count.
11. **Sequence forms.**
    A `Sequence` holds `forms`, each a `Form` with its own anchor and candidate starts, and the rule's `exceptions`.
    `find_tagged` walks each form over the one tagged sentence, and `merged` joins the matches.
    One form's matches stay as found, which keeps a `tokens` rule at Vale's alerts.
    A match that overlaps one kept from another form is dropped.
12. **Sentence splitting.**
    `Model::split_code` takes the byte ranges of the code spans, and `split` is the call with none.
    A code word is a sentence of its own after a full stop, never an abbreviation, and never an initial.
    `UNIT_SYMBOLS`, `SI_PREFIXES` and `BINARY_PREFIXES` in `src/splitter/token.rs` cite their sources, and the unit rule needs a number before the unit.
    `vs.` ends no sentence, and a date is no list number.
    A change needs a case in `src/splitter/tests.rs` and, where Vale differs, a case in `tests/sentence_differences.rs`.

## Workflow

- Work is queued in `tasks.yaml` and moved with qctl; never edit the ledger by hand.
- Each queued slice is an OpenSpec change under `openspec/changes/`, archived into `openspec/specs/` when it merges.
- The repository is rendered from the shared Copier template; change rendered files through a template update.

## Rules

- ctl's shared rules apply. Every change goes through a pull request, and `--no-verify` is never used.  
  ctl-core owns presentation, every flag has a `HAKKIRI_` variable, and nothing local leaks into a tracked file.
- The pull request's CI verifies. Do not build or test before a push.
- A change that makes hakkiri differ from Vale, or removes a difference, updates [`openspec/specs/vale-differences/spec.md`](openspec/specs/vale-differences/spec.md) in the same pull request.
- A behaviour change moves code, tests, `src/instructions.md`, `skills/hakkiri/SKILL.md`, this file and its OpenSpec change together (the ctl bundle contract).
- Prose in this repository follows ProseStyle: plain words, short sentences, no dashes as punctuation.
