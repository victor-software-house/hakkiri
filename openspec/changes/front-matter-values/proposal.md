# Proposal

Queue row: `HKR-049` in [tasks.yaml][ledger].
It follows [math as code][math] in the stack, and uses nothing from it.

## Motivation

Vale lints each top-level string value of a Markdown file's front matter, as the scope `text.frontmatter.<key>`.
hakkiri skipped the whole block, so a text rule never saw a `title` or a `description`.
The probe in `tests/fixtures/markup-differences` showed it, and the row `HKR-049` queued it with the question of which keys Vale lints.

## Changes

- Each top-level string value of YAML, TOML and JSON front matter is a node of the scope `frontmatter`, read by `text` rules and by rules scoped `frontmatter`, `frontmatter.<key>`, `text.frontmatter` and `text.frontmatter.<key>`.
- A key, a number, a boolean, a null, a date, a list and a nested mapping hold no value that is read, as in Vale.
- A block or token pattern leaves the front matter as written, as in Vale, which reads the metadata before it rewrites the body.
- A block that no parser reads fails the file with the line of the problem and exit 2, and the rest of the file is linted, where Vale lints none of it.
- The alerts sit at the text of the value, which Vale misplaces in four cases that the `vale-differences` spec records.

## Capabilities

### New Capabilities

- `front-matter`: which values are read, what scope they have, and where an alert lies.

### Modified Capabilities

- `vale-differences`: the four places where Vale puts an alert at the wrong place, and the two files it fails and hakkiri lints.

## Impact

- New: `src/frontmatter.rs`, `tests/front_matter.rs` and its fixtures, with Vale's alerts recorded from Vale at `66bbb85`.
- Changed: `Cargo.toml` and `Cargo.lock`, the tree (a scope and a label), the scope names, the Markdown builder and the ignore rewrite, `src/instructions.md`, `README.md` and `AGENTS.md`.
- Dependency: ctl-core moves to 0.7.4 and its `input-toml` feature is on. hakkiri already uses ctl-core's `input` feature, so no new crate of its own, and the binary grows by the size recorded in the tasks.

[ledger]: ../../../tasks.yaml
[math]: ../math-code/proposal.md
