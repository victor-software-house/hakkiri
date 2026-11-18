# Design

## Context

A `Rule` is built once, when its style loads, from the text of its file.  
A parameter changes a field of that text, so the rule has to be read again with the field replaced.

`Rule::read` already does this for a rule that extends another.  
`inherit::merged` lays the child's fields over the parent's.  
It writes the result as JSON text that the same reader takes.

## Goals / Non-Goals

Goals:

- A glob sets a scalar parameter by rule name.  
  The alerts equal Vale's wherever a parameter applies the same way in both.
- A fault is placed on the config's own line, and a bad parameter never passes in silence.

Non-goals:

- Reading `.vale.ini`: row `HKR-035`.  
  There, the ini form `Style.Rule[param] = value` maps to a `params` entry.
- A parameter whose value is a list or a mapping.  
  Vale's ini stores only strings, and the fields it rejects are the ones that contain lists and mappings.

## Decisions

### 1. The key is `params`, a mapping under the glob

A rule key already stores a level, and every rule name contains a dot.  
So a key called `params` cannot be a rule.

Alternative: the Vale spelling `Style.Rule[param]` as a YAML key.  
It lost because YAML can nest the keys directly.  
A key with brackets would need its own parser and its own faults.

### 2. Parameters apply per glob

Vale reads a parameter from any section into one table keyed by the rule's name.  
It compiles each rule once with that table.  
The probe at `66bbb85` showed two flaws of that:

- A `[*.md]` section with `Vale.Repetition[max] = 1` changed the alerts of a `.rs` file.
- Two sections with `max = 1` and `max = 5` gave `max = 5` to both.

Vale keeps a level per section for this reason, as a comment in [ini.go][v-ini-levels] explains.  
When every level went to one map, the last section set the level everywhere.  
A parameter has the same fault.

hakkiri applies the globs that match a file in order, as it applies levels.  
A later glob wins for each parameter.

Alternative: one value per rule, as Vale does.  
It lost because a setting written under one glob must not change another glob's files.

A rule that a file runs with a set of parameters is a rule read with that set.  
The loader reads each rule once for every set that the matching globs can add up to.  
`Project::plan` picks the one for the file by the set's text.  
Equal sets from different globs share one read.

The loader finds the sets by following each glob in order, with and without it matching.  
A config with k globs that set parameters of one rule has at most 2^k sets.  
A real config has two or three.

### 3. An empty `styles` list drops the parameters

An empty list drops every rule setting before it.  
A parameter is a rule setting, so it goes too.  
Vale keeps it, since a parameter has no scope there.

### 4. The parameters are laid over the rule's fields, and the rule's reader refuses them

`Rule::with_params` writes the parameters over the merged fields, and reads the result as any rule file.  
So each check accepts the fields it defines and nothing else, with the value types the rule file allows.  
A number in quotes reads as a number, and `metrics: Gunning Fog` reads as a list of one.

A flag is `true` or `false`.  
Vale's ini spells it `YES` or `NO`.  
Vale's decoder also takes `1`, `t` and the other words that Go's `ParseBool` reads.

Alternative: translate those words.  
It lost because a YAML config has real booleans, and the message shows the value and the line.

The reader rejects some fields before it reads anything, as Vale rejects them.  
They are `level` and `extends`, `name`, `path`, `tests`, `message`, `description`, `link`, `tokens`, `swap`, `exceptions`, `filters`, `ignore`, `raw` and `either`.  
Each one contains a list, a mapping, the rule's identity or its prose.  
Or it is the level, which the rule's own setting carries.

### 5. A rule built from a vocabulary takes no parameter

Vale builds `Terms` and `Avoid` from a copy of a default definition, and reads no parameter into it.  
So a parameter for them changes nothing.  
hakkiri rejects it, since a setting that does nothing hides a typo.

A rule that `Vale.Repetition` stands for is read from its definition as text.  
So it takes parameters like any rule file.

### 6. `Vale.Repetition` is read from text

[Decision 1 of `vale-repetition`][vale-repetition-1] built the rule in code and left out an embedded rule file.  
It did so because the vocabulary rules were built in code.  
Parameters change that.

The rule's definition is Vale's own: `extends: repetition` with its defaults.  
A parameter replaces one of its fields.  
When hakkiri builds the rule from that text, the parameters of `Vale.Repetition` are those of any `repetition` rule.  
There is no second path.  
The alerts under the five configs of `vale-repetition` do not change.

### 7. Every fault is reported at once

hakkiri reads a config's parameters.  
When a rule rejects a set, hakkiri reads the set again one parameter at a time.  
So when two parameters of one rule are at fault, both faults show.

Each fault is placed on the line of the parameter it is about.  
Some faults are about no single parameter, such as a pattern that no longer compiles.  
Those are placed on the first parameter of the set.

[v-ini-levels]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/ini.go#L176-L185
[vale-repetition-1]: ../2026-11-14-vale-repetition/design.md#1-the-rule-is-built-in-code-like-terms-and-avoid
