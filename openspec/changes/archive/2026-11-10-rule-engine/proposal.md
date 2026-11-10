# Proposal

Queue row: `HKR-004` in [tasks.yaml][ledger].

## Motivation

The tree slice gives every file a tree of scoped nodes with byte segments ([document-tree][document-tree], [node-segments][node-segments]).  
Nothing reads it yet.  
The CLI slice, `HKR-005`, needs an engine that loads rules, selects text by scope, runs checks and returns alerts that point into the file.

Vale's eleven check types are too much for one change under the spec budget.  
This change builds the engine and the four check types ProseStyle uses today: `existence`, `substitution`, `occurrence` and `metric`.  
Two later changes add the rest: [`pattern-and-readability-checks`][next] and [`case-and-sequence-checks`][last].

## Changes

- Loading: a style folder gives one rule per `.yml` file.  
  A rule with a missing, unknown or invalid field is refused with an error that names it.
- Selection: scope names match nodes by their dotted prefix; `text`, `sentence` and file extensions get Vale's meaning; an inline exclusion blanks the text it names.
- Text: a check reads the text of a node with inline markup left out, and each code span as one word.  
  Sentences come from hakkiri's own splitter, and words from one word pattern.
- Patterns: the `regex` crate, and `fancy-regex` with a backtrack limit for a pattern that needs lookaround; a limit reached is an error alert.
- Alerts: rule, level, message, line, column span and matched text, sorted and deduplicated as Vale does.
- Checks: `existence`, `substitution`, `occurrence` and `metric`, with every metric variable Vale defines plus `occurrences`.
- The tree: inline markup becomes a gap between segments, and a Markdown tree counts its code blocks.

## Capabilities

### New Capabilities

- `rule-loading`: how style folders and rule files load, and which rules are refused.
- `scope-selection`: which text a scope selects, and the text, sentences and words a check reads.
- `alerts`: what an alert holds, where it points, how its message is filled, and how pattern failures report.
- `match-checks`: the `existence`, `substitution` and `occurrence` checks.
- `metric-check`: the `metric` check, its variables, its formula and its condition.

### Modified Capabilities

- `node-segments`: inline markup becomes a gap, so a link reads as its text.
- `document-tree`: a Markdown or MDX tree counts its code blocks, for the metric variable `pre`.
- `rule-files`: the semicolon-rate scenario gains `nonword: true`, so its token matches as an `existence` token does.

## Impact

- New dependencies: `regex` and `fancy-regex`, chosen in the [bootstrap design][bootstrap], and `ctl-core` for reading rule files ([design][design]).
- Sentences come from the [`sentence-splitter`][splitter] change, which is archived as the [`sentence-splitting`][sentence-splitting] spec.
- New modules for rules, selection, text, patterns, alerts and the four checks; the tree builders and their snapshots change for inline gaps.
- `HKR-005` builds the `check` and `test` commands on this engine.

[ledger]: ../../../../tasks.yaml
[document-tree]: ../../../specs/document-tree/spec.md
[node-segments]: ../../../specs/node-segments/spec.md
[next]: ../2026-11-10-pattern-and-readability-checks/proposal.md
[last]: ../2026-11-10-case-and-sequence-checks/proposal.md
[bootstrap]: ../2026-11-09-bootstrap-hakkiri/design.md
[design]: design.md
[splitter]: ../2026-11-10-sentence-splitter/proposal.md
[sentence-splitting]: ../../../specs/sentence-splitting/spec.md
