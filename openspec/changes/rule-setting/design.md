# Design

## Context

Vale builds the rules of a file from its sections in the order written.
Each section may replace the list of styles, and may set rules on or off and give a level ([section merge][v-merge]).
A rule runs when a listed style holds it and no section turned it off, or when a section turned it on.
hakkiri keeps one map from rule name to the last choice among the matching globs, and reads it only for rules of listed styles ([plan][plan]).

The probes ran Vale `66bbb85` in a throwaway Linux box, over a style `Team` with `Terms` (suggestion, flags `foo`) and `Other` (warning, flags `bar`), and a style `Extra` that the section lists.

## Decisions

### 1. A rule setting with a level runs the rule

`Team.Terms = YES` and `Team.Terms = error` under `BasedOnStyles = Extra` ran `Team.Terms`, at suggestion and at error.
`Team.Terms = YES` alone, with no `BasedOnStyles`, ran only that rule.
hakkiri runs a rule whose name a matching glob gives a level, whichever style holds it, at that level.
The word `on` is `YES`: it runs the rule at the level of its rule file (operator, 2026-11-11).
The reader gives a bare `on` as a string, as it gives `off`, in a flow and in a block map, so the setting needs no quotes.

Alternative: keep running only the rules of listed styles.
Lost: it is the gap this change closes, and a project that turned on one rule of a style would lose it.

### 2. `off` stops a listed rule and changes nothing otherwise

`Team.Terms = NO` stopped the rule under `BasedOnStyles = Team`, and under `BasedOnStyles = Extra` it changed nothing.
hakkiri does the same, so `off` is a stop and never an enable.

### 3. Globs combine in the order written, as sections do

A later section won for the same rule: `NO` then `YES` ran it, `YES` then `NO` stopped it, and `warning` then `error` gave error on the file the later section matched.
The order of keys inside one section did not matter.
Vale keeps a level in its own map: `NO` turns a rule off without dropping the level, so `YES` after `error` and `NO` ran the rule at error.
hakkiri keeps a switch and a level apart for the same reason.
A rule turned on in a section that lists `Extra` kept running when a later section listed `Extra` alone, so a `styles` list replaces the list and leaves the rule settings.
hakkiri keeps its single map, applied in glob order, and `styles` keeps replacing the list.

Alternative: clear the rule settings at every `styles` list.
Lost: Vale keeps them for a non-empty list.

### 4. An empty `styles` list drops the earlier rule settings

`BasedOnStyles =` in a later section ran nothing on the file, though an earlier section had turned `Team.Terms` on, and the rule settings of the same section applied after it ([the reset][v-merge]).
hakkiri clears the map at an empty `styles` list, then applies that glob's own rule settings.
Without this the existing rule that an empty list turns linting off would fail for a file with an enabled rule.

Alternative: leave the map alone at an empty list.
Lost: the enabled rule would still run, although an empty list turns linting off.

### 5. Two files with one name run the later one

A style with `Terms.yml` and `Terms.custom.yml`, enabled by `Team.Terms = YES` and not listed, ran one rule, the message of `Terms.yml`, the later file in the walk.
`read_style` kept the earlier file and dropped the later one, so `Terms.custom.yml` ran.
It now lets the later file replace the earlier rule, so `Terms.yml` runs as in Vale, and it still records the clash.
`rule-loading` already refuses the clash in a listed style, so only a style enabled by a setting reaches this.

Alternative: keep the earlier file.
Lost: a rule of a clashing style would run another file than Vale runs.

### 6. The clash and bracket refusals stay with listed styles

No clash or bracket error came from a style enabled only by a rule setting, so the refusal still looks at the styles a glob lists and nothing else ([rule-names 3][rule-names-3]).
A test pins it.

### 7. What stays different

- Vale writes `YES` and `NO`, and hakkiri writes `on` and `off`.
- Vale ignores a key that names no rule, such as `Team.Nope = YES` or a rule of a style no package holds.
  hakkiri refuses it with its line, which `project-config` already requires.
- Vale reads a word that is none of its words, such as `yes`, with a warning and switches the rule off.
  hakkiri refuses a word that is neither a level, `on` nor `off`.

These differences go in `vale-differences`.

[v-merge]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/file.go#L185-L211
[plan]: https://github.com/victor-software-house/hakkiri/blob/bf04395ce723dbdcc2536b0fe099746f3d90d6ea/src/config.rs#L234
[rule-names-3]: ../archive/2026-11-10-rule-names/design.md#3-two-files-with-one-name-are-refused
