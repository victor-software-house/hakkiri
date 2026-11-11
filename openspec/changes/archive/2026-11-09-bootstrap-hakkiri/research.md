# Research: a prose linter to replace Vale

Retrieved 2026-11-09.  
Each claim links its source.  
"Read at source" means the file, page or API response was read this session; "not verified" marks the rest.

## 1. Findings

1. No existing tool replaces Vale for [ProseStyle][ps].  
   Every candidate either runs a fixed rule set, checks grammar only, or writes rules in another language (section 4).
2. ProseStyle uses a small part of Vale.  
   The package configuration turns on 42 rules of four types: existence, substitution, occurrence and metric (section 2.1).  
   The Tengo script rule and the conditional rule ship in the zip but no section turns them on.
3. The slowest and least stable rules exist because Vale gives them no scope for "prose outside code".  
   `BareURL` rebuilds fenced blocks and code spans out of lookbehinds; that rule caused the backtracking panics and the 249 s run (section 3).  
   A linter that walks a Markdown tree offers that scope, and the rule becomes a token match.
4. The Rust building blocks are mature for every part except sentence splitting and syllable counts (section 5).  
   `fancy-regex` now accepts variable-length lookbehind and returns an error instead of a panic when it runs out of backtracking budget.
5. Whether `fancy-regex` compiles the six ProseStyle patterns that use lookbehind, unchanged, is not verified.  
   That one test decides how cheap reading Vale rule YAML as it is would be.

## 2. What ProseStyle needs

Read at source in [prose-style at be9ef17][ps-head] and the upstream styles its build packs.

### 2.1 Rules turned on

The package [`.vale.ini`][ps-ini] sets `BasedOnStyles = ProseStyle, Voices, Direct` for `*.{md,mdx,rs}`, then names single rules from the other styles.

| Style | Turned on | Off | Types in use |
|:--|:--|:--|:--|
| ProseStyle | 25 | 0 | existence, substitution, occurrence (`LongSentence`), metric (`Paragraph`, `ListItem`: word counts) |
| Voices | 9 | `ColonReveal` | existence, substitution |
| Direct | 3 | `Ease` | existence; `Length` extends `Std.Readability.SentenceLength` |
| Std | 4, as suggestions | 10 | existence, substitution |
| Readability | `FleschKincaid`, as a suggestion | 7, including the Tengo rule `Polysyllables` | metric with a formula |

