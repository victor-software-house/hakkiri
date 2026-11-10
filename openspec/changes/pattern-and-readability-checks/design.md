# Design

## Context

The [`rule-engine` change][engine-design] loads rules, selects nodes and builds each node's check text with its byte table.  
It also cuts sentences and words, compiles patterns, places alerts and computes every metric variable, the five grade scores among them.

This change adds four check types on top of that engine.  
It adds no state to the engine, only what each check keeps per file.

Each Vale behaviour below was read in Vale's source at `66bbb85`, which is 3.26.0 plus 16 commits.

## Goals / Non-Goals

**Goals:**

- Give `consistency`, `conditional`, `repetition` and `readability` the meaning Vale 3.26.0 gives them, except where a decision below says otherwise.
- Keep the per-file state of a check inside that check, so two rules never share it.

**Non-Goals:**

- Views and the `in` field of `conditional`: hakkiri has no Views ([rule-engine design][engine-design]).
- Vocabularies: `vocab` loads with no effect, as in the rule-engine change.
- Fixes: `action` loads and rides along on the alert.

## Decisions

### 1. Per-file state belongs to one rule

`consistency` records which form of each pair it has seen, and `conditional` records the terms `second` captured.  
Each rule keeps that state for one file and drops it when the file ends.

Alternative: Vale's single list per file, which every `consistency` and `conditional` rule appends to ([consistency.go][v-cons-run], [conditional.go][v-cond-capture]).  
Lost: a term one `conditional` rule captures could then satisfy another rule.  
A captured word that equals a `consistency` group name, such as `Spelling2`, would also count as that form.

### 2. consistency matches as existence does, one pattern per pair

For each `either` pair, hakkiri builds one pattern with the key and the value as two alternatives inside Vale's word template.  
So `ignorecase` and `nonword` mean what they mean for `existence`, and a space in a form matches a line wrap, as in an `existence` token ([consistency.go][v-cons-new]).

A node gives one alert for a pair when the pair matched in it and both forms have matched in the file so far.  
The alert sits at the last match of the pair in that node, through the byte table.

Alternatives:

- Vale's placement.  
  Vale keeps the span of the last match, then searches for its text, so the alert lands where that text first appears ([consistency.go][v-cons-run]).  
  Lost: the rule-engine change never places an alert by searching for its text.
- Vale's weak decoding, which accepts unknown fields and converts types for this check only ([consistency.go][v-cons-new]).  
  Lost: [rule-loading][loading] refuses an unknown field for every check type.

### 3. conditional has two modes, chosen by the second pattern

`first` and `second` compile as written, with no word template, as Vale compiles them ([conditional.go][v-cond-new]).  
Group detection follows Vale's: a `(` that is not escaped, not inside a class and not `(?`, or a named group ([conditional.go][v-cond-group]).

A `first` or `second` that is empty or matches the empty string is refused.  
An empty `first` would alert at every position, and an empty `second` would stop every alert.  
A space is a pattern.  
A zero-width match such as `\b` still gives an alert, as it does for `existence`.

When `second` has no capture group, a node passes if `second` matches anywhere in it.  
Otherwise each match of `first` in the node is an alert ([conditional.go][v-cond-plain]).

When `second` has a group, the node's captures join the rule's terms first.  
Then each match of `first` is an alert, unless it is a term, an `exceptions` match or inside an exception phrase ([conditional.go][v-cond-capture]).  
Terms grow in document order, so a use before its definition gives an alert.

Alternative: read the whole file for definitions before checking any use.  
Lost: it differs from Vale, and a style that wants a term defined before use would no longer catch the early use.

### 4. repetition compares each match with the one before

The rule's `tokens` join into one pattern, with `(?i)` first when `ignorecase` is set, and no word template ([repetition.go][v-rep-new]).  
Within one selected node, each match is trimmed of spaces and compared with the one before.

An equal pair raises the count, and a count over `max` gives an alert that spans both matches.  
There is no alert when the span holds more than one sentence or matches an exception ([repetition.go][v-rep-run]).  
The count goes back to 0 only after an alert, as Vale counts it.

Alternative: reset the count on every match that does not repeat.  
Lost: a rule with `max: 1` would then give fewer alerts than Vale on the same text, a difference the [format design][format-7] does not list.

### 5. readability reads the metric scores

