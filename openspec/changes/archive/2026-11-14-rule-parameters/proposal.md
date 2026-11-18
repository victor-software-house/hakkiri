# Proposal

Queue row: `HKR-059` in [tasks.yaml][ledger].
It follows [HKR-038][vale-repetition], which built `Vale.Repetition` and left its parameters out.

## Motivation

Vale sets a scalar field of a rule from the config, with a key written `Style.Rule[param] = value` ([ini.go][v-ini]).
It applies the value when the rule compiles ([manager.go][v-manager]).
A package such as ProseStyle, and the projects that use it, set a threshold, a case flag or a scope this way.
They do not have to copy the rule.

hakkiri's config reads a level, `on` and `off` by rule name, and no parameter.
So a project that moves from Vale loses the setting, and the alerts differ from Vale's.

The ledger row spells the key `Style.Rule.param`.
Vale 3.26.0 at `66bbb85` reads `Style.Rule[param]`, and takes `Style.Rule.param` for a rule name.
So this change corrects the form.

## Changes

- A glob of `files` takes `params`.
  It maps a rule name to a mapping from a parameter name to one value: a number, `true` or `false`, or text.
  Each parameter replaces the field of the same name in the rule's file, or in `Vale.Repetition`.
  The rule is then read again with it.
- A parameter reaches the files of the glob that sets it.
  The more specific glob wins for the same parameter, as in the settings cascade of row `HKR-069`.
  An empty `styles` list drops the parameters before it.
  Vale applies a parameter to every file, whichever section sets it, and the last section wins everywhere.
- hakkiri rejects a parameter that Vale rejects, with the config file and the line of the parameter.
  These are a level, and the fields `extends`, `name`, `path`, `tests`, `message`, `description`, `link`, `tokens`, `swap`, `exceptions`, `filters`, `ignore`, `raw` and `either`.
- Vale accepts some parameters without a word, and fails on others without a place.
  hakkiri rejects them all with the file and line.
  This covers a rule that does not exist, a field its check does not define, and a value of the wrong kind.
  It also covers a negative `max`, an empty value, and a `scope` that is none.
- A rule that `Vale.Terms` or `Vale.Avoid` stands for takes no parameter, since a vocabulary builds it.
- Every fault of a config is reported at once.
- `Vale.Repetition` is read from its Vale definition, as text.
  So it takes parameters through the path that any rule file takes.
- The `vale-differences` spec records each place where hakkiri differs.
- The same spec records where a `consistency` alert goes in a paragraph of several lines.
  hakkiri puts it on the last match, and Vale puts it on the first line that contains the text.
  A fixture pins both positions.

## Capabilities

### New Capabilities

- `rule-parameters`: how a config sets the parameters of a rule, where they apply, and what is rejected.

### Modified Capabilities

- `vale-differences`: one requirement lists the parameter differences.
  Another records where a `consistency` alert goes in a paragraph of lines.

## Impact

- New: `tests/rule_parameters.rs` and `tests/fixtures/rule-parameters`, with the alerts and errors of Vale at `66bbb85`.
  Also new: `tests/fixtures/consistency-lines`, with the row Vale gave for the placement.
- Changed: `src/config.rs`, `src/rule.rs`, `src/rule/inherit.rs`, `src/rule/build.rs`, `src/instructions.md`, the skill, `README.md` and `AGENTS.md`.
- `hakkiri::rule` gains `Param` and `Rule::with_params`.
  It loses `vale_repetition_rule`, which was crate-private.
- No new dependency.

[ledger]: ../../../../tasks.yaml
[v-ini]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/ini.go#L196-L225
[v-manager]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/manager.go#L380-L386
[vale-repetition]: ../2026-11-14-vale-repetition/proposal.md
