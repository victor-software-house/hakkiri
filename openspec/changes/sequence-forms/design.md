# Design

## Context

A `sequence` walk starts from an anchor word and checks the tokens left and right of it.
Each list of tokens has its own anchor and its own candidate starts, so a rule with several lists is several walks over the same tagged sentence.

## Goals / Non-Goals

Goals:

- One rule matches where any of several token lists does, with one rule name on every alert.
- A rule of one `tokens` list gives the alerts it gives today.

Non-goals:

- Settings per form.
  `message`, `level`, `scope`, `ignorecase` and `exceptions` stay rule-wide.
- A parameter that picks a form.
  `forms` holds lists and mappings, which a parameter cannot ([HKR-059][ledger]).

## Decisions

### 1. The key is `forms`, a list of mappings that each hold `tokens`

Each form is a self-contained `tokens` list, as the rule reads today, and a mapping leaves room for a setting per form later.
Alternative: `any`, a list of bare token lists, as ast-grep's [`any`][ast-grep-any] and the pattern lists of [spaCy's matcher][spacy-matcher] do.
It lost because the nested dashes are hard to read and a form has no place for a field.
Alternative: keep `tokens` as the first form and add `or` for the rest, as Semgrep's [`pattern-either`][semgrep-either] adds alternatives to a pattern.
It lost because the first form would differ in shape from the others.

### 2. A rule holds `tokens` or `forms`

A rule with both has two meanings for the list that matters, and a rule with neither has nothing to match.
Both cases are refused at load, naming the field.
Alternative: read `tokens` as one more form.
It lost because it hides which list the author meant to be first.

### 3. The earlier match stays when forms overlap

A stretch of words that two forms match is one error, so it gives one alert, as a [LanguageTool rule group][languagetool-group] does.
The match that starts first stays, the earlier form wins a tie, and a match from the same form never drops another.
Alternative: report both.
It lost because the author reads two alerts as two errors.
Alternative: drop overlaps inside one form too.
It lost because a rule of one `tokens` list would then differ from [Vale's walk][v-walk], which reports each start.

### 4. A rule file with `forms` is read by hakkiri alone

Vale at `66bbb85` reads `tokens` through an unchecked type assertion, so a rule with `forms` and no `tokens` stops Vale with a panic ([sequence.go][v-tokens]).
A package that must load in both tools keeps one rule for each form.

[ledger]: ../../../tasks.yaml
[ast-grep-any]: https://ast-grep.github.io/guide/rule-config/composite-rule.html
[spacy-matcher]: https://spacy.io/api/matcher
[semgrep-either]: https://semgrep.dev/docs/writing-rules/rule-syntax
[languagetool-group]: https://dev.languagetool.org/development-overview
[v-walk]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/sequence.go#L343-L497
[v-tokens]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/sequence.go#L289-L292
