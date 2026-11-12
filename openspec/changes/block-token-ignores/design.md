# Design

## Context

Vale reads a Markdown file in three steps.
It rewrites the text with `applyBlockPatterns` and `applyInlinePatterns` ([html.go][v-apply]), converts the result to HTML, and lints the HTML while it finds each token in the file as written.
hakkiri has no HTML step: it parses the Markdown to a tree whose nodes hold byte ranges of the file, and every rule reads the file through those ranges.

## Goals / Non-Goals

Goals:

- A glob hides the text its patterns match, and the alerts match those of Vale on the probe files.
- Every alert is placed in the file as written.

Non-goals:

- Ignores in a file that is not Markdown or MDX, since hakkiri reads no other markup.
- `IgnorePatterns`, Vale's older name for the block patterns.

## Decisions

### 1. The text is rewritten, parsed, and mapped back

hakkiri applies Vale's own rewrite to the text, parses the result with the Markdown parser it already has, and then replaces every range in the tree by the bytes of the file it came from.
Each block match becomes the match's first group between `\n```\n` and `\n```\n`, and each token match the group between backticks ([html.go][v-apply]).
The patterns run in the order written, each over the text the one before it left, block patterns first.
A byte that no pattern added keeps its place in the file, and the group copied into the replacement keeps its place too, so a code span that holds an ignored token still points at the token.
A range with no byte of the file, such as the backticks, gives no segment, and a node that has no segment left gives way to its children.

Alternative: mask the match with spaces and leave the Markdown as it is.
It lost because a fence added around a match changes what the parser sees: a block in the middle of a paragraph splits it in two, and a code span is a node a `code` rule reads.
The probe shows both, so a mask would give different alerts.
Alternative: drop the nodes that overlap a match after parsing.
It lost for the same reason, and a match can start inside a node and end in another.

### 2. What a match turns into is what Vale gives

The probe of Vale at `66bbb85` (2026-11-11) found these:

- A pattern with no capture group leaves the text `$1`, since Vale hands the replacement `$1` to an engine that keeps it as written, so the whole match is dropped from the prose.
- A group that took no part in the match leaves nothing, and a match of no bytes becomes an empty pair of backticks everywhere in the text. hakkiri skips such a match, since nobody writes a pattern for it on purpose.
- Patterns apply to the result of the one before, so a token pattern that holds a block pattern's match no longer matches once the block is fenced.
- A `scope: raw` rule still reads the file as written, and a `scope: code` rule reads each ignored token, since it is a code span.
- Vale does not apply the patterns to a Rust, TypeScript or JavaScript file: it reads a comment as plain lines unless a View names a markup format ([lintComment][v-comment]).

### 3. A source file's comments take the patterns

Vale reads a comment of a source file as plain lines unless a View names a markup format, and applies no pattern to it ([lintComment][v-comment]).
So `TokenIgnores` on a Rust file does nothing in Vale, though one consumer sets it there and expects it to work.
hakkiri reads a comment as Markdown ([comments][comment-md]), so it applies the patterns to that Markdown, as to a Markdown file.
It does not copy the flaw.
The rewrite runs on the text of each comment with its markers removed, and the places of the nodes go back through the comment's own map to the file.

### 4. Patterns are checked when the config loads

A pattern that does not compile is an error with the key and its line, where Vale reports it when it lints a file the section matches.
The globs that match a file add their patterns in the order written, where Vale applies the sections in the order of a Go map, which is random.

### 5. A read that differs from Vale's

The probe of the comment controls showed three places where hakkiri read a file otherwise than Vale (2026-11-11, `tests/fixtures/markup-differences`).
Two were gaps that rows `HKR-048` and `HKR-049` fix: math in a Markdown file, and the string values of front matter.
The third stays: Vale runs no `sentence` rule on a comment of a source file, and hakkiri does, since it reads the comment as Markdown.
That finds more, so no row is queued.

### Not ported

- Vale rewrites the front matter into a fenced block before its patterns run ([html.go][v-apply]). hakkiri leaves the front matter to the parser, so a pattern sees its `---` lines.
- Vale places a code alert by searching the file for its text, so a code span that follows a block with no group can get the place of an earlier match of the same text. hakkiri places it by its bytes.

[v-apply]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/html.go#L72
[v-comment]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/code_ts.go#L199
[comment-md]: ../../specs/vale-differences/spec.md
