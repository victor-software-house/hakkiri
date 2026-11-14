# Design

## Context

A `Rule` is built once, when its style loads, from the text of its file.
A parameter changes a field of that text, so the rule has to be read again with the field replaced.
`Rule::read` already does this for a rule that extends another: `inherit::merged` lays the child's fields over the parent's and writes the result as JSON text that the same reader takes.

## Goals / Non-Goals

Goals:

- A glob sets a scalar parameter by rule name, and the alerts equal Vale's wherever a parameter applies the same way in both.
- A fault is placed on the config's own line, and a bad parameter never passes in silence.

Non-goals:

- Reading `.vale.ini`: row `HKR-035`. The ini form `Style.Rule[param] = value` maps to a `params` entry there.
- A parameter that holds a list or a mapping.
  Vale's ini holds only strings, and the fields it refuses are the ones that hold lists and mappings.

## Decisions

### 1. The key is `params`, a mapping under the glob

A rule key already holds a level, and every rule name holds a dot, so a key named `params` cannot be a rule.
Alternative: the Vale spelling `Style.Rule[param]` as a YAML key.
It lost because YAML holds the nesting directly, and a key with brackets would need its own parser and its own faults.

### 2. Parameters apply per glob

Vale reads a parameter from any section into one table keyed by the rule's name and compiles each rule once with it.
The probe at `66bbb85` showed two flaws of that.
A `[*.md]` section with `Vale.Repetition[max] = 1` changed the alerts of a `.rs` file, and two sections with `max = 1` and `max = 5` gave `max = 5` to both.
Vale keeps a level per section for this reason, noted in a comment of [ini.go][v-ini-levels]: writing every level to one map let the last section decide the level everywhere.
A parameter has the same fault.
hakkiri applies the globs that match a file in order, a later one winning for each parameter, as it applies levels.
Alternative: one value per rule, as Vale does.
It lost because a setting written under one glob must not change another glob's files.

A rule that a file runs with a set of parameters is a rule read with that set.
The loader reads each rule once for every set that the matching globs can add up to, and `Project::plan` picks the one for the file by the set's text.
Equal sets from different globs share one read.
The sets are found by following each glob in order, with and without it matching.
A config with k globs that set parameters of one rule has at most 2^k sets, and a real one has two or three.

### 3. An empty `styles` list drops the parameters

An empty list drops every rule setting before it, and a parameter is a rule setting, so it goes too.
Vale keeps it, since a parameter is not scoped there.

### 4. The parameters are laid over the rule's fields, and the rule's reader refuses them

`Rule::with_params` writes the parameters over the merged fields and reads the result as any rule file.
So each check accepts the fields it defines and nothing else, with the value types the rule file allows.
A number in quotes reads as a number, and `metrics: Gunning Fog` reads as a list of one.
A flag is `true` or `false`.
Vale's ini spells it `YES` or `NO`, and Vale's decoder also takes `1`, `t` and the other words Go's `ParseBool` reads.
Alternative: translate those words.
It lost because a YAML config has real booleans, and the message names the value and the line.

The fields `level` and `extends`, `name`, `path`, `tests`, `message`, `description`, `link`, `tokens`, `swap`, `exceptions`, `filters`, `ignore`, `raw` and `either` are refused before anything is read, as Vale refuses them.
Every one holds a list, a mapping, the rule's identity or its prose, or its level, which the rule's own setting carries.

### 5. A rule built from a vocabulary takes no parameter

Vale builds `Terms` and `Avoid` from a copy of a default definition and reads no parameter into it, so a parameter for them changes nothing.
hakkiri refuses it, since a setting that does nothing hides a typo.
A rule that `Vale.Repetition` stands for is read from its definition as text, so it takes parameters like any rule file.

### 6. `Vale.Repetition` is read from text

[Decision 1 of `vale-repetition`][vale-repetition-1] built the rule in code and left an embedded rule file out, since the vocabulary rules were built in code.
Parameters change that.
The rule's definition is Vale's own, `extends: repetition` with its defaults, and a parameter replaces one of its fields.
Building it from that text makes the parameters of `Vale.Repetition` the parameters of any `repetition` rule, with no second path.
The alerts under the five configs of `vale-repetition` do not change.

### 7. Every fault is reported at once

A config's parameters are read, and a set that a rule refuses is read again one parameter at a time, so the faults of two parameters of one rule both show.
A fault is placed on the line of the parameter it names.
A fault that no parameter names, such as a pattern that no longer compiles, is placed on the first parameter of the set.

[v-ini-levels]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/ini.go#L176-L185
[vale-repetition-1]: ../2026-11-14-vale-repetition/design.md#1-the-rule-is-built-in-code-like-terms-and-avoid
