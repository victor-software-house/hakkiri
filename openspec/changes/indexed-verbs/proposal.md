# Proposal

Queue row: `HKR-045` in [tasks.yaml][ledger].

## Motivation

A Vale rule message can name its values by number, such as `%[1]s` and `%[2]s`, because Vale gives the message to Go's `Sprintf` ([`CondSprintf`][v-format]).
ProseStyle `NounStack` and `ThingsSpeakNoun` write theirs that way.
hakkiri fills only plain `%s`, so it prints `%[1]s %[2]s` as written in those two messages.

## Changes

- A message fills `%[n]s` with the `n`th value, counting from 1, and a `%s` after it takes the value that follows, as Go does.
- A verb with no value to take is nothing, as a plain `%s` already was, where Vale prints Go's `%!s(MISSING)` or `%!s(BADINDEX)`.
  The `vale-differences` spec records it.
- A new fixture holds the two ProseStyle rules and three more, with the messages Vale prints for them.

## Capabilities

### Modified Capabilities

- `alerts`: a message fills `%[n]s` as well as `%s`.
- `vale-differences`: a verb with no value reads as nothing.

## Impact

- Changed: `src/alert.rs` (`fill`), `src/sequence/tests.rs`, `tests/alerts.rs`, `tests/test_command.rs`, `src/instructions.md`, the skill and `AGENTS.md` where they state the fill rule.
- New: `tests/fixtures/sequence-verbs`.
- No new dependency.

[ledger]: ../../../tasks.yaml
[v-format]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/util.go#L153-L163
