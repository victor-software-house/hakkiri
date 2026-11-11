# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.

## 1. Probe Vale

- [x] 1.1 Build Vale `66bbb85` in a throwaway Linux box, and run it over a project with the cases below.  
  The cases are `YES`, a level and `NO` for a rule of a style the section does not list, and a style listed in one section and enabled by rule in another.  
  They are also two sections that set one rule, and a later `BasedOnStyles` with a style and with none.  
  The last are a key that names no rule, and a clash and a bracket in a style enabled only by rule.  
  Verify the alerts, levels and exit status match the scenarios of the `project-config` delta, and record what differs in `vale-differences`.
  - 2026-11-11 (UTC), 26 probes ran on `Team` (`Terms` suggestion, `Other` warning) and `Extra`, with the severities read from Vale's JSON where a level mattered.  
    `YES` and a level ran `Team.Terms` under `BasedOnStyles = Extra`, `NO` stopped it only when `Team` was listed, and a later section won for one rule.
  - A later list of styles kept the rule that an earlier section turned on, and `BasedOnStyles =` dropped it, with the rule settings of its own section applying after.
  - A style with `Terms.yml` and `Terms.custom.yml` ran one rule with the message of `Terms.yml`, with exit 0.  
    A key that names no rule gave no error, and `yes` gave a warning and switched the rule off.  
    A key `Team.Weird[max]` ran nothing, and gave no error.

## 2. Run the rule

- [x] 2.1 Run a rule that a matching glob gives a level, whichever style holds it, and clear the rule settings at an empty `styles` list, in `plan` of `src/config.rs`.  
  Verify the `rule_setting` tests that enable a rule of an unlisted style, and the test of the empty list, fail on the old `plan` and pass on the new one.
  - 2026-11-11 (UTC), 8 of the 11 `rule_setting` tests failed on `config.rs` and `rule.rs` of [bf04395][c-base], in a throwaway Linux box.  
    The other 3 guard behaviour that did not change: a rule turned on after it was off, `off` on an unlisted rule, and the order of keys.  
    All 11 pass in [0b15215][c-code].
- [x] 2.2 Keep the later file's rule when two files of a style give one name, in `read_style` of `src/rule.rs`.  
  Verify the clash test of `rule_setting` fails on the old loader and passes on the new one, with one alert from `Terms.yml`.
  - 2026-11-11 (UTC), the clash test failed on the old loader with the message of `Terms.custom.yml`, and passes in [0b15215][c-code].
- [x] 2.3 Read `on` as a rule setting that runs the rule at the level of its rule file, in the reader and `plan` of `src/config.rs`.  
  Keep a set level when a later glob turns the rule off and on.  
  Verify the `on` tests of `rule_setting` fail on the `config.rs` of the previous commit and pass on the new one, with `on` written in a flow and in a block map.
  - 2026-11-11 (UTC), 4 of the 15 `rule_setting` tests failed on the previous `config.rs`.  
    They are the two `on` tests, the level that stays, and the word that is no level.  
    All 15 pass in [80ae211][c-on].  
    So the reader gives a bare `on` as a string in a flow and in a block map, as it gives `off`.
  - In a probe of `66bbb85`, `error`, then `NO`, then `YES` under another section ran `Team.Terms` at error.

## 3. Record

- [x] 3.1 Update the `project-config` spec through the delta, close the gap scenario and add the departures in `vale-differences`, and update `src/instructions.md`.  
  Verify `openspec validate --all --strict` and the budget check pass.
  - 2026-11-11 (UTC), both pass on the Mac, with 24 of 24 items valid.
- [x] 3.2 Start `HKR-033` with `qctl`.  
  Verify `qctl check` passes.
  - 2026-11-11 (UTC), `qctl check` passes with `HKR-033` active.

## 4. Gate

- [ ] 4.1 Pull request CI passes on the head of the pull request.  
  Verify the checks of the head commit pass in the run on the pull request.

[c-base]: https://github.com/victor-software-house/hakkiri/commit/bf04395ce723dbdcc2536b0fe099746f3d90d6ea
[c-code]: https://github.com/victor-software-house/hakkiri/commit/0b15215ba10aa004b10d1221b0b5c99f38de020c
[c-on]: https://github.com/victor-software-house/hakkiri/commit/80ae2110ce74a35ff000a6ebcd483f8349a11445
