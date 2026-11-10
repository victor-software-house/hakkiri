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
- Read every dictionary, word list and affix file from the pinned package that holds the rule.

**Non-Goals:**

- Spelling suggestions and fixes: `action` rides along, as in the rule-engine change.
- Vale's built-in `Vale.Spelling` rule and its `config/ignore` default; a style names its own lists.
- Languages other than English.

## Decisions

### 1. spelling reads words as Vale's tokenizer cuts them

Spelling words come from Vale's word tokenizer, which splits on spaces and trims listed prefixes and suffixes ([spelling.go][v-spell-run]).  
The metric check's word pattern is not used here.  
Each word is converted by the dictionary's `ICONV` table, and skipped by the built-in filters, unless `custom: true`, and by `filters`.  
The remaining words are looked up.

`split` takes an identifier apart and reports each misspelt part of three letters or more at its own position.  
Vale's two message rewrites stay, for a casing mistake and for an identifier in Markdown or MDX ([spelling.go][v-spell-explain]).  
A word that overlaps a code span gives no alert, as the rule-engine change decides for `existence`.

Alternative: one word splitter for every check.  
Lost: Vale's spelling and metric checks cut words differently, and either choice changes one of them.

### 2. Dictionaries and lists stay inside the package

`dictionaries`, `dicpath`, `aff`, `dic`, `ignore` and `affix` resolve inside the package that holds the rule.  
`affix: en_US-web`, and a rule with no dictionary, mean the built-in files.  
A path that leaves the package is refused.

Alternative: Vale's search, which also tries an absolute path, the working directory and `DICPATH` ([spelling.go][v-spell-dicpath], [vocab.go][v-find-affix]).  
Lost: the [project config][project-config] pins each package by hash, and a dictionary found on the machine would make two runs of one pin differ.

### 3. Fields Vale reads and never uses

A `spelling` rule's own `exceptions` and its `threshold` load with no effect.  
Vale builds its exception pattern before it decodes the rule's fields, and reads `threshold` nowhere ([spelling.go][v-spell-new]).

Alternative: apply `exceptions` as a skip list.  
Lost: imported rules would report fewer words than in Vale, a difference the [format design][format-7] does not list.

### 4. affix expands before compiling

A `word/FLAGS` part of an `existence` token or a `substitution` key becomes an alternation before the rule compiles ([affix.go][v-affix]).  
The alternation holds the word and the forms its flags name.  
The expansion uses the dictionary reader's affix rules, which `main` holds, and the rule-engine change's refusal ends here.

[case-design]: ../archive/2026-11-10-case-and-sequence-checks/design.md
[case-reader]: ../archive/2026-11-10-case-and-sequence-checks/design.md#6-spelling-ports-vales-hunspell-reader
[project-config]: ../../specs/project-config/spec.md
[format-7]: ../archive/2026-11-10-rule-format/design.md#7-intended-differences-from-vale
[v-spell-new]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/spelling.go#L74-L135
[v-spell-run]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/spelling.go#L188-L280
[v-spell-explain]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/spelling.go#L335-L355
[v-spell-dicpath]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/spelling.go#L450-L506
[v-find-affix]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/vocab.go#L93-L102
[v-affix]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/affix.go#L11-L70