Projects set `MinAlertLevel = warning` in the [template's `.vale.ini`][tpl-ini], so the five suggestion rules run in every project and never print there.  
The open [frequency-habit-rules][ps-freq] change proposes rules that count a habit per sentence or per word, which no Vale rule type expresses today.

### 2.2 Features in use

- Scopes: `raw`, `raw.rs`, `heading`, `list`, `paragraph`, `sentence`, `table.header`.
- Rule fields: `tokens`, `swap`, `exceptions`, `ignorecase`, `nonword`, `raw`, `max`, `formula`, `condition`, `extends` naming another style's rule, a `%s` or `%d` message argument, `level`.
- Regex: 43 of the enabled-style files are existence or substitution lists.  
  Six ProseStyle rules use lookbehind: `BareURL`, `BareURLInComment`, `BareWorkItem`, `BareWorkItemInComment`, `Dashes`, `Exclamation`.  
  `BareURL`'s lookbehind is variable-length and nests lookarounds ([BareURL.yml][ps-bareurl]).
- A vocabulary, `Vocab = ProseStyle`, whose `accept.txt` lists words that pass upstream rules.
- Formats: Markdown, MDX, and Rust through `[formats] rs = md`.
- Configuration: a package `.vale.ini` read before the project's, glob sections, per-rule levels and `NO`, `StylesPath`, `Packages` pointing at a release zip.
- Tests: every rule carries `tests:` cases run by `vale test --coverage`, which fails when a rule fires in no case, plus 16 package cases in [`test/package.test.yml`][ps-pkgtest].  
  `vale test` is upstream since [ae6d749][vale-test] (2026-08-11).

### 2.3 Use in projects

The Copier template's `lint:prose` task lists tracked files with `git ls-files`, passes them to `vale --output=line` through `xargs`, and fails on any printed line when `prose_fail_level` is `warning`.  
It pins Vale 3.26.0 through mise and ProseStyle v0.0.7 in `includes/pins.jinja`.  
Generated paths get an empty `BasedOnStyles` section, so large generated files are skipped rather than linted.

## 3. Vale today

Read at source in [vale-cli/vale at 66bbb85][vale-head] unless marked.

| Fact | Source |
|:--|:--|
| MIT, Go, v3.26.0 on 2026-10-31; 14 releases in 2026; 388 commits in 2026, nearly all by one maintainer | GitHub API |
| `Lint()` loops over its file arguments one at a time; the 5-worker pool runs only inside a directory walk | [lint.go L210-L290][vale-lint] |
| Markdown goes through goldmark with GFM and footnotes, is rendered to HTML, and scopes come from the HTML | [md.go][vale-md] |
| A fragment format such as Rust is linted as Markdown, then the whole file is restored for `raw` and summary scopes, so `raw` reads string literals | [fragment.go L104-L108][vale-frag] |
| The Rust query reads `(line_comment)+` only; block comments are skipped | [rs.go][vale-rs] |
| Regex runs on a fork of regexp2 with a 100,000-slot backtracking stack, and Vale's wrappers panic when a match exceeds it | [regex.go][vale-regex], [regexp2 options.go][regexp2-opts] |
| The only issue on parallel files, [issue 21][vale-21], was closed in 2017 | GitHub issue search |

The README's "files linted in parallel" holds for a directory argument, not for a list of files.

On one large Rust project's prose files, ProseStyle v0.0.6 took 249.5 s in one Vale process on 8 vCPUs.  
Version 0.0.7 took 9.0 to 9.4 s on the same machine (measured 2026-11-09, not repeated here).

## 4. The field

Licences and dates from the GitHub API unless a LICENSE link is given.

| Tool | Licence, language | State | Rule model | Fit |
|:--|:--|:--|:--|:--|
| [Vale][vale] | MIT, Go | active, one maintainer | YAML rule types plus Tengo scripts | runs the package today |
| [vale-ls][vale-ls] | MIT, Rust | v0.6.0, 2026-10-01 | language server that calls Vale | editor front end for Vale only |
| [Harper][harper] | Apache-2.0, Rust | v3.1.0, 2026-11-06 | built-in linters and the [Weir][weir] phrase DSL; no regex, scopes or metrics | grammar; style is not its goal ([discussion 1465][harper-1465]) |
| [vale-cli/Harper][vale-harper] | Apache-2.0 | v0.1.0, 2026-08-05 | Harper rules compiled to Vale YAML | a Vale style instead of a linter |
| [textlint][textlint] | [MIT][textlint-lic], TypeScript | v15.8.0, 2026-08-01 | JavaScript functions over an AST; MDX plugin | rules are code and need Node |
| [proselint][proselint] | BSD-3-Clause, Python | v0.16.0, 2025-11-14 | fixed checks | no user rules |
| [write-good][write-good], [alex][alex] | MIT, JavaScript | last commits 2025 and 2024 | fixed word lists | no user rules |
| [LanguageTool][lt] | LGPL-2.1+, Java | active | XML pattern rules, server | grammar engine, plain text only, heavy |
| [Language Check][langcheck] | MIT, Rust | started 2026-03 | runs Harper, LanguageTool, Vale and proselint | delegates style to Vale |
| [Stilyagi][stilyagi] | ISC, Rust and Python | pre-0.1.0, no rules | Python rules over extracted regions | early; extraction overlaps ours |

One aggregator page scores Vale 87 and textlint 81 ([faun.dev][faun], seen only as a search excerpt, no method shown).  
Other Rust tools found (`prosesmasher`, `lucid-lint`, `dslop`, `mq-content-lint`) ship fixed checks; none offers user rules with scopes and a vocabulary.

## 5. Rust building blocks

Versions and licences from the crates.io API on 2026-11-09.

### 5.1 Parsing

| Crate | Licence | Latest | Notes |
|:--|:--|:--|:--|
| [markdown][markdown-rs] (markdown-rs) | MIT | 1.0.0, 2025-04-23; no push since | mdast with a position on each node; GFM and MDX |
| [comrak][comrak] | BSD-2-Clause | 0.59.0, 2026-10-26 | CommonMark and GFM spec tests pass; source positions per node (not read this session); no MDX |
| [pulldown-cmark][pulldown] | MIT | 0.13.6, 2026-10-30 | event stream with byte ranges; GFM tables; no MDX |
| [tree-sitter][ts] and [tree-sitter-rust][ts-rust] | MIT | 0.27.2 and 0.24.2 | `line_comment`, `block_comment`, `doc_comment` nodes ([node-types.json][ts-rust-nodes]) |
| [ra_ap_rustc_lexer][ra-lexer] | MIT or Apache-2.0 | 0.184.0, 2026-11-08 | comment tokens without a grammar or C build |

### 5.2 Matching

| Crate | Licence | Latest | Notes |
|:--|:--|:--|:--|
| [regex][regex] | MIT or Apache-2.0 | 1.13.1 | linear time; no lookaround |
| [fancy-regex][fancy] | MIT | 0.20.0, 2026-10-24 | variable-length lookbehind since 0.18 ([CHANGELOG][fancy-cl]); `backtrack_limit` returns [an error][fancy-err] instead of a panic |
| [aho-corasick][aho] | Unlicense or MIT | 1.1.5 | word lists |
| [rayon][rayon], [ignore][ignore] | MIT or Apache-2.0; Unlicense or MIT | 1.12.0; 0.4.34 | parallel files; gitignore-aware walk |

### 5.3 Text

| Need | Candidates | State |
|:--|:--|:--|
| Sentences | [unicode-segmentation][unicode-seg] (Unicode UAX 29) | active; abbreviations need our own list |
| Syllables for Flesch-Kincaid | `textstat`, `readsight`, `syllarust`, `hypher` | low use or unproven; write our own and test it against Vale's numbers |
| Spelling | [spellbook][spellbook] | MPL-2.0, Hunspell dictionaries, alpha; only needed if a spelling rule ships |
| Script rules | [rhai][rhai] | pure Rust, operation limits; only needed if the Tengo rule stays |

### 5.4 Around the CLI

| Need | Crate | Notes |
|:--|:--|:--|
| LSP | [lsp-server][lsp-server] or [tower-lsp-server][tlsp] | `tower-lsp` has had no release since 2023 |
| YAML | [serde-saphyr][serde-saphyr] and [granit-parser][granit] | already used by ctl-core and yamled; `serde_yaml` is deprecated |
| CLI chassis | [ctl-core][ctl-core] | clap, shared flags, pretty and JSON output, `.ctl/<verb>.yaml` config home |

## 6. What the design has to settle

These follow from the findings; [design.md][design] records how the operator settled each one.

1. Rule format: read Vale `.vale.ini` and rule YAML as they are, or a ctl-style `.ctl/<verb>.yaml` plus a new rule format with a one-time converter.  
   Reading Vale YAML as it is ties us to Vale's scope names and regexp2 syntax.  
   A new format can add the scopes in finding 3.
2. First release: Markdown, MDX and Rust comments; existence, substitution, occurrence, metric and `extends`.  
   Tengo, conditional, spelling and the five suggestion rules are not needed to reproduce what projects see today.
3. Where it runs: the template's `lint:prose` task first; CI follows from it; an editor server and a pre-commit hook later.
4. How rules ship: a release zip as today, rules built into the binary, or both.
5. Whether to draft a Vale issue for parallel file arguments; it would be drafted only, not sent.

## 7. Not verified

- Whether `fancy-regex` compiles and matches the six lookbehind patterns as regexp2 does.
- comrak's per-node source positions, and markdown-rs positions on table cells and list items.
- Accuracy of `unicode-segmentation` sentence breaks next to Vale's.
- Rhai sandbox limits.
- textlint editor support and parallelism.

[design]: design.md
[ps]: https://github.com/victor-software-house/prose-style
[ps-head]: https://github.com/victor-software-house/prose-style/tree/be9ef1746c7b23a10a9388ccce1dc6a4bbdff7e3
[ps-ini]: https://github.com/victor-software-house/prose-style/blob/be9ef1746c7b23a10a9388ccce1dc6a4bbdff7e3/ProseStyle/.vale.ini
[ps-bareurl]: https://github.com/victor-software-house/prose-style/blob/be9ef1746c7b23a10a9388ccce1dc6a4bbdff7e3/ProseStyle/styles/ProseStyle/BareURL.yml
[ps-pkgtest]: https://github.com/victor-software-house/prose-style/blob/be9ef1746c7b23a10a9388ccce1dc6a4bbdff7e3/test/package.test.yml
[ps-freq]: https://github.com/victor-software-house/prose-style/tree/be9ef1746c7b23a10a9388ccce1dc6a4bbdff7e3/openspec/changes/frequency-habit-rules
[tpl-ini]: https://github.com/victor-software-house/templates
[vale]: https://github.com/vale-cli/vale
[vale-head]: https://github.com/vale-cli/vale/tree/66bbb85a1a78a5b5497331e04d4033357caced4d
[vale-lint]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/lint.go#L210-L290
[vale-md]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/md.go
[vale-frag]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/fragment.go#L104-L108
[vale-rs]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/code/rs.go
[vale-regex]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/regex/regex.go#L17-L20
[vale-test]: https://github.com/vale-cli/vale/commit/ae6d749
[vale-21]: https://github.com/vale-cli/vale/issues/21
[regexp2-opts]: https://github.com/jdkato/regexp2/blob/af31cb0c57f76300c511ddf59009a2b27c2ade4b/options.go#L8
[vale-ls]: https://github.com/errata-ai/vale-ls
[harper]: https://github.com/Automattic/harper
[harper-1465]: https://github.com/Automattic/harper/discussions/1465
[weir]: https://writewithharper.com/docs/weir
[vale-harper]: https://github.com/vale-cli/Harper
[textlint]: https://github.com/textlint/textlint
[textlint-lic]: https://github.com/textlint/textlint/blob/4c916a878ee7ec60622ec795f5654e75f1cb4357/LICENSE
[proselint]: https://github.com/amperser/proselint
[write-good]: https://github.com/btford/write-good
[alex]: https://github.com/get-alex/alex
[lt]: https://github.com/languagetool-org/languagetool
[langcheck]: https://github.com/KaiErikNiermann/LangCheck
[stilyagi]: https://github.com/leynos/stilyagi
[faun]: https://faun.dev/toolbox/compare-prose-linters/
[markdown-rs]: https://github.com/wooorm/markdown-rs
[comrak]: https://github.com/kivikakk/comrak
[pulldown]: https://github.com/pulldown-cmark/pulldown-cmark
[ts]: https://github.com/tree-sitter/tree-sitter
[ts-rust]: https://github.com/tree-sitter/tree-sitter-rust
[ts-rust-nodes]: https://github.com/tree-sitter/tree-sitter-rust/blob/77a3747266f4d621d0757825e6b11edcbf991ca5/src/node-types.json
[ra-lexer]: https://crates.io/crates/ra_ap_rustc_lexer
[regex]: https://github.com/rust-lang/regex
[fancy]: https://github.com/fancy-regex/fancy-regex
[fancy-cl]: https://github.com/fancy-regex/fancy-regex/blob/a460c32089d11493e205ae085723bc80214e61d3/CHANGELOG.md
[fancy-err]: https://docs.rs/fancy-regex/latest/fancy_regex/enum.RuntimeError.html
[aho]: https://github.com/BurntSushi/aho-corasick
[rayon]: https://github.com/rayon-rs/rayon
[ignore]: https://crates.io/crates/ignore
[unicode-seg]: https://github.com/unicode-rs/unicode-segmentation
[spellbook]: https://github.com/helix-editor/spellbook
[rhai]: https://github.com/rhaiscript/rhai
[lsp-server]: https://crates.io/crates/lsp-server
[tlsp]: https://github.com/tower-lsp-community/tower-lsp-server
[serde-saphyr]: https://crates.io/crates/serde-saphyr
[granit]: https://crates.io/crates/granit-parser
[ctl-core]: https://github.com/victor-software-house/ctl-core
