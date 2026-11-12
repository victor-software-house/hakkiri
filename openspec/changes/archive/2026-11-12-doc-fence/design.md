# Design

## Context

`comments::groups` joins adjacent line comments of one marker into a group, and each group is one `raw` root and one comment node.
It joined a line only when exactly one line break lay between two comments.
A blank line ended the group, so the `///` lines after it started a new text and a fence opened before the blank line was gone.

## Goals / Non-Goals

Goals:

- A fence in a doc comment stays open across a blank line, in every scope.
- The ProseStyle v0.0.14 `BareURLInComment` case passes.

Non-goals:

- Plain `//` comments: rustdoc has no reading for them, and a blank line ends their run as before.
- Doc comments separated by code: they belong to two items.

## Decisions

### 1. Doc line comments join across blank lines

Rustc turns each `///` or `//!` line into one `doc` attribute, so blank lines between them leave nothing, and rustdoc reads one text.
The group takes a following doc line when only white space separates the two comments, and any number of line breaks may lie between.
Vale joins a line only when its number follows the previous one and the offset and format match ([`doneMerging`][v-merge]).
A blank line ends a run there, so the `raw` scope in Vale sees the whole file for this case, and its text scopes read two comments.
Alternative: join only the `raw` roots and leave the Markdown nodes cut at the blank line.
It lost because a rule on `code` or `text` would still read the fenced lines as prose, and the bug is in the reading of the comment, not in one scope.
Alternative: join every line comment across blank lines.
It lost because a blank line between two plain comments ends a thought, and rustdoc gives no reason to read them as one.

## Risks

- A doc comment with a blank line between two paragraphs now reads as one paragraph, as rustdoc reads it.
  A rule on a paragraph or a sentence sees the joined text.

[v-merge]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/code/comments.go#L43-L58
