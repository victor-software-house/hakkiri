# Design

## Context

The [rule-format design][rule-format] fixed the scope names a rule selects text with, and that every comment reads as Markdown.  
The [bootstrap design][bootstrap] chose markdown-rs for Markdown and MDX and tree-sitter grammars for source languages.  
This slice builds the tree both feed, with no checks yet.

## Goals / Non-Goals

**Goals:**

- One tree type for every file, each node holding its scope name, its byte span in the file, and its children.
- Spans that point into the original file, also for text that sits inside a comment.

**Non-Goals:**

- Sentences and words, which come with word counting in `HKR-004`.
- Checks, alerts and the command line.

## Decisions

### 1. markdown-rs parses Markdown and MDX

[markdown-rs][markdown-rs] 1.0.0 parses `.md` with GFM and front matter, and `.mdx` with its MDX constructs.  
Every mdast node carries a start and end offset, which become the node's span.  
It is the only Rust parser with MDX and a position on each node, as the [bootstrap research][research] found.

Alternative: comrak. Lost: no MDX, so `.mdx` would need a second parser.

### 2. tree-sitter finds comments

tree-sitter with the [Rust][ts-rust] 0.24.2, [TypeScript][ts-ts] 0.23.2 and [JavaScript][ts-js] 0.25.0 grammars finds each comment node.  
Only comment nodes enter the tree, so a string literal never does.  
The grammar names the comment type: Rust has `line_comment`, `block_comment` and a `doc_comment` marker; TypeScript and JavaScript have `comment`, whose text shows `//`, `/*` or `/**`.

Alternative: a lexer per language, such as `ra_ap_rustc_lexer`. Lost: one per language, where tree-sitter adds a language as a grammar.

### 3. Comment text keeps a map to the file

hakkiri strips the comment markers: `//`, `///`, `//!`, `/*`, `/**`, `/*!`, `*/`, and a leading `*` on each line.  
It joins adjacent line comments of the same type into one block, as rustdoc does.  

It keeps, for each byte of the stripped text, the byte it came from in the file.  
The stripped text parses as Markdown (rule-format decision 8).  
Each Markdown node's span maps back through that table.

### 4. Scopes from mdast nodes

| mdast node | Scope |
|:--|:--|
| `paragraph` | `paragraph` |
| `heading` of depth 1 to 6 | `heading.h1` to `heading.h6` |
| `blockquote` | `blockquote` |
| `listItem` | `list` |
| `tableRow` | `table.row` |
| `tableCell` in the first row, and in the others | `table.header`, `table.cell` |
| `link`, `linkReference` | `link` |
| `inlineCode` | `code` |
| `strong`, `emphasis` | `strong`, `emphasis` |
| `image` alt text | `alt` |

These carry no prose scope: fenced code, raw HTML, link definitions, front matter, MDX expressions, MDX imports, and the tags of JSX elements.  
Text inside a JSX element is prose.  
`raw` holds the whole file for Markdown and MDX, and the comment text for a source file.

### 5. JSDoc and TSDoc tags

In a `/** */` comment of TypeScript or JavaScript, a line that starts with `@` begins a tag.  
The tag name, and the parameter name after `@param` or `@typedef`, are not prose; the description after them is.

### 6. insta snapshots

Each test file under `tests/fixtures/` has an [insta][insta] snapshot of its tree: scope, span and the text the span covers, one node per line.  
A change to the tree shows as a snapshot diff in review.

## Risks / Trade-offs

- [markdown-rs has had no release since 2025-04] → the first tests check its spans on table cells and list items.  
  comrak stays the fallback for `.md` if they fail.
- [TypeScript 0.23.2 is older than the other grammars] → each grammar crate exposes a language function that tree-sitter 0.27 loads.  
  CI proves the build.

[rule-format]: ../archive/2026-11-10-rule-format/design.md
[bootstrap]: ../archive/2026-11-09-bootstrap-hakkiri/design.md
[research]: ../archive/2026-11-09-bootstrap-hakkiri/research.md
[markdown-rs]: https://crates.io/crates/markdown/1.0.0
[ts-rust]: https://crates.io/crates/tree-sitter-rust/0.24.2
[ts-ts]: https://crates.io/crates/tree-sitter-typescript/0.23.2
[ts-js]: https://crates.io/crates/tree-sitter-javascript/0.25.0
[insta]: https://crates.io/crates/insta/1.49.0
