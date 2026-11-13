# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.

## 1. The patterns

- [x] 1.1 Probe Vale at `66bbb85` for block and token patterns on Markdown, MDX and Rust files, and record Vale's alerts in `tests/fixtures/ignores/vale.tsv`.  
  Verify the file holds the alerts of every scenario of the requirements, with the Vale commit named in the test.  
  Evidence, 2026-11-11 (UTC): a Vale build of `66bbb85`, with the Rust grammar built from the same commit, ran in a box over `tests/fixtures/ignores` with the ten configs in `vale/`, and gave 709 alerts.  
  A second probe called Vale's `applyPatterns` on eighteen texts and printed the rewrite of each.
- [x] 1.2 Add `block_ignores` and `token_ignores` to a glob, rewrite a Markdown or MDX file under them, and give every node of the parsed tree the bytes of the file it came from.  
  Verify `tests/ignores.rs` fails before the change and passes after it.  
  Evidence, 2026-11-11 (UTC): the six tests failed on the source before the change, and `every_config_gives_the_alerts_vale_gives` passes after it for the ten configs.  
  With the block patterns left out, or the places of the nodes left unrestored, in copies of the tree, it fails.  
  With the token patterns run before the block patterns it passes, so a unit test of the rewrite holds that order.
- [x] 1.3 Check each pattern when the config loads, and add the patterns of every matching glob in the order written.  
  Verify the scenarios on a pattern that does not compile, a word for a list, a package and the project, and two globs pass.  
  Evidence, 2026-11-11 (UTC): `an_invalid_pattern_is_an_error_on_its_key`, `a_word_where_a_list_of_patterns_belongs_is_an_error`, `a_packages_ignores_and_the_projects_add_up` and the `twoglobs` config pass.
- [x] 1.4 Apply the patterns to the Markdown of each comment of a source file, which Vale does not do.  
  Verify the `rust` config of `tests/ignores.rs` gives Vale's alerts less the two the patterns hide.  
  Evidence, 2026-11-11 (UTC): in a box, hakkiri hides `very` between `BEGIN` and `END` in `//!` lines and `really` in `{{ really }}` in a `///` comment, and the rest of the 709 alerts match Vale's.  
  The new test of a doc comment and a string failed on the source before the change.  
  The full suite, 658 tests, passed in a box with `cargo fmt`, `clippy -D warnings` on the host and musl targets, `cargo deny` and `cargo machete`.

## 2. The surfaces

- [x] 2.1 Update `src/instructions.md`, `skills/hakkiri/SKILL.md`, `README.md` and `AGENTS.md`, record the differences in the `vale-differences` spec, and add the patch changeset.  
  Verify `operator_docs` passes, the instructions name both keys, and the changeset names `hakkiri: patch`.  
  Evidence, 2026-11-11 (UTC): the full suite, 621 tests, passed in a box with `cargo fmt`, `clippy -D warnings` on the host and musl targets, `cargo deny` and `cargo machete`, and `operator_docs` with it.  
  The Mac gate passed: the prose lint, rumdl, typos, workflow lint, workflow policy, `check:local`, `specs:budget`, `format:toml`, `qctl check` and `openspec validate --all --strict`.

## 3. Integration

- [x] 3.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.  
  Evidence, 2026-11-12 (UTC): PRs 95 and 102 merged, and CI on main at 930ecee passed: `Build and test`, `Static checks` and `Verify`.
