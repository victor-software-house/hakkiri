# Design

## Context

The [`rule-engine` change][engine-design] selects sentences and nodes, builds check text and places alerts.  
The [`pattern-and-readability-checks` change][pattern-design] adds four more check types on top of it.  
The three types here need data that neither change holds:

- `sequence` needs words and Penn Treebank tags for each sentence ([sequence.go][v-seq-run]).
- `capitalization` needs title and sentence case converters, and AP title case reads one tag ([title.go][p-title]).
- `spelling` and `affix` need a Hunspell dictionary and its affix rules ([spelling.go][v-spell-run], [affix.go][v-affix]).

Vale gets the tagger and the converters from prose.  
Its dictionary reader is its own `spell` package, MIT and derived from gospell ([LICENSE][v-spell-license]).

Each Vale behaviour below was read at `66bbb85`, which is 3.26.0 plus 16 commits.  
Two of those commits touch `sequence` and `repetition`, and only to choose the language a file is read in.

## Goals / Non-Goals

**Goals:**

- Give `capitalization`, `sequence`, `spelling` and `affix` Vale 3.26.0's meaning, so the importer, `HKR-007`, converts upstream styles without changing what they report.
- Read every dictionary, word list and affix file from the pinned package that holds the rule.

**Non-Goals:**

- Spelling suggestions and fixes: `action` rides along, as in the rule-engine change.
- Vale's built-in `Vale.Spelling` rule and its `config/ignore` default; a style names its own lists.
- Languages other than English.

## Decisions

### 1. Case conversion ports prose's converters

The sentence and title converters and their word lists are ported from prose's `strcase` package, under MIT ([title.go][p-title], [sentence.go][p-sentence]).  
Vale's agreement count is ported from its `title` and `sentence` functions ([variables.go][v-case]).

In sentence case, a text fails when its first word fails.  
Otherwise it passes when enough words agree: equal to the converted form, in capitals, or matching an exception.  
The share that agrees must be at least `threshold`.

Alternative: `titlecase` 3.6.0, which applies John Gruber's rules.  
Lost: it keeps a different list of small words, so headings that pass in Vale would fail.

### 2. AP title case keeps Vale's tag lookup

In AP style, `to` is lower case only when the tag after it starts with `NN` ([title.go][p-title]).  
So `to` before an article starts with a capital.

Vale reads that tag by the position of the word among the converter's words.  
The tags come from the tagger's own words, and a contraction or hyphen before `to` shifts the two lists.  
hakkiri reads the tag the same way.

Alternative: find the tag by the word's byte offset.  
Lost: it gives a different answer from Vale in exactly the headings where the lists shift.

### 3. sequence walks tokens around an anchor

Each token compiles as Vale compiles it ([sequence.go][v-seq-new], [upos.go][v-upos]):

- `pattern` is anchored to one word, with `(?i)` under `ignorecase`.
- `tag` is a pattern matched anywhere in the tag.
- `upos` becomes the Penn tags Vale maps it to, plus Vale's word list where Penn cannot express the tag.
- `skip` adds optional tokens before the one it pads, and `min` repeats the token with its window ([sequence.go][v-seq-tokens]).

The search starts from the first token with a pattern, or else the first with a tag.  
It walks left and right from each place that token matches ([sequence.go][v-seq-match]).  
The alert spans the matched words, or the words from the first to the last `target` token.

Alternative: compile the tokens into one pattern over a string of `word/TAG` pairs.  
Lost: `negate` at a sentence edge and `target` spans have no direct form there, and results would drift from Vale's walk.

### 4. model is refused

`model` names another checker's tagger model in Vale's dictionary folder ([sequence.go][v-seq-new]).  
hakkiri refuses a rule that sets it.

Alternative: read Vale's model format.  
Lost: it ties hakkiri to a Vale file format, for a field no known style uses.

### 5. spelling reads words as Vale's tokenizer cuts them

Spelling words come from Vale's word tokenizer, which splits on spaces and trims listed prefixes and suffixes ([spelling.go][v-spell-run]).  
The metric check's word pattern is not used here.  
Each word is converted by the dictionary's `ICONV` table, and skipped by the built-in filters, unless `custom: true`, and by `filters`.  
The remaining words are looked up.

