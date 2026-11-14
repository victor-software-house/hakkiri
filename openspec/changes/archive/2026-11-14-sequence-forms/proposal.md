# Proposal

Queue row: `HKR-062` in [tasks.yaml][ledger].

## Motivation

A `sequence` rule reads one list of `tokens`, so a check with several shapes needs one rule for each shape.
`ProseStyle.ThingsSpeak` is three: a noun and a present verb, a noun and a past verb with a guard word after it, and a pronoun and a verb.
Each rule is an alert name of its own, and a project that turns off or retunes the check must find all three.
The operator ruled on 2026-11-12 that a `sequence` rule takes alternative token lists, matches where any list matches, and gives one rule name to every alert, and picked the key `forms`.

## Changes

- A `sequence` rule takes `forms`: a list of mappings that each hold `tokens`.
  The rule matches where any form does, and the rest of the rule (`message`, `level`, `scope`, `ignorecase`, `exceptions`) applies to every form.
- A rule holds `tokens` or `forms`, not both, and not neither.
- Two forms that match overlapping words give one alert: the match that starts first stays, and the earlier form wins a tie.
  Matches of one form never drop each other, so a rule of one `tokens` list gives the alerts it gives today, which are Vale's.
- A fault in a form names it, as `forms[1].tokens[0].pattern`.
- The `sequence-check` spec states the key, and `vale-differences` records that Vale panics on a rule with `forms`.
- The instructions, the skill and `AGENTS.md` describe the key.

## Capabilities

### Modified Capabilities

- `sequence-check`: alternative token lists.
- `vale-differences`: a rule file that only hakkiri reads.

## Impact

- New: `tests/sequence_forms.rs` and `tests/fixtures/sequence-forms`, with the three forms of `ThingsSpeak` and the one-list rules they must equal.
- Changed: `src/rule.rs`, `src/rule/schema.rs`, `src/rule/build.rs`, `src/sequence.rs`, `src/instructions.md`, the skill and `AGENTS.md`.
- `hakkiri::rule::Sequence` holds `forms` and `exceptions`, and the new `Form` holds what `Sequence` held: `tokens`, `ends`, `anchor` and `candidates`.
- No new dependency.

[ledger]: ../../../../tasks.yaml
