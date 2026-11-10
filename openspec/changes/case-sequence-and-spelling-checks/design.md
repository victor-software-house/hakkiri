# Design

## Context

The [`rule-engine` change][engine-design] selects sentences and nodes, builds check text and places alerts.  
The [`pattern-and-readability-checks` change][pattern-design] adds four more check types on top of it.  
`capitalization` and `sequence` need data that neither change holds, and so do `spelling` and `affix`, which the [`spelling-and-affix-checks` change][spelling-design] builds on the dictionary reader decided below:

- `sequence` needs words and Penn Treebank tags for each sentence ([sequence.go][v-seq-run]).
- `capitalization` needs title and sentence case converters, and AP title case reads one tag ([title.go][p-title]).
- `spelling` and `affix` need a Hunspell dictionary and its affix rules ([spelling.go][v-spell-run], [affix.go][v-affix]).

Vale gets the tagger and the converters from prose.  
Its dictionary reader is its own `spell` package, MIT and derived from gospell ([LICENSE][v-spell-license]).

Each Vale behaviour below was read at `66bbb85`, which is 3.26.0 plus 16 commits.  
Two of those commits touch `sequence` and `repetition`, and only to choose the language a file is read in.

## Goals / Non-Goals

**Goals:**

- Give `capitalization` and `sequence` Vale 3.26.0's meaning, so the importer, `HKR-007`, converts upstream styles without changing what they report.
- Hold the data these checks and the spelling checks read: the tagger, the dictionary reader and the built-in dictionary.

**Non-Goals:**

- Fixes: `action` rides along, as in the rule-engine change.
- Languages other than English.

## Decisions

### 1. Case conversion ports prose's converters

The sentence and title converters and their word lists are ported from prose's `strcase` package, under MIT ([title.go][p-title], [sentence.go][p-sentence]).  
Vale's agreement count is ported from its `title` and `sentence` functions ([variables.go][v-case]).

In sentence case, a text fails when its first word fails.  
Otherwise it passes when enough words agree: equal to the converted form, in capitals, or matching an exception.  
The share that agrees must be at least `threshold`.

Vale hands the converters its own `exceptions` list after [`updateExceptions`][v-update-exceptions] has sorted it and rewritten each term in place, as `(?-i)` and the term with its periods escaped.  
The converters therefore match an exception as a case-sensitive pattern, and keep a word as written when one matches it.  
They do not set a word to the case an exception was written in.  
hakkiri hands its converters the same rewritten list.

Three more points follow Vale or prose as they are:

- A text with no word has a share of 0 over 0, which is not at least `threshold`, so it fails.
- A pattern that gives up on a text finds nothing, as prose's wrapper reads it.
- An empty `match` is refused, since every text would pass it, and a `style` other than `AP` or `Chicago` is refused where Vale reads any other value as `AP`.

One point is a gap, not a departure: `vocab` loads and has no effect.  
In Vale, `vocab` defaults to true, and `true` adds the project's accepted Vocab terms to the rule's exceptions, both in the exceptions pattern and in the list the converters read ([capitalization.go][v-capitalization-vocab]).  
hakkiri has no project vocabulary yet, so a rule's exceptions are only its own.  
When Vocab lands, `vocab` must add the accepted terms before `exceptions` is rewritten and sorted.

The title converter's tagging and its checks of the text around a word use prose's byte offsets, as prose writes them.  
Prose counts the width of each word in characters and adds it to a byte offset, and searches a copy of the title whose curly quotes and dashes are plain marks.  
hakkiri keeps both, so a title with those marks gets the same words in lower case as in Vale.

Alternative: `titlecase` 3.6.0, which applies John Gruber's rules.  
Lost: it keeps a different list of small words, so headings that pass in Vale would fail.

### 2. AP title case keeps Vale's tag lookup

In AP style, `to` is lower case only when the tag after it starts with `NN` ([title.go][p-title]).  
So `to` before an article starts with a capital.

Vale reads that tag by the position of the word among the converter's words.  
The tags come from prose's Treebank word tokenizer and the tagger, not from the tokenizer that `sequence` uses, and a contraction or hyphen before `to` shifts the two lists.  
hakkiri ports that tokenizer, and reads the tag the same way.  
Go reads each map of its replacements in an order that changes between runs, and hakkiri reads them in the order the file lists them.  
A test compares its words with prose's on over 600 texts.

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

Vale's walk has two quirks that hakkiri keeps ([sequence.go][v-seq-run]):

- It starts once for each place the anchor's pattern occurs, or each word for a tag anchor, and each start passes over the words earlier starts took. A sentence can give fewer alerts than the pattern has places.
- A `target` on a token with `skip` narrows the alert only when every copy of the token read a word.