`split` takes an identifier apart and reports each misspelt part of three letters or more at its own position.  
Vale's two message rewrites stay, for a casing mistake and for an identifier in Markdown or MDX ([spelling.go][v-spell-explain]).  
A word that overlaps a code span gives no alert, as the rule-engine change decides for `existence`.

Alternative: one word splitter for every check.  
Lost: Vale's spelling and metric checks cut words differently, and either choice changes one of them.

### 6. Dictionaries and lists stay inside the package

`dictionaries`, `dicpath`, `aff`, `dic`, `ignore` and `affix` resolve inside the package that holds the rule.  
`affix: en_US-web`, and a rule with no dictionary, mean the built-in files.  
A path that leaves the package is refused.

Alternative: Vale's search, which also tries an absolute path, the working directory and `DICPATH` ([spelling.go][v-spell-dicpath], [vocab.go][v-find-affix]).  
Lost: the [project config][project-config] pins each package by hash, and a dictionary found on the machine would make two runs of one pin differ.

### 7. Fields Vale reads and never uses

A `spelling` rule's own `exceptions` and its `threshold` load with no effect.  
Vale builds its exception pattern before it decodes the rule's fields, and reads `threshold` nowhere ([spelling.go][v-spell-new]).

Alternative: apply `exceptions` as a skip list.  
Lost: imported rules would report fewer words than in Vale, a difference the [format design][format-7] does not list.

### 8. affix expands before compiling

A `word/FLAGS` part of an `existence` token or a `substitution` key becomes an alternation before the rule compiles ([affix.go][v-affix]).  
The alternation holds the word and the forms its flags name.  
The expansion uses the dictionary reader's affix rules, so `affix` lands with that reader, and the rule-engine change's refusal ends.

### 9. Tags come from a port of prose's tagger and model

hakkiri ports prose's averaged perceptron and its English tokenizer, `tokenize.New()`, and embeds the English model (operator, 2026-11-10).  
Vale tags each sentence with that tokenizer and the perceptron, so the port is those two and not prose's Treebank tokenizer ([tag step][v-tag-text]).  
The model is trained on UD English EWT under CC BY-SA 4.0, and weighs 1.9 MB ([MODELS.md][p-models]).  
It ships as its own data file with its notice.

The port keeps prose's case and category tests where Go and Rust agree.  
The shape of a word reads Rust's Unicode properties and not Go's categories, so a word of characters such as `²` can get another shape, and so another feature.  
A test compares tokens and tags with prose's on 188 sentences.

Creative Commons lists GPLv3 as compatible with BY-SA 4.0 ([compatible licenses][cc-compat]).  
Section 13 of hakkiri's AGPLv3 allows a combination with GPLv3 work.

It is the only candidate whose tags equal Vale's, and every tag-based rule depends on them.

Alternatives:

- `english-pos-tagger`: a Brill tagger with Penn tags. Lost: its tags differ from Vale's.
- `nlprule`: its LanguageTool 5.2 data is LGPL-2.1 ([nlprule readme][nlprule-readme]).
- No tagger, refusing `tag` and `upos` tokens. Lost: a style written for Vale stops loading.

### 10. Spelling ports Vale's Hunspell reader

hakkiri ports Vale's `spell` package, about 2,900 lines of Go under MIT ([LICENSE][v-spell-license]) (operator, 2026-11-10).  
It needs no C++ build, and it reads `ICONV` and the affix flags as Vale does.  
The `affix` field reuses it.

Alternatives:

- `spellbook`: MPL-2.0, which [deny.toml][deny] does not allow.
- `zspell`: its license is not a standard identifier.
- `hunspell-rs` over `hunspell-sys`: builds the C++ Hunspell library.
- `symspell`: reads word lists, and no affix rules.

### 11. The built-in dictionary is Vale's

hakkiri embeds `en_US-web.aff` and `en_US-web.dic`, 81,558 words built from SCOWL, now the English Speller Database (operator, 2026-11-10).  
Its [license][esdb-license] allows any use, if the copyright notice ships with every copy, so the notice ships beside the files.

