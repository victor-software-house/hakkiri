# Tasks

## 1. Read

- [ ] 1.1 Depend on `front-matter` 0.0.2, and add its new crates to `deny.toml` where they duplicate a version.
  Verify: the `deny` job of CI passes.
- [ ] 1.2 Read front matter through `front_matter::find` and `front_matter::read` in `src/frontmatter.rs`.
  Verify: the unit tests of `src/frontmatter/tests.rs` pass in CI, with a case per form.
- [ ] 1.3 Cut the block from the body for every form, with markdown-rs's front matter construct off.
  Verify: `tests/front_matter.rs` passes in CI, with the JSON block of `f8.md` no paragraph and a test per form.

## 2. Docs

- [ ] 2.1 Update `src/instructions.md`, `skills/hakkiri/SKILL.md` where it names the forms, `README.md`, `AGENTS.md` and a changeset.
  Verify: the docs name the eight forms and the crate.