Vale cuts each block again with prose's segmenter and tags each part apart ([prose.go][v-tag-text]), and skips the tagger when a literal that a token needs is missing.  
hakkiri tags its own sentence whole, so a text the two cut differently can get other tags.

Two costs differ:

- Vale makes one step for each copy of a `skip` and `min` ([sequence.go][v-seq-tokens]). With a 6 GB limit, `skip: 1000` with `min: 1000` ran out of memory in 13 seconds. hakkiri counts the steps and stores none.
- Vale finds each start by scanning from the first word and through the earlier starts, so a sentence whose anchor matches most words costs the cube of its words. It took 18 seconds for 8,000 words and passed 100 for 16,667. hakkiri takes the next word the anchor accepts, and reads 16,667 words in 31 ms.

hakkiri refuses what Vale loads and never matches: tokens with nothing to start from (a `pattern`, `tag` or `upos` without `negate`), a token field it does not define, and a `skip` or `min` below 0.  
It refuses a `skip` and `min` whose steps overflow a count too.

Alternative: compile the tokens into one pattern over a string of `word/TAG` pairs.  
Lost: `negate` at a sentence edge and `target` spans have no direct form there, and results would drift from Vale's walk.

### 4. model is refused

`model` names another checker's tagger model in Vale's dictionary folder ([sequence.go][v-seq-new]).  
hakkiri refuses a rule that sets it.

Alternative: read Vale's model format.  
Lost: it ties hakkiri to a Vale file format, for a field no known style uses.

### 5. Tags come from a port of prose's tagger and model

hakkiri ports prose's averaged perceptron and its English tokenizer, `tokenize.New()`, and embeds the English model (operator, 2026-11-10).  
Vale tags each sentence with that tokenizer and the perceptron, so the port is those two and not prose's Treebank tokenizer ([tag step][v-tag-text]).  
The model is trained on UD English EWT under CC BY-SA 4.0, and weighs 1.9 MB ([MODELS.md][p-models]).  
It ships as its own data file with its notice.

The port keeps prose's case and category tests where Go and Rust agree.  
The shape of a word reads Rust's Unicode properties and not Go's categories, so a word of characters such as `²` can get another shape, and so another feature.  
A test compares tokens and tags with prose's on 188 sentences.

The port splits a token's prefixes, suffixes and contractions in one pass, with the same tokens.  
prose searches what is left of a token for a contraction ending again after each prefix or suffix it takes off, so a token of 20,000 marks takes quadratic time there.

Creative Commons lists GPLv3 as compatible with BY-SA 4.0 ([compatible licenses][cc-compat]).  
Section 13 of hakkiri's AGPLv3 allows a combination with GPLv3 work.

It is the only candidate whose tags equal Vale's, and every tag-based rule depends on them.

Alternatives:

- `english-pos-tagger`: a Brill tagger with Penn tags. Lost: its tags differ from Vale's.
- `nlprule`: its LanguageTool 5.2 data is LGPL-2.1 ([nlprule readme][nlprule-readme]).
- No tagger, refusing `tag` and `upos` tokens. Lost: a style written for Vale stops loading.

### 6. Spelling ports Vale's Hunspell reader

hakkiri ports Vale's `spell` package, about 2,900 lines of Go under MIT ([LICENSE][v-spell-license]) (operator, 2026-11-10).  
It needs no C++ build, and it reads `ICONV` and the affix flags as Vale does.  
The `affix` field reuses it.

The port is smaller than Vale's package in three ways, none of which changes what an English dictionary accepts:

- It makes every form of every entry when it loads, where Vale makes the forms of an entry when it looks a word up.
- It leaves out the directives for compounds made by flag (`COMPOUNDFLAG`, `COMPOUNDBEGIN`, `COMPOUNDMIDDLE`, `COMPOUNDEND`), for suggestions, for Turkic casing and for `ß`. A dictionary that uses them loads, and a German or Dutch compound is not accepted.
- It reads UTF-8 and ISO 8859-1 and refuses any other `SET` with an error that names it (operator, 2026-11-10). Vale reads every character set that `golang.org/x/text` knows, and reading them here would add `encoding_rs` as a dependency.

It differs from Vale's package in two behaviours, where the port does what Hunspell does or what costs less:

- A rule whose strip text is not on the word makes no form, as in Hunspell. Vale puts the affix on the whole word, so `cat` with a rule that strips `y` and adds `ies` gives `caties` there.
- A word split at its breaks keeps the result of each part for each depth, so a token of many hyphenated parts takes polynomial time. Vale checks every part again at each split, and caches only the result for the whole word, so such a token takes exponential time there.

A flag is one byte in a file that declares `SET UTF-8` and no `FLAG`, and one character in any other file, as in Vale.  
In a Latin-1 file, `é` and `è` are therefore two flags.

