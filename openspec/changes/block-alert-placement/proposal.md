# Proposal

Queue row: `HKR-074` in [tasks.yaml][ledger].

## Motivation

In the `vale-differences` spec, Vale points a measurement alert on a list item at column 1 of the block's first line.
The ProseStyle v0.0.16 alert diff showed otherwise: of 504 paired `ProseStyle.Paragraph` alerts, Vale placed 500 on the paragraph's last line.

A probe on 2026-11-14 ran Vale 3.26.0 and Vale 66bbb85 on one file with a `metric` rule over paragraphs and one over list items.
Both put each alert at column 1 of the block's last line.
hakkiri puts it at the first byte of the node, past the bullet.
Both tools count a nested list item as its own block.

## Changes

- The `vale-differences` scenario for a `metric` alert states that Vale uses the block's last line, with the probe's input and alerts.
- A new scenario covers a nested list item.
- A test in `tests/metric.rs` pins hakkiri's alerts on the probe file and names Vale 66bbb85's lines.
- No code changes, and the difference itself stays.

## Capabilities

### Modified Capabilities

- `vale-differences`: where Vale places a `metric` alert on a block.

## Impact

- Changed: `openspec/specs/vale-differences/spec.md` through this change, and `tests/metric.rs`.

[ledger]: https://github.com/victor-software-house/hakkiri/blob/9cfe923fcc4cd9490cd3fa26db21bf363e5b3da6/tasks.yaml
