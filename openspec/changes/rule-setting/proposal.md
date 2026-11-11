# Proposal

Queue row: `HKR-033` in [tasks.yaml][ledger].

## Motivation

A glob of `files` can set a rule by name, such as `Team.Terms: warning`.
hakkiri applies the setting only to a rule of a style the glob lists, so a setting for a rule of an unlisted style changes nothing.
Vale runs that rule instead: `Team.Terms = YES` or `Team.Terms = error` under a section that does not list `Team` turns the rule on, at the level it names ([Vale's section merge][v-merge]).
A project that moved from Vale to hakkiri loses every rule it turned on that way.
The operator decided hakkiri does as Vale does unless Vale is wrong, and this is no defect of Vale (operator, 2026-11-10).

## Changes

- A key that names a rule and gives a level runs that rule on the matched files at that level, whichever style holds it.
- `off` on a rule of an unlisted style changes nothing, and `off` on a rule of a listed style still stops it.
- A later glob wins for a rule, and a later `styles` list replaces the earlier list and keeps the rule settings.
- An empty `styles` list drops every rule setting before it, as Vale's empty `BasedOnStyles` does, and the settings of the same glob apply after it.
- A name clash or a bracket in the style of a rule that a setting enables is not refused, since no glob lists the style, as in Vale.
- A style with two files of one name runs the later file's rule, as Vale registers it, where hakkiri ran the earlier one.
- The `vale-differences` spec drops the gap and records what stays different: Vale's `YES` has no spelling in hakkiri, and hakkiri refuses a key that names no rule or a word that is no level.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `project-config`: what a rule setting enables, and how rule settings and style lists combine.
- `vale-differences`: the closed gap and the differences that remain.

## Impact

- `plan` in `src/config.rs` and `read_style` in `src/rule.rs` change. `check` and `test` both take their rules from `plan`. No new dependency.

[ledger]: ../../../tasks.yaml
[v-merge]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/file.go#L185-L211
