# Design

## Context

The [`case-and-sequence-checks` change][case-design] built the Hunspell reader and the built-in dictionary ([decisions 6 and 7][case-reader]).  
This change adds the two checks that read them:

- `spelling` checks each word against a dictionary ([spelling.go][v-spell-run]).
- `affix` expands `word/FLAGS` in `existence` and `substitution` ([affix.go][v-affix]).

Each Vale behaviour below was read at `66bbb85`, which is 3.26.0 plus 16 commits.

## Goals / Non-Goals

**Goals:**

- Give `spelling` and `affix` Vale 3.26.0's meaning, so the importer, `HKR-007`, converts upstream styles without changing what they report.
- Read every dictionary, word list and affix file from the pinned package that contains the rule.

**Non-Goals:**

- Spelling suggestions and fixes: `action` rides along, as in the rule-engine change.
- Vale's built-in `Vale.Spelling` rule and its `config/ignore` default; a style refers to its own lists.
- Languages other than English.

## Decisions

### 1. spelling reads words as Vale's tokenizer cuts them

Spelling words come from Vale's word tokenizer, which splits on spaces and trims listed prefixes and suffixes ([spelling.go][v-spell-run]).  
Spelling does not use the metric check's word pattern.  
Each word goes through the dictionary's `ICONV` table.  
The built-in filters skip words, unless `custom: true`, and so do `filters`.  
Then the remaining words are looked up.

`split` takes an identifier apart and reports each misspelt part of three letters or more at its own position.  
Vale's two message rewrites stay, for a casing mistake and for an identifier in Markdown, or in Markdown with JavaScript XML (MDX) ([spelling.go][v-spell-explain]).

A word that overlaps a code span gives no alert, as the rule-engine change decides for `existence`.

A word with a curly quote or `&rsquo;` is read with a straight quote, as Vale reads it.  
hakkiri reports it where it stands in the file, as written.  
Vale gives such a word no position ([tokenize.go][v-tokenize-offsets]), so its alert lands at a wrong place.  
The text of Vale's alert is the straight form.

The text of an autolink, a URL or an email address is a gap in the prose around it ([node-segments spec][node-segments], [format design][format-7]).  
So no word in it is read.  
Vale reads an email address, so it reports one when `custom` turns the filters off.  
The [parity test][parity-test] lists these alerts as departures.

Alternative: one word splitter for every check.  
Lost: Vale's spelling and metric checks cut words differently, and either choice changes one of them.

### 2. Dictionaries and lists stay inside the package

`dictionaries`, `dicpath`, `aff`, `dic`, `ignore` and `affix` resolve inside the package that contains the rule.  
`affix: en_US-web`, and a rule with no dictionary, mean the built-in files.

A path that leaves the package is refused.  
So is a rule that sets only one of `aff` and `dic`, which Vale answers with the built-in dictionary.  
A list in `ignore` that does not exist loads nothing, as in Vale.

Alternative: Vale's search, which also tries an absolute path, the working directory and `DICPATH` ([spelling.go][v-spell-dicpath], [vocab.go][v-find-affix]).  
Lost: the [project config][project-config] pins each package by hash, and a dictionary found on the machine would make two runs of one pin differ.

### 3. Fields Vale reads and never uses

Vale accepts `exceptions` and `threshold` in a `spelling` rule and uses neither.  
It builds its exception pattern before it decodes the rule's fields, and reads `threshold` nowhere ([spelling.go][v-spell-new]).

A probe of Vale 66bbb85 ran four rules and gave the same seven alerts for each.  
Among them were `blorft` and a misspelling of `receive`.  
The rules were: one with `exceptions` that list both words, one with `threshold: 99`, one with both fields, and one with neither.

hakkiri refuses a rule that sets either one, with a message that says the field has no effect on a spelling rule.  
This is an intended difference from Vale: a rule that Vale loads and hakkiri refuses.  
A `threshold` of 0 reads as unset, as in a `capitalization` rule.

Alternative: load both and ignore them, as Vale does.  
Lost: the rule's author wrote the field to change what is reported, and it would change nothing, with no sign.  
Alternative: apply `exceptions` as a skip list.  
Lost: imported rules would report fewer words than in Vale, a difference the [format design][format-7] does not list.

### 4. affix expands before compiling

A `word/FLAGS` part of an `existence` token or a `substitution` key becomes an alternation before the rule compiles ([affix.go][v-affix]).  
The alternation has the word and the forms its flags list.  
It uses the dictionary reader's affix rules, which `main` has.  
The rule-engine change's refusal ends here.

Each space-separated part of a token is read alone.  
A part with no unescaped slash, or a slash written `\/`, is text.  
A `\b` at either end of a part stays outside the alternation.

The word before the slash must be plain text.  
Each form is escaped the way Go's `regexp.QuoteMeta` escapes it.  
A rule with no `affix` keeps every slash.

The file is `name.aff` in the package's dictionaries folder.  
It wins over the built-in file of the same name `en_US-web`, as in Vale ([vocab.go][v-find-affix]).

A probe of Vale 66bbb85 and a parity test cover these cases:

- the built-in file and a package file
- `\b`
- an escaped slash
- a part with no flags
- `ignorecase`
- the swap keys of `substitution`

The same probe refused each of these as a rule error, as hakkiri does:

- a flag that has no class
- a word that is not plain text
- flags with no word
- a file that is missing

One difference is read from Vale's source and not probed: two keys of a `swap` that expand to the same text.  
Vale keeps the replacement of the key its map visits last.  
That key can change from run to run.  
hakkiri keeps both in the order written, so the first gives the replacement.

[case-design]: ../2026-11-10-case-and-sequence-checks/design.md
[case-reader]: ../2026-11-10-case-and-sequence-checks/design.md#6-spelling-ports-vales-hunspell-reader
[project-config]: ../../../specs/project-config/spec.md
[format-7]: ../2026-11-10-rule-format/design.md#7-intended-differences-from-vale
[node-segments]: ../../../specs/node-segments/spec.md#requirement-a-url-is-a-gap-in-the-prose-around-the-link
[parity-test]: ../../../../tests/spelling_parity.rs
[v-tokenize-offsets]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/nlp/tokenize.go#L80-L148
[v-spell-new]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/spelling.go#L74-L135
[v-spell-run]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/spelling.go#L188-L280
[v-spell-explain]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/spelling.go#L335-L355
[v-spell-dicpath]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/spelling.go#L450-L506
[v-find-affix]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/vocab.go#L93-L102
[v-affix]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/affix.go#L11-L70
