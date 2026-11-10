# Design

## Context

The [`vale-differences` design][first-design] sets how the list is built: one scenario for each difference, with a link to its decision, and only what is true now.  
Any behaviour that differs from Vale belongs in it, whether or not a design calls it a difference.  
This change adds the decisions that the [rule-format][format], [rule-engine][engine] and [bootstrap][bootstrap] designs record as choices, and that differ from Vale when checked.

## Goals / Non-Goals

**Goals:**

- Each decision of the format, engine and bootstrap designs that changes what a rule does, or what a project writes, listed as a scenario.

**Non-Goals:**

- New decisions: each scenario records what its design decided.
- Differences of the Markdown parser that the designs do not state and no probe has measured.

## Decisions

### 1. Each case is checked against Vale 66bbb85

A probe of Vale 66bbb85 built from source gave these results:

- A plain `//` comment holding a code span: Vale reported both the word inside the backticks, at column 9, and the plain word, at column 27. This build could not fetch the Rust grammar, so Vale read comments by their delimiters. The tree-sitter path was read in its source instead: with no markup format named, `lintComment` lints plain lines, and the default Rust query names none ([code_ts.go][v-comment], [rs.go][v-rust]).
- `scope: table.row` and `scope: nonsense`: Vale loaded both and reported nothing, where `table`, `table.cell` and `text` reported the word in a cell. Vale's block scopes have `table.header`, `table.cell` and `table.caption`, and no `row` ([ast.go][v-table]).
- A `metric` rule with `tokens`: Vale refused it as an invalid key. A formula that reads `occurrences`: Vale refused it as an unresolved reference.
- The formula `words > 1 ? 7.0 : 0.0`: Vale reported 7.00. Vale runs the formula as one Tengo expression ([metric.go][v-eval]).
- An `extends: script` rule: Vale loaded it ([script.go][v-script]). The refusal in hakkiri is from its spec.

Alternative: list these from the designs alone. Lost: no design states that Vale differs, so each scenario would repeat a claim no one had checked.

### 2. Configuration is listed with the behaviour

A project that moves from Vale writes new settings, so the three differences in configuration and packages sit in the same spec.

Alternative: leave configuration to the importer's documentation. Lost: the list holds any difference, and a project meets these at its first run.

## Risks / Trade-offs

- [A probe of one input is read as a rule] → each scenario states the case it measured and no more.
- [Decisions with no stated Vale behaviour stay unchecked] → a decision found later gets a probe before it joins the list.

[first-design]: ../vale-differences/design.md
[format]: ../archive/2026-11-10-rule-format/design.md
[engine]: ../archive/2026-11-10-rule-engine/design.md
[bootstrap]: ../archive/2026-11-09-bootstrap-hakkiri/design.md
[v-comment]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/code_ts.go#L199-L203
[v-rust]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/code/rs.go#L9-L19
[v-table]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/ast.go#L51-L58
[v-eval]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/metric.go#L165-L198
[v-script]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/script.go
