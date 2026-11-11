# Design

## Context

[`vocabularies`][vocabularies] reads `accept.txt` and `reject.txt` and holds back the matches the accepted terms name.
Vale also builds two rules from the lists, in [`loadVocabRules`][v-load-rules], which runs when the style `Vale` is needed.
hakkiri has no style `Vale` yet, and `HKR-026` and `HKR-038` will add `Spelling` and `Repetition` to it.

## Goals / Non-Goals

Goals:

- `Vale.Terms` and `Vale.Avoid` give the alerts of Vale's rules, with the same messages and levels.
- A config runs them the way a Vale config does: the style `Vale`, or the rule's name.
- A glob's vocabularies build rules of their own, which run on the glob's files.

Non-goals:

- The alert's action, which hakkiri loads and does not apply.

## Decisions

### 1. Terms and Avoid are rules of a style named Vale

hakkiri builds `Vale.Terms` when a vocabulary holds an accepted term, and `Vale.Avoid` when it holds a rejected one ([manager.go][v-load-rules]).
`Vale.Terms` is a `substitution` rule: each accepted term lowercased is a swap key with the term as its value, matched without regard to case, with the boundaries `(?<!\w)` and `(?!\w)` and not `\b`.
The message is `Use '%s' instead of '%s'.`, the level `error` and the scope `text`, and the value shown is the term in the case of the text it replaced ([substitution.go][v-recase]).
`Vale.Avoid` is an `existence` rule over the rejected terms, case-sensitive, between word boundaries, with the message `Avoid using '%s'.`, the level `error` and the scope `text`.
Accepted terms are exceptions of `Vale.Avoid`, not of `Vale.Terms` ([definition.go][v-default-rules]).

The probe of Vale at `66bbb85` (2026-11-11) found these:

- `github` and `Github` each draw `Use 'GitHub' instead of 'github'.`, and `GitHub` draws nothing.
- `harness(?:es)?` draws `Use 'harnesses' instead of 'Harnesses'.` for `Harnesses`, so a pattern is shown as the spelling it matched.
- `Node.js` matches `node.js` and not `nodexjs`, so a period of a plain term is literal.
- A rejected term that is also accepted draws no `Vale.Avoid` alert.
- `foo.*bar` draws one alert over the whole span from the first `foo` to the last `bar`.
- `Bad Word` matches `Bad  Word` with two spaces, and not `bad word`.
- `colou?r` draws `Use 'color' instead of 'COLOR'.` and `Use 'colour' instead of 'Colour'.`, and a term with a class, a wildcard or a repeat is shown as written.
- Two terms with one lowercase form, `iOS` and `IOS`, give one swap key, and the later one is the value: `ios`, `Ios` and `iOS` each draw `Use 'IOS' instead of ...`.
- Vale lowercases the whole pattern, escapes included, so `data\Wset` builds the key `data\wset`: it flags `dataxset`, draws `Use 'data\Wset' instead of 'dataxset'.`, and leaves `data set` alone. hakkiri does the same, to match Vale.
- A phrase that wraps matches across the line break: `mea\nCulpa` draws `Use 'mea culpa' instead of 'mea Culpa'.`, and `mea\nculpa` draws nothing.
- Nothing runs from a vocabulary that a config lists without the style `Vale`, and `Vale.Terms: off` leaves `Vale.Avoid` on.
- A glob's own rule turns on after the glob's settings, so `Vale.Team.Terms: off` beside `vocab: [Team]` in one glob leaves it on.

Alternative: swap keys that keep the term's case and match with the `(?i)` flag.
It lost because Vale lowercases the key, so a term written in capitals and one written in lowercase give one key, and the later term wins.

### 2. The rules run where the style Vale or their name is listed

A rule runs on a file when its style is listed for the file, or its name is set ([lint.go][v-should-run]).
So `styles: [Vale]` runs both rules, and `Vale.Terms: warning` runs `Terms` at that level, as `BasedOnStyles = Vale` and `Vale.Terms = warning` do.
hakkiri's rule settings already run a rule whichever style holds it.
With no accepted term there is no `Vale.Terms`, and a setting that names it is an error, as for any rule no loaded style holds.

The names are Vale's own, so a ProseStyle config reads the same in both tools.
Alternative: run both rules whenever a vocabulary is named.
It lost because the probe shows Vale keeps them off unless the style or the name is listed, and a project that lists only `Team` gets no `Terms` alerts.
Alternative: own names such as `Vocab.Terms`.
It lost because a name the operator did not choose is a decision for the operator, and Vale's names need none.

### 3. A glob's vocabulary builds rules named after the vocabulary

A glob that names `Team` builds `Vale.Team.Terms` and `Vale.Team.Avoid` from that vocabulary, as `Vale.Terms` and `Vale.Avoid` are built from the top level ([manager.go][v-load-rules]).
Vale turns them on for the files the glob matches, so no style or setting is needed, and the probe shows `Use 'harness' instead of 'HARNESS'.` from `Vale.Team.Terms` with only a `Vocab` in the section.
Alternative: build one `Vale.Terms` over the lists of the file.
It lost because Vale's names and its alerts carry the vocabulary, and a project sets a level for one vocabulary's rule.

### 4. The style Vale needs a vocabulary term

`styles: [Vale]` is an error today, because no package holds a style `Vale`.
It stays an error when no named vocabulary holds a term, and `HKR-026` and `HKR-038` will give the style rules that need no vocabulary.
Alternative: a style `Vale` that is always present and holds no rule.
It lost because an empty style hides a wrong vocabulary name: the config would load and lint nothing.

### 5. A package style named Vale is refused

Vale refuses a style named `Vale` beside its built-in one ([manager.go][v-load-rules]).
hakkiri refuses it too, with an error on the package's entry, but only when a vocabulary gives the built-in style a rule: a project with no vocabulary term keeps a package style named `Vale`.
Alternative: refuse it always.
It lost because it would break a config that has never named a vocabulary.

[vocabularies]: ../archive/2026-11-11-vocabularies/design.md
[v-load-rules]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/manager.go#L559-L628
[v-recase]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/substitution.go#L453-L464
[v-default-rules]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/definition.go#L133-L154
[v-should-run]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/lint.go#L841-L890