Alternative: no built-in dictionary, so a `spelling` rule must name one. Lost: a style written for Vale expects it.

### 12. A style other than AP or Chicago is refused

Vale reads any other value as AP ([capitalization.go][v-cap-new]).  
hakkiri refuses the rule with an error that names the value (operator, 2026-11-10).  
hakkiri does better than Vale where Vale's behaviour is a defect.

Alternative: read it as AP. Lost: a misspelt `chicago` would apply AP with no sign.

### Crates read

Each crate was read on crates.io on 2026-11-10.

| Crate | Version | Released | License | deny.toml |
|:--|:--|:--|:--|:--|
| `titlecase` | 3.6.0 | 2025-06-05 | MIT | allowed |
| `english-pos-tagger` | 0.2.0 | 2026-07-08 | MIT | allowed |
| `nlprule` | 0.6.4 | 2021-04-24 | MIT OR Apache-2.0, data LGPL-2.1 | code allowed |
| `pos-tagger` | 0.2.0 | 2026-07-08 | GPL-3.0-or-later | refused |
| `spellbook` | 0.4.2 | 2026-06-03 | MPL-2.0 | refused |
| `zspell` | 0.5.5 | 2024-06-13 | non-standard | refused |
| `hunspell-rs` | 0.4.0 | 2022-09-18 | Apache-2.0 | allowed |
| `hunspell-sys` | 0.3.1 | 2022-10-24 | MIT OR Apache-2.0 | allowed |
| `symspell` | 0.5.2 | 2026-03-22 | MIT | allowed |

## Risks / Trade-offs

- [Tag scenarios assume tags such as `MD` for `can` and `DT` for `the`] → each scenario uses a word that every candidate tags the same way.  
  A tagger that differs fails the test in CI, before any user sees it.
- [The model's share-alike license binds anyone who changes the model file] → the model stays a separate data file with its own notice.
- [Porting about 5,600 lines of Go is the largest task in the engine work] → the tasks split the tagger, the converters and the reader.  
  Each has its own tests.

[engine-design]: ../archive/2026-11-10-rule-engine/design.md
[pattern-design]: ../archive/2026-11-10-pattern-and-readability-checks/design.md
[project-config]: ../../specs/project-config/spec.md
[format-7]: ../archive/2026-11-10-rule-format/design.md#7-intended-differences-from-vale
[deny]: ../../../deny.toml
[v-seq-new]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/sequence.go#L123-L236
[v-seq-tokens]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/sequence.go#L289-L330
[v-seq-match]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/sequence.go#L387-L497
[v-seq-run]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/sequence.go#L643-L733
[v-tag-text]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/nlp/prose.go#L253-L288
[v-upos]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/upos.go#L78-L95
[v-cap-new]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/capitalization.go#L72-L109
[v-case]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/variables.go#L87-L144
[v-spell-new]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/spelling.go#L74-L135
[v-spell-run]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/spelling.go#L188-L280
[v-spell-explain]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/spelling.go#L335-L355
[v-spell-dicpath]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/spelling.go#L450-L506
[v-spell-license]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/spell/LICENSE
[v-affix]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/affix.go#L11-L70
[v-find-affix]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/vocab.go#L93-L102
[p-title]: https://github.com/jdkato/prose/blob/b33e60dbff0492289b227f3681120ac057d8131b/strcase/title.go#L99-L164
[p-sentence]: https://github.com/jdkato/prose/blob/b33e60dbff0492289b227f3681120ac057d8131b/strcase/sentence.go#L62-L119
[p-models]: https://github.com/jdkato/prose/blob/b33e60dbff0492289b227f3681120ac057d8131b/tag/lang/MODELS.md
[nlprule-readme]: https://github.com/bminixhofer/nlprule/blob/ebb54617ea5f3378c0dc63486631724b7e97e13c/README.md#license
[esdb-license]: https://github.com/en-wl/wordlist/blob/1e5b7d3a72f47a71da5d28686c1dd4b397178485/Copyright
[cc-compat]: https://creativecommons.org/share-your-work/licensing-considerations/compatible-licenses
