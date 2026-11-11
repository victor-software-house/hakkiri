# Design

## Context

Vale lints the front matter before it reads the body.
It parses YAML and JSON with a YAML parser and each TOML string by searching the block for it ([metadata.go][v-meta]), and it keeps the top-level string scalars ([frontmatter.go][v-values]).
hakkiri's parser reads the block as one node with no prose, and the tree dropped it.

## Goals / Non-Goals

Goals:

- The alerts in front matter match those of Vale on the probe files, for YAML, TOML and JSON, in Markdown and MDX.
- A rule reaches a value by `text`, `frontmatter` and the key, as in Vale.

Non-goals:

- YAML beyond what a top-level `key: value` line uses: anchors, aliases, complex keys and tags other than `!!str`.

## Decisions

### 1. The values come from a read of the block's lines

hakkiri has no YAML crate of its own, and a new dependency needs approval, so a small reader takes the top-level entries of the block line by line.
An entry holds a plain, single-quoted or double-quoted scalar, which may go on over the lines under it, or a `|` or `>` block scalar.
Each value becomes a node of the new scope `frontmatter`, with its key as the node's label, and its segments are the bytes of the text in the file.
A folded scalar and a scalar over lines join their lines with a space, and a literal scalar with a line feed.

The probe of Vale at `66bbb85` (2026-11-11), over ten files and 590 alerts in three configs, answers which keys it lints:

- A top-level key whose value is a string is linted.
  In YAML that is a plain, a quoted, a folded or a literal scalar.
  In TOML it is a string before the first table, and in JSON a string member of the object.
- Never a key, a list, a flow collection, a nested mapping, or a member of a TOML table or a JSON object below the top.
- A plain scalar that resolves to another type is not a string.
  Those are `null`, `~`, `true` and `false` in three cases, every integer and float form, `.inf` and `.nan`, a date and a date with a time, and `<<`.
  A scalar under another tag such as `!!int` is not a string either.
- `yes`, `no`, `on`, `off`, `y`, `n`, `12:30`, `1,000`, `1.2.3`, `v1.0`, `1e`, `0xZZ` and `a:b` are strings, and so is `!!str tagged`.
- A comment after the scalar is no part of it, as in `very # trail`.

### 2. A key completes the scope

Vale scopes a value `text.frontmatter.<key>`.
A rule scoped `text`, `frontmatter`, `text.frontmatter`, `frontmatter.<key>` or `text.frontmatter.<key>` reads it, and a rule scoped `paragraph` or `sentence` does not.
The node names are `text`, `frontmatter.<key>` and `text.frontmatter.<key>`, and the scope check accepts a name that goes on after `frontmatter` or `text.frontmatter`.

### 3. JSON front matter is read and not cut

A file that starts with a line `;;;`, an object and a line `;;;` has JSON front matter.
markdown-rs reads no such block, so the text stays a paragraph of the body, and Vale lints it as that too: its probe gives a `paragraph` alert and a `frontmatter` alert for the same word.
hakkiri adds the values and leaves the body as it was.

### 4. The patterns leave the front matter

Vale reads the metadata from the file as written, before `BlockIgnores` and `TokenIgnores` rewrite the body, so a pattern that matches in a value hides nothing there.
The ignore rewrite of hakkiri reads the front matter again from the file as written and drops what the rewritten text gave.

### 5. Where Vale puts an alert

The probe shows three alerts that Vale puts at the wrong place, and hakkiri puts them at the text:

- The words of the second line of a folded scalar land on its first line.
- A TOML value is found by searching the block, so `title` in `title = "A very toml title"` lands on the key.
- The text of a scalar under `!!str` starts at the tag.

[v-meta]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/metadata.go
[v-values]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/frontmatter.go
