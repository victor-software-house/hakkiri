# Changelog

## hakkiri 0.0.6

### Added

- A glob of `files` takes `block_ignores` and `token_ignores`, each a list of patterns, and a package's settings file can hold them too.
  In a Markdown or MDX file, each match of a block pattern is read as a fenced code block that holds its first group.
  Each match of a token pattern is read as a code span that holds it.
  No rule that reads prose reports text inside them.
  The patterns of every glob that matches a file apply, a package's first ([`c475d5c`](https://github.com/victor-software-house/hakkiri/commit/c475d5c62b194f87982d2471c6ae1a44242dd2e5)).
- An HTML comment in a Markdown file, and a `{/* … */}` comment in an MDX file, turns rules off and on for the text after it, with the reach of Vale's comments.
  Write `<!-- hakkiri off -->` and `<!-- hakkiri on -->`, or `<!-- hakkiri Style.Rule: off -->` and `<!-- hakkiri Style.Rule: on -->` for one rule.
  `<!-- hakkiri Style.Rule: suggestion -->`, `warning` or `error` sets the level of a rule's alerts from there on.
  Vale's own comments, such as `<!-- vale off -->` and `<!-- vale Style.Rule = NO -->`, work as they do in Vale ([`16981dc`](https://github.com/victor-software-house/hakkiri/commit/16981dcecbfe46f30dbc285087a22efd3eb1cb11)).
- A walk skips the files that `.gitignore`, `.git/info/exclude` and the global git excludes ignore, inside a git repository.
  `gitignore: false`, `--no-gitignore`, `--gitignore=false` and `HAKKIRI_GITIGNORE=false` read them.
  A file named on the command line is never skipped for that, and one that no glob selects draws a warning: `on_unmatched`, `--on-unmatched` and `HAKKIRI_ON_UNMATCHED` take `warn`, `error` or `ignore`.
  `fail_level`, `gitignore` and `on_unmatched` are config keys under the flag and the variable, and a package's settings file that holds one is an error ([`ecc757d`](https://github.com/victor-software-house/hakkiri/commit/ecc757de6edd79681b1138e90d2762fde112d772)).
- A scope `title` selects the title of a Markdown link, image and link reference definition, which no other scope reads, as in Vale, where only `raw` does.
  A rule that lists `title`, such as ProseStyle's `BareURL` for hakkiri, sees a URL in `[t](u "see https://x")` ([`037cf08`](https://github.com/victor-software-house/hakkiri/commit/037cf08823bef9c39df8b0b4621d3c79954a5abd)).
