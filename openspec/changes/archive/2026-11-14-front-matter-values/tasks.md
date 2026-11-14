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
  The first run failed on the dates and on `<<`, which serde-saphyr reads as strings.  
  Unit tests in `src/frontmatter/tests.rs` pin an anchored value, an anchored key, an alias, a sequence as a key, a repeated key and an opening fence with blanks.
- [x] 1.3 Read what ctl-core 0.7.4 lists, and fail a file whose front matter no parser reads.  
  Verify the three original `.inf`, `-.inf` and `.NaN` values are back in the probe file f7, and the alerts of Vale for a repeated key, a merge key, number and null keys and an alias match.  
  Verify `hakkiri check` reports a block that no parser reads as a failure with its line, exits 2, and lints the rest of the file, and that a sequence key does neither.  
  Evidence, 2026-11-12 (UTC): Vale at `66bbb85` run again over the fixture folder gave exactly the 896 rows of `vale.tsv`, which are the 590 of the first probe and 306 for `f11` to `f15`.  
  Vale fails a file with a block no parser reads, and a block with a sequence key, with E201 on line 1 and lints none of it.  
  In a cloud box, `every_config_gives_the_alerts_vale_gives` and `front_matter_no_parser_reads_fails_the_file_and_leaves_the_rest_linted` passed, and so did the 18 unit tests of `frontmatter`.

## 2. The surfaces

- [x] 2.1 Update `src/instructions.md`, `README.md` and `AGENTS.md`, drop the difference from the `block-token-ignores` change, and add the patch changeset.  
  Verify `operator_docs` passes and the changeset names `hakkiri: patch`.  
  Evidence, 2026-11-12 (UTC): in the same box, the whole `cargo test` ran with every test binary passing, `cargo clippy --all-targets --all-features -D warnings` passed, and the formatting nightly `cargo fmt` changed three files, which are in the commit.  
  Not run here: `cargo deny`, `cargo machete`, the doc build and the musl clippy, which the pull request's CI runs.  
  The release binary grew from 14,375,208 to 15,038,184 bytes (662,976, 4.6%) with ctl-core 0.7.3 and its `input-toml` feature, one build of each in the box.

## 3. Integration

- [x] 3.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.  
  Evidence, 2026-11-14 (UTC): The CI of the stack that [pull request 121](https://github.com/victor-software-house/hakkiri/pull/121) heads passed, and the stack merged as `7aac10a`.