`readability` computes each score its `metrics` list names, from the same counts a `metric` rule reads ([readability.go][v-read-run]).  
It averages them and reports when the average is greater than `grade`.  
The comparison uses the unrounded average, and the message prints it with two decimals.

With no scope, or `scope: text`, the rule measures the prose of the whole file, as a `metric` rule does.  
Its alert then sits at line 1, column 1 ([ast.go][v-summary], [file.go][v-locate]).  
Text with no sentence gives no alert: it has no words either, and each score divides by one of those counts.

Alternative: compute the scores in the check itself.  
Lost: two copies of each formula could drift, and the Flesch-Kincaid parity test in the rule-engine change would cover only one.

### 6. An unknown or empty metrics list is refused

The [readability spec][readability] refuses a `metrics` list that is empty or names a score outside the five.  
hakkiri gives a better result than Vale where Vale's is a defect (operator, 2026-11-10).

Alternative: Vale's behaviour.  
Vale adds 0 to the sum for an unknown name, but still counts it in the divisor ([readability.go][v-read-new]).  
A rule with no scope then measures each `text` node instead of the whole file, and an empty list divides by 0.

Lost: a misspelt score name lowers the grade with no sign of why, measured on other text than the author asked for.

### 7. ignorecase on a conditional applies to both patterns

When `ignorecase` is `true`, a `conditional` rule matches `first` and `second` without regard to case, and compares a match of `first` with a captured term the same way.  
Vale decodes the field and nothing reads it, so both patterns stay case-sensitive ([conditional.go][v-cond-fields]).  
The [pattern-checks spec][pattern] states the behaviour, by the same rule as decision 6 (operator, 2026-11-10).

Alternatives:

- Load it with no effect, as Vale does. Lost: an author who sets the field gets no sign that it does nothing.
- Refuse the field. Lost: a style written for Vale that sets it would stop loading.

### 8. A score listed twice counts once

Vale adds a listed score once however often it is listed, and divides by the length of the list ([readability.go][v-read-run]).  
hakkiri averages the distinct scores, so a repeated name changes nothing: it gives a better result than Vale where Vale's is a defect, as in decision 6.

Alternative: Vale's division.  
Lost: `[SMOG, SMOG]` would report half of SMOG's grade.

### 9. Code is no match in the pattern checks

Vale masks code in the text a rule reads, so two adjacent code spans can read as one repeated word, and a form in code counts as seen.  
hakkiri treats a match that overlaps a code span as no match, as `existence` does.  
For `repetition`, the match after it has none before it to repeat.  
For `consistency`, it is not a form seen.  
For `conditional`, a match of `first` gives no alert and a match of `second` defines nothing.

Alternative: Vale's masked text.  
Lost: ``Use `a` `b` `c` `` would report a repeat of nothing the reader sees, and a spelling in a code sample would count against the prose.

## Risks / Trade-offs

- [hakkiri's splitter cuts some text apart from Vale's] → `repetition` reads sentences only through the engine.  
  Its scenarios use sentences that both splitters cut the same way.
- [Vale's repetition count is a quirk a style author may not expect] → the spec states it, and a scenario pins it.
- [A `consistency` alert at its own match can differ from Vale's column] → this happens only when the matched text appears earlier in the node.  
  The rule, level, message and matched text still agree.

[engine-design]: ../archive/2026-11-10-rule-engine/design.md
[loading]: ../../specs/rule-loading/spec.md
[readability]: specs/readability-check/spec.md
[pattern]: specs/pattern-checks/spec.md
[format-7]: ../archive/2026-11-10-rule-format/design.md#7-intended-differences-from-vale
[v-cons-new]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/consistency.go#L35-L85
[v-cons-run]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/consistency.go#L88-L126
[v-cond-fields]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/conditional.go#L14-L34
[v-cond-group]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/conditional.go#L36-L72
[v-cond-new]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/conditional.go#L101-L116
[v-cond-plain]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/conditional.go#L155-L166
[v-cond-capture]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/conditional.go#L168-L209
[v-rep-new]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/repetition.go#L48-L60
[v-rep-run]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/repetition.go#L66-L130
[v-read-new]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/readability.go#L21-L38
[v-read-run]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/readability.go#L41-L73
[v-summary]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/ast.go#L597-L617
[v-locate]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/file.go#L689-L716
