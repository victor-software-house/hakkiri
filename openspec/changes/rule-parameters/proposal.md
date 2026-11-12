# Proposal

Queue row: `HKR-059` in [tasks.yaml][ledger].
It follows [HKR-038][vale-repetition], which built `Vale.Repetition` and left its parameters out.

## Motivation

Vale sets a scalar field of a rule from the config, with a key written `Style.Rule[param] = value` ([ini.go][v-ini]), and applies it when the rule compiles ([manager.go][v-manager]).
A package such as ProseStyle, and the projects that use it, set a threshold, a case flag or a scope this way without copying the rule.
hakkiri's config reads a level, `on` and `off` by rule name and no parameter, so a project that moves from Vale loses the setting, and the alerts differ from Vale's.

The ledger row spells the key `Style.Rule.param`.
Vale 3.26.0 at `66bbb85` reads `Style.Rule[param]` and takes `Style.Rule.param` for a rule name, so the form is corrected here.

## Changes

- A glob of `files` takes `params`: a mapping from a rule name to a mapping from a parameter name to one value, which is a number, `true` or `false`, or text.
  Each parameter replaces the field of the same name in the rule's file, or in `Vale.Repetition`, and the rule is read again with it.
- A parameter reaches the files of the glob that sets it, a later glob winning for the same parameter, and an empty `styles` list drops the parameters before it.
  Vale applies a parameter to every file, whichever section sets it, and the last section wins everywhere.
- A parameter that Vale refuses is refused, with the config file and the line of the parameter: a level, and the fields `extends`, `name`, `path`, `tests`, `message`, `description`, `link`, `tokens`, `swap`, `exceptions`, `filters`, `ignore`, `raw` and `either`.
- A parameter that Vale accepts without a word, or fails on without a place, is refused with the file and line.
  That covers a rule that does not exist, a field the rule's check does not define, a value of the wrong kind, a negative `max`, an empty value, and a `scope` that is none.
  A rule that `Vale.Terms` or `Vale.Avoid` names takes none, since a vocabulary builds it.
- Every fault of a config is reported at once.
- `Vale.Repetition` is read from its Vale definition, as text, so it takes parameters through the path that any rule file takes.
- The `vale-differences` spec records each place where hakkiri differs.

## Capabilities

### New Capabilities

- `rule-parameters`: how a config sets the parameters of a rule, where they apply, and what is refused.

### Modified Capabilities

- `vale-differences`: one requirement lists the differences.

## Impact

- New: `tests/rule_parameters.rs` and `tests/fixtures/rule-parameters`, with Vale's alerts and errors recorded from Vale at `66bbb85`.
- Changed: `src/config.rs`, `src/rule.rs`, `src/rule/inherit.rs`, `src/rule/build.rs`, `src/instructions.md`, the skill, `README.md` and `AGENTS.md`.
- `hakkiri::rule` gains `Param` and `Rule::with_params`, and loses `vale_repetition_rule`, which was crate-private.
- No new dependency.

[ledger]: ../../../tasks.yaml
[v-ini]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/ini.go#L196-L225
[v-manager]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/manager.go#L380-L386
[vale-repetition]: ../vale-repetition/proposal.md
