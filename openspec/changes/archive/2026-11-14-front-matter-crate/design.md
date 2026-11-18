# Design

## Context

The files are Markdown and MDX (Markdown with JavaScript XML).  
hakkiri gives each top-level string in a file's front matter a node of the scope `frontmatter` ([front-matter-values]).  
It finds the block itself for three fences, and reads it with ctl-core's `Input::top_level`.  
markdown-rs cuts the `---` and `+++` blocks from the body.

Vale finds the block with `adrg/frontmatter` ([metadata.go][v-meta]).  
That library reads `---`, `---yaml`, `+++`, `---toml`, `;;;`, `---json` and a bare `{` object.

The [front-matter] crate finds the same forms, and reads each value with the parser of its format.  
YAML comes from `granit-parser` events, TOML from `toml`, and JSON from `struson`.  
The crate gives each top-level value its key, the byte span of the value as written, its style and its text.

## Goals / Non-Goals

Goals:

- One reader of front matter, shared with other tools, with one parser per format.
- The block is metadata, never body text, whatever its form.

Non-goals:

- Nested values: neither Vale nor hakkiri lints them.

## Decisions

### 1. The crate reads front matter

`src/frontmatter.rs` calls `front_matter::find` and `front_matter::read`.  
It maps each value's style to the bytes of its text, as before.  
A quoted string or a JSON string loses its quotes.  
A block scalar loses its header line and indent, and a TOML multi-line string loses its delimiters.

The crate follows the YAML 1.2 core schema.  
It leaves the rules of a reader such as yaml.v3, which Vale uses, to its caller.

yaml.v3 reads a plain date and the merge key `<<` as no string.  
It does the same for an integer with `_` between its digits or a `0b` prefix, such as `1_000` and `0b11`.  
js-yaml 4, the reader of most JavaScript site tools, does the same.

hakkiri leaves them out unless a tag or a quote makes them strings, as it did with ctl-core's reader.

A YAML key can be a list, a mapping or an alias.  
Such a key gives no value, and the entries after it are still read, as [front-matter 0.0.2][fm-keys] does.

Alternative considered: keep ctl-core's `Input`.  
It lost because it reads JSON as YAML, so a block that is no JSON passes as JSON.

### 2. Every form the crate finds is front matter

hakkiri reads the eight forms of the crate's `find`.  
Vale 3.26.0 reads the values of `---`, `+++`, `---toml`, `;;;`, a bare `{` object and YAML closed by `...`.  
It fails a `---yaml` or `---json` file with `E201` on line 1, because it parses the fence line as YAML ([probe](#probe)).

hakkiri reads those two by their fence, as the forms that `adrg/frontmatter` declares.

Alternative considered: keep the three fences.  
It lost because Vale reads five more, so a file Vale lints would lose its values in hakkiri.

### 3. The block is cut from the body for every form

hakkiri writes a space over each byte of the block that is not a line feed, before markdown-rs parses the file.  
So offsets and lines stay those of the file.

markdown-rs then sees blank lines, and its own front matter construct is off.  
The values come from the file as written, ahead of the body's nodes, as the ignore rewrite already reads them.

Vale cuts only the `---` and `+++` blocks from the body.  
It also lints the lines of a `;;;`, `---toml` or bare `{` block, and of YAML closed by `...`, as a paragraph.  
So a key such as `Install` gives an alert ([probe](#probe)).  
In markdown-rs, a `---toml` block is a paragraph that the closing `---` turns into a heading.

Alternative considered: keep markdown-rs's construct and lint the other blocks as body, as Vale does.  
It lost because a block is metadata: a JSON key or a TOML table header is no prose.

### 4. A fault shows the line where reading stopped

When a parser rejects a block, the file fails with `front matter: line N: …`, and the rest of the file is linted.  
The line is where the parser stopped.  
The crate counts it from the top of the file.

## Probe

Vale 3.26.0 ran with one `existence` rule of scope `text` that flags `Install`, on files whose body is `Body`:

| Block | Title `Install guide` | Key `Install` with value 1 |
|:--|:--|:--|
| `---` | 2:8 | none |
| `---yaml` | E201 | E201 |
| `+++` | 2:10 | none |
| `---toml` | 2:10 | 2:1 |
| `;;;` | 2:12 | 2:3 |
| `---json` | E201 | E201 |
| bare `{` on one line | 1:12 | 1:3 |
| bare `{` over three lines | 2:13 | 2:4 |
| `---` closed by `...` | 2:8 | 2:1 |

A key gives an alert only where Vale lints the block as a paragraph.

[front-matter]: https://crates.io/crates/front-matter
[fm-keys]: https://github.com/victor-software-house/front-matter/pull/9
[front-matter-values]: ../2026-11-14-front-matter-values/design.md
[v-meta]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/metadata.go