A test compares the words accepted with Vale's on about 9,000 words, run in Go.

Alternatives:

- `spellbook`: MPL-2.0, which [deny.toml][deny] does not allow.
- `zspell`: its license is not a standard identifier.
- `hunspell-rs` over `hunspell-sys`: builds the C++ Hunspell library.
- `symspell`: reads word lists, and no affix rules.

### 7. The built-in dictionary is Vale's

hakkiri embeds `en_US-web.aff` and `en_US-web.dic`, 81,558 words built from SCOWL, now the English Speller Database (operator, 2026-11-10).  
Its [license][esdb-license] allows any use, if the copyright notice ships with every copy, so the notice ships beside the files.

Alternative: no built-in dictionary, so a `spelling` rule must name one. Lost: a style written for Vale expects it.

### 8. A style other than AP or Chicago is refused

Vale reads any other value as AP ([capitalization.go][v-cap-new]).  
hakkiri refuses the rule with an error that names the value (operator, 2026-11-10).  
hakkiri does better than Vale where Vale's behaviour is a defect.

Alternative: read it as AP. Lost: a misspelt `chicago` would apply AP with no sign.

### 9. A part reads an element and everything inside it

Vale gives each block of text the scope of its own element ([ast.go][v-tag-scope], measured at `66bbb85`).  
The text of a list item in a quote reads as `list`, a heading or table cell in a quote as `heading` or `table.cell`, and a quote in a list item as `blockquote`.  
Only the first paragraph of an item or quote reads as the item or quote.  
The paragraphs after it read as `paragraph`.

hakkiri's tree gives a list item or a quote the text of everything inside it, as the [node-segments spec][node-segments] fixes.  
So a part reads an element and everything inside it, as `~` does at any depth ([format design 7][format-7]).  
Four cases follow, and the `scope-selection` spec records them:

- `list` and `blockquote`, alone or joined to `sentence`, also read what Vale gives to the element inside.
- `paragraph` leaves out the later paragraphs of an item or a quote.
- `~list` and `~blockquote` leave out text that Vale keeps.
- `sentence.list` is refused as an unknown scope, where Vale reads it as `sentence & list`.

A test runs an `existence` and a `sequence` rule for 24 scopes over five Markdown files that nest sentences in items, quotes, tables and headings.  
Of the 1,735 alerts Vale gave to the rules hakkiri loads, 1,610 are hakkiri's too, and `departures.tsv` lists the other 125 and the 156 that only hakkiri gives.  
The other scopes tested give Vale's alerts, `heading`, `table`, `text` and `sentence` among them.

Alternative: Vale's own element only.  
Lost: it changes the text every `list` and `blockquote` rule reads, and the node-segments change fixed that text.

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
[spelling-design]: ../spelling-and-affix-checks/design.md
[deny]: ../../../deny.toml
[v-tag-scope]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/ast.go#L51-L58
[node-segments]: ../../specs/node-segments/spec.md
[format-7]: ../archive/2026-11-10-rule-format/design.md#7-intended-differences-from-vale
[v-seq-new]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/sequence.go#L123-L236
[v-seq-tokens]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/sequence.go#L289-L330
[v-seq-match]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/sequence.go#L387-L497
[v-seq-run]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/sequence.go#L643-L733
[v-tag-text]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/nlp/prose.go#L253-L288
[v-upos]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/upos.go#L78-L95
[v-cap-new]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/capitalization.go#L72-L109
[v-update-exceptions]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/definition.go#L473-L507
[v-case]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/variables.go#L87-L144
[v-spell-run]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/spelling.go#L188-L280
[v-spell-license]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/spell/LICENSE
[v-affix]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/affix.go#L11-L70
[v-capitalization-vocab]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/capitalization.go#L52-L70
[p-title]: https://github.com/jdkato/prose/blob/b33e60dbff0492289b227f3681120ac057d8131b/strcase/title.go#L99-L164
[p-sentence]: https://github.com/jdkato/prose/blob/b33e60dbff0492289b227f3681120ac057d8131b/strcase/sentence.go#L62-L119
[p-models]: https://github.com/jdkato/prose/blob/b33e60dbff0492289b227f3681120ac057d8131b/tag/lang/MODELS.md
[nlprule-readme]: https://github.com/bminixhofer/nlprule/blob/ebb54617ea5f3378c0dc63486631724b7e97e13c/README.md#license
[esdb-license]: https://github.com/en-wl/wordlist/blob/1e5b7d3a72f47a71da5d28686c1dd4b397178485/Copyright
[cc-compat]: https://creativecommons.org/share-your-work/licensing-considerations/compatible-licenses
