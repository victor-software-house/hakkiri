# Proposal

Queue row: `HKR-031` in [tasks.yaml][ledger].
It follows [block-token-ignores][block-token-ignores] in the stack, and uses nothing from it.

## Motivation

Vale reads HTML comments such as `<!-- vale Style.Rule = NO -->` and turns rules off for the text after them.
The consumers use the per-rule form: 58 tags in one private repository, 4 in nishiki, 2 in hakkiri and 2 in prose-style.
hakkiri read no comment, so a project that moves from Vale lints text it had turned off.
The [vale-differences spec][differences] lists the missing controls as a known gap.

## Changes

- A Markdown file's HTML comments turn rules off and on with Vale's reach: `vale off`, `vale on`, `vale Style.Rule = NO` and `YES`, `vale Style = NO`, the matches of a rule, and `vale styles = …`.
- hakkiri writes the same controls in the words of its config: `<!-- hakkiri off -->`, `<!-- hakkiri on -->` and `<!-- hakkiri Style.Rule: off -->` or `on`.
- A comment `<!-- hakkiri Style.Rule: suggestion -->`, `warning` or `error` sets the level of that rule's alerts from there on.
- The instructions and the README show the hakkiri form first (operator, 2026-11-11).
- The `vale-differences` spec drops the gap, and records the forms hakkiri adds and what it leaves unread.

## Capabilities

### New Capabilities

- `comment-controls`: the comments that turn rules off and on, and where each reaches.
- `comment-options`: the comments that turn off a match, limit the styles or set a level, and the files that read none.

### Modified Capabilities

- `vale-differences`: the gap for the comment controls closes, and the differences that remain are listed.

## Impact

- New: `src/control.rs`, `tests/comment_controls.rs` and its fixtures, with Vale's alerts recorded from Vale at `66bbb85`.
- Changed: the Markdown builder, the HTML tokenizer, the tree, the engine, `src/instructions.md`, the skill, `README.md` and `AGENTS.md`.
- No new dependency.

[ledger]: ../../../../tasks.yaml
[block-token-ignores]: ../2026-11-12-block-token-ignores/proposal.md
[differences]: ../../../specs/vale-differences/spec.md
