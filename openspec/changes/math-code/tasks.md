# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.

## 1. The math

- [x] 1.1 Probe Vale at `66bbb85` for inline spans and display blocks in Markdown and MDX, and record Vale's alerts in `tests/fixtures/math/vale.tsv`.  
  Verify the file holds the alerts of every scenario of the requirements, with the Vale commit named in the test.  
  Evidence, 2026-11-11 (UTC): a Vale build of `66bbb85` ran in a box over `tests/fixtures/math`, four files and five rules, and gave 80 alerts.
- [x] 1.2 Find the math in each paragraph, heading and table cell, write the file again with the same length, and parse it, giving a span's code node its delimiters.  
  Verify `tests/math.rs` fails before the change and passes after it.  
  Evidence, 2026-11-11 (UTC): `every_file_gives_the_alerts_vale_gives` failed on the source before the change, and passes after it for the 80 alerts.  
  With the display blocks left alone, in a copy of the tree, it fails.

## 2. The surfaces

- [x] 2.1 Update `src/instructions.md`, `README.md` and `AGENTS.md`, remove the difference from the `block-token-ignores` change, and add the patch changeset.  
  Verify `operator_docs` passes and the changeset names `hakkiri: patch`.  
  Evidence, 2026-11-11 (UTC): the offline `verify` set passed in a box: `cargo fmt`, `clippy -D warnings --all-features`, the musl clippy, the no-default-features check, 674 tests, the doc-tests, `doc` on the nightly with warnings denied, `cargo deny` and `cargo machete`.  
  The Mac gate passed: the prose lint, rumdl, typos, workflow lint, workflow policy, `check:local`, `specs:budget`, `format:toml`, `qctl check` and `openspec validate --all --strict`.

## 3. Integration

- [ ] 3.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.
