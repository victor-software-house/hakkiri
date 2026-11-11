# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.

## 1. The values

- [x] 1.1 Probe Vale at `66bbb85` for the keys, the scalar types, the scopes and the formats of front matter, and record Vale's alerts in `tests/fixtures/front-matter/vale.tsv`.  
  Verify the file holds the alerts of every scenario of the requirements, with the Vale commit named in the test.  
  Evidence, 2026-11-11 (UTC): a Vale build of `66bbb85` ran in a box over `tests/fixtures/front-matter`, ten files, nine rules and three configs, and gave 590 alerts.  
  The probe answers which keys Vale lints, as the design lists.
- [x] 1.2 Read the top-level string values of YAML, TOML and JSON front matter into nodes of the scope `frontmatter`, with their keys, and let the patterns leave them as written.  
  Verify `tests/front_matter.rs` fails before the change and passes after it.  
  Evidence, 2026-11-11 (UTC): `every_config_gives_the_alerts_vale_gives` failed on the source before the change, and passes after it for the three configs, less the 12 alerts that Vale misplaces.  
  With the front matter left out of the nodes, in a copy of the tree, it fails.

## 2. The surfaces

- [x] 2.1 Update `src/instructions.md`, `README.md` and `AGENTS.md`, drop the difference from the `block-token-ignores` change, and add the patch changeset.  
  Verify `operator_docs` passes and the changeset names `hakkiri: patch`.  
  Evidence, 2026-11-11 (UTC): the offline `verify` set passed in a box: `cargo fmt`, `clippy -D warnings --all-features`, the musl clippy, the no-default-features check, 706 tests, the doc-tests, `doc` on the nightly with warnings denied, `cargo deny` and `cargo machete`.  
  The two snapshots of `tests/tree.rs` that hold a front matter block each gained one `frontmatter` node, accepted from a real run, and the test that kept `A title` out of the prose now checks it is a node of its own.  
  The Mac gate passed: the prose lint, rumdl, typos, workflow lint, workflow policy, `check:local`, `specs:budget`, `format:toml` and `openspec validate --all --strict`.

## 3. Integration

- [ ] 3.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.
