# Proposal

Queue row: `HKR-038` in [tasks.yaml][ledger].
It follows [HKR-042][vocabulary-rules], which built the style `Vale` with `Terms` and `Avoid`.

## Motivation

Vale builds the rule `Vale.Repetition` into its style `Vale`, whenever a section lists that style ([definition.go][v-rules], [manager.go][v-manager]).
It reports a word that repeats, such as `the the`, at the level `error`, and a Vale package with `BasedOnStyles = Vale` relies on it.
hakkiri built the style `Vale` only while a top-level vocabulary held a term, and it held no repetition rule, so a project that moved from Vale lost the alerts, or had its config refused.

## Changes

- The style `Vale` always exists and holds `Vale.Repetition`, besides `Vale.Terms` and `Vale.Avoid` when a vocabulary gives them.
- `Vale.Repetition` has Vale's defaults: `extends: repetition`, level `error`, message `'%s' is repeated!`, scope `text`, `ignorecase`, `alpha`, `max` 0 and the tokens `[^\s.!?]+`.
- Accepted terms hold a repeat back, at the top level and in a glob, as in Vale.
- A level, `on` and `off` by name apply to it as to any rule.
- Two words differ in case without regard to the final `ς`, as Go's `strings.EqualFold` reads them, where hakkiri lowercased the whole word and so told `ΣΑΣ` from `σασ`.
- The `vale-differences` spec drops the gap for the repetition rule and the refusal of the style `Vale` with no term, and `Vale.Spelling` stays a gap.

## Capabilities

### New Capabilities

- `vale-repetition`: what the built-in rule reports, how it compares words, and how a config sets it.

### Modified Capabilities

- `project-config`: the style `Vale` always exists.
- `vale-differences`: two scenarios close.

## Impact

- New: `tests/builtin_repetition.rs` and its fixtures, with Vale's alerts recorded from Vale at `66bbb85`.
- Changed: `src/rule/build.rs`, `src/rule.rs`, `src/config.rs`, `src/check.rs`, the Vale-style fixtures that now turn `Vale.Repetition` off, `src/instructions.md`, `README.md` and `AGENTS.md`.
- No new dependency.

[ledger]: ../../../../tasks.yaml
[v-manager]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/manager.go#L498-L521
[v-rules]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/definition.go#L155-L169
[vocabulary-rules]: ../2026-11-11-vocabulary-rules/proposal.md