- Math in a Markdown or MDX file reads as code, as in Vale.
  An inline span such as `$x$` follows Pandoc's delimiter rules, so `$5 and $10` stays prose, and a block that starts with `$$` ends at a line that ends with `$$`.
  No text rule reports math, and a `code` rule reads it ([`50cda2e`](https://github.com/victor-software-house/hakkiri/commit/50cda2e104cbad3cdec6acff0bc4eb3c0de871fd)).
- A file that a glob selects and that no reader reads is now linted as plain text, where `hakkiri check notes.txt` was a failure and a walk skipped it.
  `formats` maps an extension, a file name or a glob to a reader (`md`, `mdx`, `rs`, `ts`, `tsx`, `js` or `text`), as Vale's `[formats]` does, and a package's settings file may hold it.
  `on_unknown`, `--on-unknown` and `HAKKIRI_ON_UNKNOWN` take `text`, `warn`, `error` or `ignore` for a selected file that no reader reads.
  A binary file is skipped with no message ([`139b064`](https://github.com/victor-software-house/hakkiri/commit/139b064ce24ca865efd2dc06b50b3999124bc74d)).
- A run selects the text of each scope once for every rule that reads it, where it selected it again for each rule.
  The ProseStyle package takes 5.5 s on a Markdown file of 12,000 lines, where it took 25 s, with the same alerts ([`5c5e493`](https://github.com/victor-software-house/hakkiri/commit/5c5e49358b44888ab00d0a9d3a2b7d6c7de505f1)).
- A `hakkiri test` case through the config follows `on_unknown` when no reader reads its `format`, as `hakkiri check` does for a selected file.
  `error` makes the case fail to run, and `warn` and `ignore` give it no alerts ([`e93e92c`](https://github.com/victor-software-house/hakkiri/commit/e93e92c45e0bc79c9d107427586c5726ff4b7d56)).
- `hakkiri test` reads a case whose `format` no extension names, such as `txt`, as plain text, where it refused the case.
  A case that runs through the config reads its input by the config's `formats` entry first, as `hakkiri check` does for a file ([`8f9f7ef`](https://github.com/victor-software-house/hakkiri/commit/8f9f7ef45da6bff338948b5984fbdb99f3749b06)).
- A vocabulary builds two rules, as in Vale: `Vale.Terms` reports an accepted term written in another case, and `Vale.Avoid` reports a rejected term.
  List the style `Vale` in `styles` to run them, or set them by name.
  A vocabulary that a glob names builds `Vale.<Name>.Terms` and `Vale.<Name>.Avoid`, which run on that glob's files.
  A term that is a pattern keeps its escapes: `data\Wset` still means a non-word character between the words, where Vale lowercases it to `\w` ([`d4ce0a5`](https://github.com/victor-software-house/hakkiri/commit/d4ce0a53b3c5296c164f19d1eba913a3ffe9ac81)).

### Changed

- A fault in the config names the line of its own key, also for a glob that holds `.`, `*`, `/`, `[` or `{`, and for a config written in flow style.
  A fault of the first glob that sets a rule used to name the line of a later glob that sets it ([`39fc702`](https://github.com/victor-software-house/hakkiri/commit/39fc7021643396bdf63a70e50cc27509f84b016e)).
- Doc line comments of one type join across blank lines, as rustdoc reads them.
  A fenced block that a blank line splits stays code in every scope.
  Plain line comments and block comments read as before ([`958aeba`](https://github.com/victor-software-house/hakkiri/commit/958aebaea5440dee4ae05c882aa9761157e30fb0)).
- A rule message fills `%[1]s` and `%[2]s`, which take the first and second value, as Vale's messages do.
  A `%s` after one takes the next value, and a verb with no value reads as nothing ([`58947e9`](https://github.com/victor-software-house/hakkiri/commit/58947e9b0a378a685597b53fc1968f531ee996de)).
- A fault about a key itself in the config names the line of that key, also when the value below it spans lines.
  The keys are a glob that is no glob, a setting that names no rule, a key that a package's settings file may not hold, and a `path` entry in such a file.
  Before, the fault named the first line of the value ([`6be7abb`](https://github.com/victor-software-house/hakkiri/commit/6be7abbbc9041debaa22095530daa1dc65b8195a)).
- A fault about one item of a list in the config names the line of that item.
  The items are a `styles` name that is no style, a `vocab` name that is no vocabulary, and a pattern of `block_ignores` or `token_ignores` that does not compile.
  Before, each fault named the first line of the list ([`8d879bb`](https://github.com/victor-software-house/hakkiri/commit/8d879bb4f90c8049d70ea55225fa776a51d2e66b)).
- The JSON alert gains `end_line`, and `end_line` and `end_column` name the last character of the match, also for a match that wraps a line.
  A wrapped match used to end at the end of its first line.
  The pretty line keeps `path:line:column` ([`b6fb20f`](https://github.com/victor-software-house/hakkiri/commit/b6fb20f7cdd741b9e962a666ea1363592b2c4dbb)).

### Fixed

- A `code` rule reads an MDX file's `{…}` expressions and its JSX elements with no children, as in Vale.
  The tags of an element with children and a `{/* … */}` comment still read as no node ([`8d61a7d`](https://github.com/victor-software-house/hakkiri/commit/8d61a7db4a5e149452543e1cc2fbcafb8996ff1c)).
- `hakkiri test` reads a case through the config with the config's block and token ignores, as `hakkiri check` reads a file.
  A level from the config or from a `hakkiri Style.Rule: level` comment leaves a rule failure an error, so a broken rule still fails the run.
  A vocabulary term keeps its group names, backreferences and flags, so `(?<Dup>Ab)\k<Dup>` and `(?U)` build as written.
  A `$` span beside a stray backtick, a code span or another span reads as math, a line indented four columns opens no display block, and a paragraph of unclosed `$` or `<` costs its length ([`668c3e9`](https://github.com/victor-software-house/hakkiri/commit/668c3e9888da4e47b2d36bc33cfbb0abe4032350)).

## hakkiri 0.0.5

### Added

- A glob's empty `vocab` list drops only the lists that earlier configs gave the same glob text, as Vale's `NONE` clears only the section of that name.
  The other globs that match a file still add their lists ([`a855a45`](https://github.com/victor-software-house/hakkiri/commit/a855a45fb2963bcd52b598009a51c54b85c5e48e)).
- A package folder may hold `.ctl/hakkiri.yaml` in the format of the project config.
  hakkiri applies the settings of each package ahead of the project's own, so the project overrides a package and a later package overrides an earlier one.
  A package may list packages, pinned by `url` and `sha256`, to a depth of 10 ([`15c5e81`](https://github.com/victor-software-house/hakkiri/commit/15c5e813715f1148b2e047735ae7bbe68da5875f)).

## hakkiri 0.0.4

### Added

- A project names vocabularies in `vocab`, at the top level and in a glob of `files`, and hakkiri reads each from `config/vocabularies/<name>/` in a package.
  The accepted terms hold back matches in `capitalization`, `conditional`, `existence`, `repetition`, `substitution` and `spelling` rules, as in Vale, and a glob's list does so for the files it matches ([`5023925`](https://github.com/victor-software-house/hakkiri/commit/5023925ee456344843e79f56f9879fa3576c7a72)).

## hakkiri 0.0.3

### Added

- A package pinned by `url` and `sha256` is downloaded on the first run, checked against its hash, and kept in a cache folder, so a second run needs no network.
  A zip is a Vale package: one top folder that holds a `styles` folder, or is a style itself, kept as the zip laid it out.
  `--cache-dir`, or `HAKKIRI_CACHE_DIR`, moves the cache from its default in the per-user cache folder ([`51cdd62`](https://github.com/victor-software-house/hakkiri/commit/51cdd62651718a6377c915eeb7e450174ef87ddd)).
- A Vale style loads as it is: a rule whose `extends` names another rule starts from it, with `key+` and `key-`, as in Vale.
  A rule hakkiri cannot run, a `script` rule or a `doc()` scope, no longer stops its style from loading.
  The rest of the style runs, and the run exits 2 for that rule until the config sets it `off` ([`d4dc1f9`](https://github.com/victor-software-house/hakkiri/commit/d4dc1f9e29411884ddf002741e109dfa7507dacf)).

## hakkiri 0.0.2

### Added

- A key that names a rule and a level under a glob of `files` now runs that rule, though no glob lists its style, as a rule setting does in Vale.
  The word `on` turns a rule on at the level of its rule file, as Vale's `YES` does.
  An empty `styles` list also drops the rule settings before it ([`b3c8136`](https://github.com/victor-software-house/hakkiri/commit/b3c813682f5b9d9de10270df2dc91214882bb51f)).

### Changed

- The `linux-x64` release is a static musl binary, so it starts on a Linux machine with no glibc loader, such as NixOS without nix-ld.
  The musl build links mimalloc, because musl's own allocator made a parallel `check` about eight times slower.
  The asset name does not change, so a `mise` pin of `github:victor-software-house/hakkiri` needs no edit ([`b5ceac7`](https://github.com/victor-software-house/hakkiri/commit/b5ceac7227ac03a51489000c3f90de8b3e499baa)).

## hakkiri 0.0.1

### Added

- First release. `hakkiri check` lints Markdown, MDX and the comments of Rust, TypeScript and JavaScript under `.ctl/hakkiri.yaml`, in parallel, and exits 2, 1 or 0 as Vale does.
  `hakkiri test` runs the cases that rule files and `*.test.yml` files hold, and `--coverage` fails for a rule no case makes flag anything.
  Releases are GitHub Release tarballs for `linux-x64` and `darwin-arm64`, each with its checksum, and mise installs them with the `github:` backend ([`bf3b6d5`](https://github.com/victor-software-house/hakkiri/commit/bf3b6d559f07e06278646d7bbe587b05d7ca418c)).

