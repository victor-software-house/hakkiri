# Proposal

Queue row: `HKR-075` in [tasks.yaml][ledger].

## Motivation

hakkiri cut an inline `img` tag in a line of Markdown whole, so its alt text read as nothing.
Vale reads that alt text as an `alt` block of its own, as it reads the alt text of a Markdown image.
The `vale-differences` spec lists this as a known gap ([markdown-probe 9][probe-9]).

A probe on 2026-11-14 ran Vale 3.26.0 on three paragraphs that hold an inline `img`: alone, inside an `a` tag, and inside a Markdown link.
Vale gave each alt text an `alt` alert and a `text` alert.
It gave no `link` alert and no `sentence` alert on it, and the paragraph's word count left it out.

## Changes

- An inline HTML tag in a line of Markdown gives an `alt` node for the alt text of an `img`, from the same walk that gives its titles ([`html::walk`][walk]).
- The paragraph reads around the node, and a rule on `link` or on sentences does not select it, as in Vale.
- The opening tag of a paired inline element and of a JSX element keep their reading.
- The `html-block-text` spec gains the requirement, and the `vale-differences` gap is removed.

## Capabilities

### Modified Capabilities

- `html-block-text`: the alt text of an inline `img` tag.
- `vale-differences`: the known gap is gone.

## Impact

- Changed: `src/html_text.rs`, `src/markdown.rs` and `tests/select.rs`, and a patch changeset.

[ledger]: https://github.com/victor-software-house/hakkiri/blob/3c1bcb3cf953470019771ff7a2e89d478e63b8aa/tasks.yaml
[probe-9]: ../archive/2026-11-14-markdown-probe/design.md#9-known-gap-the-alt-text-of-an-inline-img
[walk]: https://github.com/victor-software-house/hakkiri/blob/3c1bcb3cf953470019771ff7a2e89d478e63b8aa/src/html.rs#L708-L721
