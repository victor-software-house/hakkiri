# Proposal

Queue row: `HKR-012` in [tasks.yaml][ledger].

## Motivation

An alert points into the file through the span of a node, and a check reads the text of that node.  
A span is one range, so it cannot leave out the markers in the middle of a node.

Some paragraphs hold bytes that are syntax, not text.  
They sit in a block quote, over two line comments, or around an HTML tag.

## Changes

- Every node holds segments: the byte ranges of its text, in order, with the syntax between them left out.
- The text of a node is its segments joined, so no marker, bullet, number, indentation, line ending, heading hash or table pipe is in it.
- Code, HTML, link definitions, MDX expressions and the tags of JSX elements are gaps between the segments of the node around them.
- A block quote, a list item and a table row read as the content inside them.

## Capabilities

### New Capabilities

- `node-segments`: which byte ranges make the text of a node, and how they join into that text.

### Modified Capabilities

None.
The tree, its scopes and its spans stay in [document-tree][document-tree].

## Impact

- The `Node` type gains `segments` and a `text` method, and the tree builders produce them.
- A rule engine slice (`HKR-004`) reads text through the segments, so it never sees syntax.

[ledger]: ../../../tasks.yaml
[document-tree]: ../document-tree/proposal.md
