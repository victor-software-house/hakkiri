# Proposal

Queue row: `HKR-062` in [tasks.yaml][ledger].

## Motivation

A `sequence` rule reads one list of `tokens`, so a check with several shapes needs one rule for each shape.  
`ProseStyle.ThingsSpeak` is three rules:

- a noun and a present verb;
- a noun and a past verb, with a guard word after it;
- a pronoun and a verb.

Each rule is an alert name of its own.  
A project that turns off or retunes the check must find all three.

On 2026-11-12 the operator ruled that a `sequence` rule takes alternative token lists.  
It matches where any list matches, and it gives one rule name to every alert.  
The operator picked the key `forms`.

## Changes

- A `sequence` rule takes `forms`: a list of mappings that each contain `tokens`.  
  The rule matches where any form does.  
  The rest of the rule (`message`, `level`, `scope`, `ignorecase`, `exceptions`) applies to every form.
- A rule has `tokens` or `forms`, not both, and not neither.
- Two forms that match overlapping words give one alert: the match that starts first stays, and the earlier form wins a tie.  
  Matches of one form never drop each other.  
  So a rule of one `tokens` list gives the alerts it gives today, which are Vale's.
- The error for a fault in a form shows its path, as `forms[1].tokens[0].pattern`.
- The `sequence-check` spec states the key, and `vale-differences` records that Vale panics on a rule with `forms`.
- The instructions, the skill and `AGENTS.md` describe the key.

## Capabilities

### Modified Capabilities

- `sequence-check`: alternative token lists.
- `vale-differences`: a rule file that only hakkiri reads.

## Impact

- New: `tests/sequence_forms.rs` and `tests/fixtures/sequence-forms`, with the three forms of `ThingsSpeak` and the one-list rules they must equal.
- Changed: `src/rule.rs`, `src/rule/schema.rs`, `src/rule/build.rs`, `src/sequence.rs`, `src/instructions.md`, the skill and `AGENTS.md`.
- `hakkiri::rule::Sequence` has `forms` and `exceptions`.  
  The new `Form` has the fields `Sequence` had: `tokens`, `ends`, `anchor` and `candidates`.
- No new dependency.

[ledger]: ../../../../tasks.yaml
