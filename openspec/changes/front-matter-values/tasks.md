# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.

## 1. The values

- [x] 1.1 Probe Vale at `66bbb85` for the keys, the scalar types, the scopes and the formats of front matter, and record Vale's alerts in `tests/fixtures/front-matter/vale.tsv`.  
  Verify the file holds the alerts of every scenario of the requirements, with the Vale commit named in the test.  
  Evidence, 2026-11-11 (UTC): a Vale build of `66bbb85` ran in a box over `tests/fixtures/front-matter`, ten files, nine rules and three configs, and gave 590 alerts.  
  The probe answers which keys Vale lints, as the design lists.
- [x] 1.2 Read the top-level string values of YAML, TOML and JSON front matter into nodes of the scope `frontmatter`, with their keys, YAML and TOML through ctl-core's `Input::top_level`, and let the patterns leave them as written.  
  Verify `tests/front_matter.rs` fails before the change and passes after it.  
  Evidence, 2026-11-12 (UTC): in a cloud box, `every_config_gives_the_alerts_vale_gives` gave the 590 alerts of Vale over the three configs, less the 4 it misplaces, once dates and `<<` were left out in hakkiri.  
  The first run failed on the dates and on `<<`, which serde-saphyr reads as strings, and on the three non-finite floats of the probe file, which ctl-core refuses (the vale-differences spec lists them).  
  Unit tests in `src/frontmatter/tests.rs` pin an anchored value, an anchored key, an alias, a sequence as a key, a repeated key and an opening fence with blanks.

## 2. The surfaces

- [x] 2.1 Update `src/instructions.md`, `README.md` and `AGENTS.md`, drop the difference from the `block-token-ignores` change, and add the patch changeset.  
  Verify `operator_docs` passes and the changeset names `hakkiri: patch`.  
  Evidence, 2026-11-12 (UTC): in the same box, the whole `cargo test` ran with every test binary passing, `cargo clippy --all-targets --all-features -D warnings` passed, and the formatting nightly `cargo fmt` changed three files, which are in the commit.  
  Not run here: `cargo deny`, `cargo machete`, the doc build and the musl clippy, which the pull request's CI runs.  
  The release binary grew from 14,375,208 to 15,038,184 bytes (662,976, 4.6%) with ctl-core 0.7.3 and its `input-toml` feature, one build of each in the box.

## 3. Integration

- [ ] 3.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.
