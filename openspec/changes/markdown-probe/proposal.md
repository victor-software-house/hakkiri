# Proposal

Queue row: [`HKR-027`][ledger].

## Motivation

hakkiri reads Markdown through markdown-rs, and Vale reads the HTML that goldmark writes.
The two read some constructs otherwise, and no probe had measured where.
The ledger row held two cases found by chance: an inline tag between letters, and a footnote.
A probe of Vale at `66bbb85` over 111 inputs, each holding one construct, gave 2155 alerts under a style of eleven rules.
hakkiri before this change gave most of them, and each difference below came from one of its readings.

## Changes

- A new fixture, `tests/fixtures/markdown-corpus`, holds the inputs, Vale's rows and the 201 rows that only one side gives, with a test that fails on any change to them.
  hakkiri gives 2056 of Vale's 2155 rows and 102 of its own, 44 of them the kept addresses of the `link` scope.
- hakkiri reads each construct below as Vale does, where it read it otherwise:
  - A table row has the cells of its header, and a cell past the last one is a gap.
  - A bare email address, a `www.` address, an address with an upper-case scheme and a `mailto:` address are text, and an `ftp://` address is a gap.
  - The `link` scope keeps the address of an autolink, a bare URL and a bracket link whose text is its destination, where Vale gives it no text, because ProseStyle's `BareURL` rule reads the address there.
  - An inline HTML element that Vale scopes, `b`, `strong`, `i`, `em`, `code`, `tt` or `a`, has that scope.
  - A footnote reference is a gap, a definition no reference names is a gap, and a definition that one names is a `list` node.
- hakkiri reads a backslash escape and a hard break written with a backslash without the backslash, where Vale gives no alert for a word that holds an escape.
- The `vale-differences` spec drops the known gap of the Markdown parser and the scenario where inline HTML has no element scope, and lists the nine places where hakkiri differs on purpose.
- The text of an inline `img` tag stays unread, which `vale-differences` lists as a known gap, and row `HKR-075` takes it.

## Capabilities

### Modified Capabilities

- `node-segments`: links, escapes, table cells, footnotes and inline HTML elements.
- `vale-differences`: the known gap closes, and the differences are listed.

## Impact

- New: `tests/markdown_corpus.rs` and `tests/fixtures/markdown-corpus`.
- Changed: `src/markdown.rs`, and the link tests of `tests/tree.rs` and its snapshots.
- A `link` rule still sees the address of an autolink and a bare URL, and `vale-differences` lists it.
- No new dependency.

[ledger]: ../../../tasks.yaml
