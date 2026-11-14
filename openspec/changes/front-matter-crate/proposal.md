# Proposal

## Motivation

hakkiri reads front matter through ctl-core's config reader, which reads JSON as YAML, so a `;;;` block such as `{title: Install guide}` passes when it is no JSON.
markdown-rs reads only the `---` and `+++` blocks, so a `;;;` block is linted twice, once as values and once as a paragraph, and a `---toml` block turns into a heading.
The operator ruled on 2026-11-13 that front matter is document content, read by one parser per format, and on 2026-11-14 that the reader is its own crate, [front-matter], on crates.io.
This change is row `HKR-068` of the [ledger].

## Changes

- hakkiri finds and reads front matter through the `front-matter` crate, and ctl-core's `Input` no longer reads it.
- Every form the crate finds is front matter: `---`, `---yaml`, `+++`, `---toml`, `;;;`, `---json`, a bare `{` object, and YAML closed by `...`.
- The block is cut from the body for every form, so no rule reads its lines as a paragraph, a heading or a sentence.
- A block its parser refuses fails the file with the crate's line and message, and the rest of the file is linted, as before.

## Capabilities

### Modified Capabilities

- `front-matter`: the forms of block, and the block as no body text.
- `vale-differences`: the blocks Vale lints as body text, and the named YAML and JSON fences Vale fails.

## Impact

- New dependency: `front-matter` 0.0.2, MIT OR Apache-2.0, with `granit-parser` 2, `toml` 1 and `struson` 0.7 under it.
- `src/frontmatter.rs` and `src/markdown.rs` change; markdown-rs's front matter construct is off.

[front-matter]: https://crates.io/crates/front-matter
[ledger]: ../../../tasks.yaml
