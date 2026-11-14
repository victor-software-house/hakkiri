# Tasks

## 1. Read

- [ ] 1.1 Depend on `front-matter` 0.0.2, drop ctl-core's `input-toml` feature, which only front matter used, and add the new crates to `deny.toml` where they duplicate a version.
  Verify: the static checks of CI pass, `cargo deny` included.
- [ ] 1.2 Read front matter through `front_matter::find` and `front_matter::read` in `src/frontmatter.rs`.
  Verify: the unit tests of `src/frontmatter/tests.rs` pass in CI, with a case per form.
- [ ] 1.3 Cut the block from the body for every form, with markdown-rs's front matter construct off.
  Verify: `tests/front_matter.rs` passes in CI, with the JSON block of `f8.md` no paragraph and a test per form.

## 2. Docs

- [ ] 2.1 Update `src/instructions.md` and `AGENTS.md`, and add a changeset.
  Verify: `src/instructions.md` names every form and `AGENTS.md` names the crate.
