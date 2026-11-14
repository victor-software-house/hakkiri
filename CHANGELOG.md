# Changelog

## hakkiri 0.1.0

### Added

- The top-level string values of a Markdown or MDX file's front matter are linted as text, as in Vale.
  YAML between `---` lines, TOML between `+++` lines and JSON between `;;;` lines are read, and a rule scoped `text`, `frontmatter` or `frontmatter.<key>` reads a value.
  A block or token pattern leaves the front matter as written.
  The YAML and TOML are read through ctl-core, so anchors, tags and complex keys keep their values ([`d34ff10`](https://github.com/victor-software-house/hakkiri/commit/d34ff10923f00a980f186dd522757d1676e8f7ab)).
- The text of a block builds in time that grows with its length.
  A paragraph of 12,000 lines with 24,000 code spans and links took 0.4 to 0.8 s for each scope that read it, and takes a few milliseconds.
  The ProseStyle package runs on that file in 1.8 s, where it took 5.8 s, with the same alerts ([`dc82952`](https://github.com/victor-software-house/hakkiri/commit/dc829527f6a888abbb7321e5dd45feffcf1edc08)).
- A Markdown file is read as Vale's converter reads it in more places.
  A table cell past the header's last cell, a footnote reference and a definition that nothing references are no text.
  A definition that a reference names is a `list` node.
  A backslash escape and a hard break written with a backslash lose the backslash, so a rule sees `well-known` and not `well\-known`.
  An `ftp://` address is no text, and a bare email address or `www.` address is text.
  The `link` scope holds no word for an autolink, a bare URL or a link whose text is its destination.
  An inline `b`, `strong`, `i`, `em`, `code`, `tt` or `a` element has the scope of its tag ([`df3ff06`](https://github.com/victor-software-house/hakkiri/commit/df3ff06759af242598039d9adadf1703f018aac6)).
- A glob of `files` takes `params`, a mapping from a rule name to the fields of that rule it sets, such as `Vale.Repetition: {max: 1}`, as Vale's `Style.Rule[param] = value` does.
  A parameter reaches the files of the glob that sets it, where Vale applies it to every file, and a later glob wins for the same parameter.
  A parameter that cannot be set is an error that names the config line.
  That covers a level, `tokens` and the other fields that hold lists or prose, and a field the rule's check does not define.
  It also covers a value of the wrong kind, a rule that does not exist, and any parameter of `Vale.Terms` or `Vale.Avoid` ([`52bf327`](https://github.com/victor-software-house/hakkiri/commit/52bf327de6365a39b262607333227c3fcc5a2ecd)).
- A `sequence` rule takes `forms` in place of `tokens`: a list of mappings that each hold `tokens`.
  The rule matches where any form does, and every alert carries the one rule name, so a check with several shapes, such as a noun and a verb or a pronoun and a verb, is one rule.
  Two forms that match overlapping words give the match that starts first.
  A rule that sets both `tokens` and `forms`, or neither, is an error, and a fault in a form names it, as `forms[1].tokens[0].pattern`.
  A config that sets `forms` as a parameter is refused, as it is for `tokens`.
  A rule with one `tokens` list gives the alerts it gave before.
  Vale stops with a panic on a rule with `forms` and no `tokens` ([`474b591`](https://github.com/victor-software-house/hakkiri/commit/474b591282e9fb75b53e5f7ffbc5ce386198c2a5)).
- `title & link` and `title & image` select the title of a link and of an image, and a new scope `image` selects the alt text of an image.
  The `title` attribute of an HTML tag, in a block or in a line of text, and of a JSX element is a title too, and `title & link` and `title & image` select that of an `a` and an `img` tag.
  The title of a tag in the text of a link belongs to that tag, so `title & link` leaves it out.
  `title & ~link` and `title & ~image` leave out the titles of links and of images ([`8601916`](https://github.com/victor-software-house/hakkiri/commit/860191653bea180708a721087553c7a92bd4e3c7)).
- The built-in style `Vale` always exists and holds the rule `Vale.Repetition`, with Vale's defaults, so `styles: [Vale]` is no longer refused when no vocabulary holds a term, and it reports a word that repeats, such as `the the`.
  Accepted terms hold a repeat back, and a level, `on` and `off` set by the name apply to it.
  Two words that differ in case compare character by character, as in Vale, so `ΣΑΣ` and `σασ` are a repeat ([`b6f6a39`](https://github.com/victor-software-house/hakkiri/commit/b6f6a398525201b452839614ced4903c6ad64948)).
- The built-in style `Vale` holds `Vale.Spelling`, which reports a word the built-in dictionary lacks, at the level `error`, as Vale does.
  Each package's `config/dictionaries` folder adds its dictionaries, and every `.txt` file under its `config/ignore` folder adds the words it lists.
  A link under `config/ignore` that reaches outside the package is refused, as every other path of a package is.
  A project that lists `Vale` in `styles` now gets spelling alerts, and turns them off with `Vale.Spelling: off`.
  The rule is built only when a glob lists `Vale`, names a rule that starts with `Vale.`, or sets `params` for one, so a package with a dictionary that cannot load fails no run that does not use it.
  A config sets a field with `params`, such as `Vale.Spelling: {split: true}`.
  It differs from Vale where several packages each hold settings, in a code span of a code comment, and in the name after `@param`.
  A Markdown file now keeps a bare email address, a `www.` address and an angle-bracket email address in its text, as Vale does, where it dropped them as links ([`c262948`](https://github.com/victor-software-house/hakkiri/commit/c262948863eb89a9290e768ece167af0107bf7a1)).
- A vocabulary folder may hold `vocab.yml` and `vocab.yaml` besides `accept.txt` and `reject.txt`, as in Vale.
  A YAML entry is a term, a term mapped to nothing, or a term mapped to the options `regex`, `ignorecase` and `flags`, with a plain term read as literal text.
  An `affix` key names an affix file in the `config/dictionaries` folder of the package that holds the vocabulary, then of every other package, and never in `$DICPATH`.
  A key the file does not define is an error, and every wrong entry is reported with its file and line ([`45795a4`](https://github.com/victor-software-house/hakkiri/commit/45795a47e4ab8fe3fa058e522fe3e0be23ced47b)).

### Changed

- A code span is a noun in a `sequence` rule: the tagger reads a noun in its place, its tag is `NN`, and a pattern reads it as the letter `X`.
  A rule of a noun and a verb, such as `ThingsSpeakNoun`, flags `` `flag` says `` as it flags `the flag says`.
  A rule for four lowercase nouns, such as `NounStack`, counts no code span.
  The square brackets of an intra-doc link, as in ``[`Name`]``, belong to the span ([`f38ad77`](https://github.com/victor-software-house/hakkiri/commit/f38ad772adabd4b8858e3ed70c1c09c57ecb300b)).

  The tagger corrects the verbs of saying and asking after a noun.
  `the docs say` reads as a verb, `for the asks that wait` reads `asks` as a noun, and `the hook told of the file` reads `told` as a participle.
  A rule with a verb tag on its verb token then flags a thing that speaks and no request or participle.

  An `existence` rule reads a code span as one `*`, as Vale masks it, and reports a match that runs across a span.
  It drops a match that starts or ends on one.
  The package's `ClauseChain` gives about 500 alerts more on nine repositories, and Vale gives them too.

  The tagger reads `check`, `prompt`, `thinking` and `fade` as nouns after a determiner.
  It reads `prompt`, `round` and `split` as nouns before `pull request` in a title, as in `the host load split pull request`.
  It reads `check` as a noun that ends a compound of singular nouns.
  So `NounStack` counts them in a run of nouns.
- Front matter is read through the `front-matter` crate, with one parser per format: YAML, TOML and JSON.
  hakkiri reads each form that the crate finds, as Vale does where Vale reads it.
  YAML sits between `---` lines, after `---yaml` or closed by `...`.
  TOML sits between `+++` lines or after `---toml`.
  JSON sits between `;;;` lines, after `---json`, or as an object that opens with `{` on the first line.
  A YAML key that is a list, a mapping or an alias gives no value and drops none of the entries after it ([`c90d7b1`](https://github.com/victor-software-house/hakkiri/commit/c90d7b1d5af82213fea4f6e819e1dce7f2260724)).

  Every block is cut from the body, so no line of it is a paragraph.
  Vale lints the lines of a `;;;`, `---toml` or bare `{` block and of YAML closed by `...` as a paragraph too, so a key such as `Install` alerts there.
  hakkiri lints only the values of the block.
- A Markdown or MDX file whose front matter no parser reads now fails with the line of the problem and exit 2, and the rest of the file is still linted.
  A key written twice, a merge key, a key that is a number or a null, and the values `.inf`, `-.inf` and `.NaN` no longer hide the other values of the block, as in Vale.
  ctl-core moves to 0.7.4 ([`92a47fc`](https://github.com/victor-software-house/hakkiri/commit/92a47fca102a4fd9e0f27c409fde55b0912c39e2)).
- Each file, fenced code block and comment has a language from langdex, the GitHub Linguist table.
  A file takes the language of its path, or of the interpreter on its first line, and is read by the reader of that language, so `.mts`, `.cts`, `.markdown` and a script that starts `#!/usr/bin/env node` get their reader.
  The last piece of a scope part names a language by any of its names, so `text.comment.js` reaches `.jsx`, `.mjs` and `.cjs` files, and `text.txt` no longer reaches a `.rst` file or a `README`.
  A fenced code block of a language whose comments hakkiri reads is parsed with that grammar, and its comments are linted.
  `formats` takes any name of a language hakkiri reads, and `READERS` is gone ([`7e909a7`](https://github.com/victor-software-house/hakkiri/commit/7e909a7ef821882c0b70af66a1dd88754fd760bd)).
- The globs that match a file now rank by specificity, not by the order written.
  A more specific glob wins for the same key wherever it is written ([`9422967`](https://github.com/victor-software-house/hakkiri/commit/94229675a135c10b8c11c19bce12ded74586ac25)).

  At equal specificity the project beats a package.
  The top level of a config file ranks below every glob, so a package glob beats the project's top-level `code_span_size`.

  The project can force a value with the YAML tag `!override`.
  A forced value applies after every value without the tag.
  A package's settings file that holds the tag is an error, and so is any other tag.

  A glob with `{…}` groups ranks, for each file, as the most specific of its alternatives that matches that file.
  So a skip list such as `{CHANGELOG.md,notes/**}` beats a package's `*.{md,mdx,rs}` for the files it names.
  A glob with more than 256 alternatives is an error.
- A `files` or `formats` glob matches by the rules of gitignore: `*` stays inside one folder, `**` spans folders, and a glob with no `/` matches the file name in any folder.
  So `docs/*.md` no longer matches `docs/x/a.md`: write `docs/**/*.md` for every file below `docs`.
  A leading `/` ties a glob to the project root, and a trailing `/` matches every file the folder holds ([`63265bb`](https://github.com/victor-software-house/hakkiri/commit/63265bbe6215c9285ba873215ab0780f29a2ab9b)).
- The tagger corrects the model's tags where it is wrong on our prose.
  The cases are a possessive `'s` after a noun, a modal written in capitals, a name in capitals and a semicolon.
  A verb after a noun phrase, which the model reads as a plural noun, is the fifth.
  On UD English GUM the tags match the gold tags 92.53% of the time, up from 92.16%.
  `NounStack` and `ThingsSpeakNoun` give 68 fewer alerts on the nine repositories of the alert diff, all false.
  A title is still tagged as the model tags it ([`e704f69`](https://github.com/victor-software-house/hakkiri/commit/e704f6916bc9723636aeb39e7cac97f11b087021)).

### Fixed

- Add `code_span_size`, the number of words a code span counts as in a word count.
  It is 1 by default, and 0 counts as Vale does.
  It reaches the `words`, `sentences` and `syllables` of a `metric` or `readability` rule, and the scores built on them.
  A package's settings file, the project config and a glob set it, and `--code-span-size` and `HAKKIRI_CODE_SPAN_SIZE` win over them.
  `engine::lint_accepting` and `summary::summarize` take it as a last argument ([`ef5449f`](https://github.com/victor-software-house/hakkiri/commit/ef5449f2f273d2b0924aaa942cb1b806acf41289)).
- A comment that turns rules off no longer reaches text that Vale reads before the comment.
  Vale reads the text of a tight list item before a nested list, quote, code block or table.
  It reads the text of an HTML block before a tag that opens a block, and the text of a paragraph before a `div` or `p` tag inside it.
  A comment after that text does not hide its alerts.
  A comment in a block of its own, after the last text of an HTML block, reaches that text when no tag flushes it first.
  A block tag inside an inline tag, such as a `div` inside a `span`, is read as inline, so it flushes no text.
  A `pre`, `script`, `style`, `noscript` or `iframe` tag, or a block tag with a skipped class, flushes the text at its end tag, not at its start tag ([`2f09179`](https://github.com/victor-software-house/hakkiri/commit/2f09179ea2d137856b6a7551fd5255dcaee31e0d)).
- hakkiri reads the comments of Go, Swift, YAML, TOML and shell files, through new `go`, `swift`, `yaml`, `toml` and `sh` readers that `formats` takes.
  It leaves out the comments that tools read, in every language.
  It draws no alert in the link or name of a Go, Rust or JSDoc doc comment.
  A line comment of a Go, YAML, TOML or shell file keeps its indentation after the first space.
  A file with one of the new extensions that a glob selects gives its comments only, where it was plain text ([`e16b451`](https://github.com/victor-software-house/hakkiri/commit/e16b45109e7232d56edc338b76aebc3e6e7a4019)).
- A file that `formats` maps has its own extension and its reader's for a scope such as `text.md`, so `text.md` and `text.txt` both select text in a `.txt` file mapped to `md` ([`1aa8750`](https://github.com/victor-software-house/hakkiri/commit/1aa87509430754f9d3fac79e861fe42a804cdaa0)).
- The alt text of an `img` tag in a line of Markdown reads as an `alt` block, as Vale reads it.
  A rule on `text` or `alt` now checks it, and the paragraph around it reads as before ([`5166636`](https://github.com/victor-software-house/hakkiri/commit/51666368b8c953c9c1424fbb2ce135f789907e4d)).
- A file that is not valid UTF-8, such as a Latin-1 one, is linted with each bad byte read as one `?`, so a column still counts bytes, as in Vale ([`b0a74e8`](https://github.com/victor-software-house/hakkiri/commit/b0a74e8fffe328aac4020d14c9c6b278a8f1d49b)).
- Count no word for a Markdown link whose text is its address, as Vale counts none.
  The text is compared with the address as goldmark writes it, percent-encoded, so `[ä.md](ä.md)` and `[a b](<a b>)` keep their words, as in Vale ([`ab5a103`](https://github.com/victor-software-house/hakkiri/commit/ab5a103771660575d22e7ad77346275e64429eb6)).
- A line with no `>` in a quote holds none of the containers inside the quote, so a `$$` line indented six spaces after `> - Item` opens no display block ([`f6a076a`](https://github.com/victor-software-house/hakkiri/commit/f6a076a26b36cb9611d59073adf75ab0017c4bfc)).
- The sentence splitter reads code spans: a code span begins a sentence after a full stop and ends one before a full stop.
  A unit of measure after a number, as in `92 ms. Tests`, ends a sentence before a capital word.
  `vs.` ends no sentence, and a date is no list number ([`38f8260`](https://github.com/victor-software-house/hakkiri/commit/38f8260015434db386f9b09cb46cc9e76741914b)).
- A case that runs through the config has the extension that `hakkiri check` gives the file, the reader's when `formats` picks it.
  So `text.md` selects in a `format: cfg` case that `cfg: md` maps ([`f42f3fc`](https://github.com/victor-software-house/hakkiri/commit/f42f3fc589129ac10afda71740a3d1e9c3983bf1)).
- A package that holds a style folder named `Vale` loads again: hakkiri skips that folder, as Vale does, and prints one warning that the built-in style `Vale` wins.
  The other styles of the package lint as before ([`a079dc9`](https://github.com/victor-software-house/hakkiri/commit/a079dc953c66d4b798ab1cf856d0b70ab5d51b73)).
- A YAML vocabulary term with `ignorecase: true` is quoted whole, so the period of `Node.js` matches a period and not any character.
  Vale reads that period as a wildcard, and the `vale-differences` spec records the difference ([`ef7ab12`](https://github.com/victor-software-house/hakkiri/commit/ef7ab12cefc395396738f55e5cd6170546b775b8)).

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

