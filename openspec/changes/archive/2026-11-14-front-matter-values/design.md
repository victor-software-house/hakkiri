# Design

## Context

Vale lints the front matter before it reads the body.  
It parses YAML and JSON with a YAML parser, and it finds each TOML string by searching the block for it ([metadata.go][v-meta]).  
It keeps the top-level string scalars ([frontmatter.go][v-values]).  
hakkiri's parser reads the block as one node with no prose, and the tree dropped it.

## Goals / Non-Goals

Goals:

- The alerts in front matter match those of Vale on the probe files.  
  This covers YAML, TOML and JSON, in Markdown and in MDX.
- A rule reaches a value by `text`, `frontmatter` and the key, as in Vale.

Non-goals:

- A reader of YAML or TOML in hakkiri: ctl-core's `Input::top_level` reads both.
- Nested values: Vale lints none, and neither does hakkiri.

## Decisions

### 1. The values come from ctl-core's top-level entries

hakkiri has no YAML or TOML parser of its own.  
An earlier version read the block line by line, and it skipped anchors, aliases and complex keys.  
So [ctl-core 0.7.4][ctl-core-top-level] gives it `Input::top_level` instead.

The method lists each top-level entry of a YAML or TOML text with these parts:

- the key;
- the bytes of the value as written;
- how the value is written;
- its text, when the parser reads it as a string.

serde-saphyr sets the types of YAML, and the `toml` crate sets those of TOML.  
yamled and the `toml` tree give the spans.  
The parser resolves anchors, tags and aliases, so an anchored or tagged value keeps its place.

The listing rejects nothing that YAML allows.  
A key written twice, a merge key, and `.inf`, `-.inf` and `.NaN` are entries.  
A key that is a number, a boolean or a null is listed by its text.  
An entry whose key is a sequence, a mapping or an alias is left out, and the entries after it are kept.

hakkiri takes the bytes of the value from the entry and cuts them down to the text:

- a quoted value loses its quotes;
- a block scalar loses its header line and its indent;
- a TOML multi-line string loses its delimiters.

Each value becomes a node of the new scope `frontmatter`, with its key as the node's label.  
Its segments are the bytes of the text in the file.  
A folded scalar and a scalar over lines join their lines with a space, and a literal scalar joins them with a line feed.

A block the parser rejects has no values, and it is a fault of the file ([decision 6](#6-a-block-no-parser-reads-is-a-fault-of-the-file)).  
JSON front matter is no markdown-rs construct.  
A YAML parser reads it, since a JSON object is a YAML flow mapping.

Vale reads with yaml.v3, and yaml.v3 and serde-saphyr differ on two plain scalars.  
yaml.v3 reads a date and the merge key `<<` as no string, and serde-saphyr reads them as strings.  
hakkiri leaves those two out unless a tag or a quote makes them strings.

The probe of Vale at `66bbb85` (2026-11-11) shows which keys it lints.  
It ran over ten files and 590 alerts in three configs:

- A top-level key whose value is a string is linted.  
  In YAML that is a plain, a quoted, a folded or a literal scalar.  
  In TOML it is a string before the first table, and in JSON a string member of the object.
- Never a key, a list, a flow collection, a nested mapping, or a member of a TOML table or a JSON object below the top.
- A plain scalar that resolves to another type is not a string.  
  Those are `null`, `~`, `true` and `false` in three cases, every integer and float form, and `.inf` and `.nan`.  
  A date, a date with a time, and `<<` are on the list too.
- serde-saphyr reads all of these as no string, except the dates and `<<`.
- A scalar under another tag such as `!!int` is not a string either.
- `yes`, `no`, `on`, `off`, `y`, `n`, `12:30`, `1,000`, `1.2.3`, `v1.0`, `1e`, `0xZZ` and `a:b` are strings, and so is `!!str tagged`.
- A comment after the scalar is no part of it, as in `very # trail`.

### 2. A key completes the scope

Vale scopes a value `text.frontmatter.<key>`.  
A rule scoped `text`, `frontmatter`, `text.frontmatter`, `frontmatter.<key>` or `text.frontmatter.<key>` reads it, and a rule scoped `paragraph` or `sentence` does not.  
The node names are `text`, `frontmatter.<key>` and `text.frontmatter.<key>`.  
The scope check accepts a name that goes on after `frontmatter` or `text.frontmatter`.

### 3. JSON front matter is read and not cut

A file that starts with a line `;;;`, an object and a line `;;;` has JSON front matter.  
markdown-rs reads no such block, so the text stays a paragraph of the body.

Vale lints it as a paragraph too, and its probe gives a `paragraph` alert and a `frontmatter` alert for the same word.  
hakkiri adds the values and leaves the body as it was.

### 4. The patterns leave the front matter

Vale reads the metadata from the file as written, before `BlockIgnores` and `TokenIgnores` rewrite the body.  
So a pattern that matches in a value hides nothing there.  
The ignore rewrite of hakkiri reads the front matter again from the file as written, and drops what the rewritten text gave.

### 5. Where Vale puts an alert

The probe shows alerts that Vale puts at the wrong place, and hakkiri puts them at the text:

- The words of the second line of a folded scalar land on its first line.
- A TOML value is found by searching the block, so `title` in `title = "A very toml title"` lands on the key.
- The text of a scalar under `!!str` starts at the tag, and the text of a scalar after an anchor starts at the anchor.

### 6. A block no parser reads is a fault of the file

Vale fails a file when its parser rejects the front matter, and also when the block has a sequence key.  
It reports E201 on line 1 and lints none of the file.  
hakkiri reads the block with the parser that reads its values.  
So the sequence key is no fault, and every other value is read.

A block the parser rejects gives no values and a fault.  
The tree stores the fault with the line of the file and the parser's message.  
`hakkiri check` reports the fault as a failure of the file, which exits 2, and lints the rest of the file.  
The operator then sees both the failure and the alerts.

[ctl-core-top-level]: https://docs.rs/ctl-core/0.7.4/ctl_core/input/struct.Input.html#method.top_level
[v-meta]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/metadata.go
[v-values]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/frontmatter.go
