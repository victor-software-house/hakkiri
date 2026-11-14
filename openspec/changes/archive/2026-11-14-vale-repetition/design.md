# Design

## Context

Vale's `repetition` check is already ported: `run_repetition` in `src/check.rs` follows [repetition.go][v-check], and the `repetition` check type loads from a rule file.
What was missing is the built-in rule that Vale registers for its style `Vale`, and the style itself.

## Goals / Non-Goals

Goals:

- `styles: [Vale]` runs `Vale.Repetition` with Vale's defaults, with the alerts Vale gives.
- A config sets it by level, `on` and `off`, as it sets any rule.

Non-goals:

- `Vale.Spelling`, the other built-in rule: row `HKR-026`.
- Rule parameters such as `Vale.Repetition.max = 2`, which the config reads for no rule yet: row `HKR-059`.

## Decisions

### 1. The rule is built in code, like `Terms` and `Avoid`

`vale_repetition_rule` in `src/rule/build.rs` makes the `Repetition` check from Vale's defaults and the project's accepted terms, as `terms_rule` and `avoid_rule` make theirs.
Alternative: an embedded rule file read by the rule loader.
It lost because the vocabulary rules are already built in code, and a file would add a second way to spell the same defaults.

### 2. The style `Vale` always exists

Vale registers `Vale.Repetition` and `Vale.Spelling` for every section that lists `Vale`, and then adds the vocabulary rules ([manager.go][v-manager]).
hakkiri builds the style `Vale` from `Vale.Repetition` and, when a top-level vocabulary has terms, `Terms` and `Avoid`.
A glob's vocabulary still builds `Vale.<Name>.Terms` and `Vale.<Name>.Avoid` only.
Alternative: keep refusing `Vale` when no term exists.
It lost because Vale runs a rule from it, and a package that lists `Vale` for repetition alone would not load.

### 3. Accepted terms hold a repeat back through the rule's `vocab`

The rule reads the accepted terms of the project as exceptions, as `vocab: true` does in Vale, and the engine passes a glob's accepted terms to it as it passes them to any `repetition` rule.
The probe shows both hold back `Bye bye`, `had had` and `That that`.

### 4. Words compare by simple case folding

Vale compares with Go's `strings.EqualFold`, which folds each character on its own.
hakkiri lowercased both whole words, and Rust lowercases a `Σ` at the end of a word to `ς`, so `ΣΑΣ` and `σασ` differed, where Vale reports them.
`equal_fold` folds each character by the lowercase of its uppercase when each is one character, which gives `Σ`, `σ` and `ς` the same `σ`, and keeps `ß`, `İ` and the dotless `ı` apart as Go does.
Alternative: a case-folding crate.
It lost because one function covers the probe, and a dependency needs a decision.

### 5. What the probe shows and hakkiri keeps

The probe of Vale at `66bbb85` over four Markdown files, a Rust file and five configs gave 86 alerts under the plain config.
hakkiri gives the same alerts, with these results kept on purpose because Vale gives them:

- A run of three words gives two alerts, `Three three` and `three three`, since the count restarts after an alert.
- A word with an apostrophe, a hyphen, a quote or a digit is no repeat, since `alpha` wants letters only.
- Two words in different sentences are no repeat.
- A code span breaks the chain.

[v-check]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/repetition.go
[v-manager]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/manager.go#L498-L521
