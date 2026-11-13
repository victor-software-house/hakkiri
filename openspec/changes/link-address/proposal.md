# Proposal

Queue row: `HKR-064` in [tasks.yaml][ledger].

## Motivation

Vale reads a link whose text is its address, such as `[research.md](research.md)`, as a bare URL, and counts no word for it.
hakkiri counts the words of the text, two for `research.md`.
On 2026-11-12 the word-count probe of [HKR-063][ledger] found it in 70 of the 860 differences that Vale's duplicate-position drop does not explain, and in the paragraph and file metrics of the same text.
A link of this shape is a name that repeats the address, and no prose, so hakkiri reads it as Vale does.

## Changes

- A Markdown link, inline or by reference, whose text is one run of text equal to its address is a gap in the prose around it, as an autolink is, and stays in its `link` node.
- The address of a reference link is the destination of the first definition of its label.
- The address is compared as goldmark writes it into the `href`, percent-encoded, since Vale compares the text with that `href`.
- The `node-segments` spec states the rule.
- The `vale-differences` spec records a bare address with a letter outside ASCII, which goldmark's linkify ends before that letter.

## Capabilities

### Modified Capabilities

- `node-segments`: a link that names its address is a gap in the prose around it.
- `vale-differences`: a bare address ends where GFM ends it.

## Impact

- Changed: `src/markdown.rs` and `tests/metric.rs`.
- No new dependency and no new key.

[ledger]: ../../../tasks.yaml
