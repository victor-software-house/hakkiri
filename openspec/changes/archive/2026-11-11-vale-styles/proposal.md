# Proposal

Queue row: `HKR-007` in [tasks.yaml][ledger].

## Motivation

Vale styles are plain YAML files, and hakkiri keeps Vale's field names, so a Vale style should load as it is.
Two things stop it.  
A rule may extend another rule, as Vale 3.26.0 allows: `Direct.Length` in Voices extends `Std.Readability.SentenceLength`, and three GenZ rules extend a shared fragment.
hakkiri reads `extends` as a check type only, so a style that holds such a rule fails to load.  
A `script` rule makes hakkiri refuse its whole style, so `Readability`, which holds one script rule beside seven others, cannot load.  
The row first named a converter, `hakkiri import vale`, for the rule files.
Measured against the pinned Std, Voices and Readability styles, the converter has nothing else to do.
So the operator decided hakkiri reads a Vale style as it is, and keeps the name `hakkiri import vale` for the `.vale.ini` conversion (operator, 2026-11-11).

## Changes

- A rule whose `extends` holds a dot starts from the rule it names and lays its own fields on top, as Vale does: `key+` adds, `key-` removes, and the parent's `tests` do not carry over.
  The parent is found in any loaded package, in a nested folder or in a `_` folder.
- A chain Vale refuses is refused the same way.
  Those are a key written both bare and with `+` or `-`, an edit of a field the parent lacks, a cycle, and a chain over ten rules.
  They are also a parent that is missing, and a `Vale.` parent.
- A rule that is valid Vale and that hakkiri cannot run, a `script` rule or a rule with a `doc()` scope, no longer stops its style from loading.
  The rest of the style runs.
- Each such rule in a style a glob lists, or that a level or `on` turns on, makes the run exit 2 with a `failed` line until the config sets it `off`. In a style no glob lists it is ignored.
- `hakkiri test` reports the cases of such a rule as not run.
- The `vale-differences` spec lists each Vale feature hakkiri does not run as a Known gap with a row: Tengo scripts, formulas past arithmetic, the built-in `Vale.Repetition`, `doc()` scopes and Views, and the `.vale.ini` import.
- `AGENTS.md` and the operator contract stop saying styles enter only through an importer.
- Three upstream styles are committed as fixtures with their licences: Voices, Std and Readability.
- The packages of the config, a fetched zip included, are all on the search path before the first style is read.
  A test loads ProseStyle v0.0.14 from the cache, and its six script rules fail the run until the config sets them `off`.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `rule-loading`: inheritance, and a rule hakkiri cannot run.
- `rule-files`: a script rule is reported, not refused.
- `project-config`: a rule hakkiri cannot run, and `off`.
- `test-command`: the cases of such a rule.
- `vale-differences`: the Known gaps and the departures.

## Impact

- `src/rule.rs`, a new `src/rule/inherit.rs`, `src/rule/schema.rs`, `src/config.rs`, `src/run.rs`, `src/suite.rs` and `src/report.rs` change.
  `read_style` takes the package folders to search, and `Style` and `LoadProblem` carry the rules hakkiri cannot run.
  No new dependency.

[ledger]: ../../../../tasks.yaml
