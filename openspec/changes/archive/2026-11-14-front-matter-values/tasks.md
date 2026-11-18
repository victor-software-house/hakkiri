# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.

## 1. The values

- [x] 1.1 Probe Vale at `66bbb85` for the keys, the scalar types, the scopes and the formats of front matter.
  Record Vale's alerts in `tests/fixtures/front-matter/vale.tsv`.
  Verify the file contains the alerts of every scenario of the requirements, and the test gives the Vale commit.
  - 2026-11-11 (UTC): A Vale build of `66bbb85` ran in a box over `tests/fixtures/front-matter` and gave 590 alerts.
    The run covered ten files, nine rules and three configs.
    The probe shows which keys Vale lints, as the design lists.
- [x] 1.2 Read the top-level string values of YAML, TOML and JSON front matter into nodes of the scope `frontmatter`, with their keys.
  Read YAML and TOML through ctl-core's `Input::top_level`, and let the patterns leave the values as written.
  Verify `tests/front_matter.rs` fails before the change and passes after it.
  - 2026-11-12 (UTC): In a cloud box, `every_config_gives_the_alerts_vale_gives` gave the 590 alerts of Vale over the three configs.
    It gave them less the 4 that Vale misplaces, once hakkiri left out dates and `<<`.
    The first run failed on the dates and on `<<`, which serde-saphyr reads as strings.
  - 2026-11-12 (UTC): Unit tests in `src/frontmatter/tests.rs` pin an anchored value, an anchored key, an alias and a sequence as a key.
    They also pin a repeated key and an opening fence with blanks.
- [x] 1.3 Read what ctl-core 0.7.4 lists, and fail a file whose front matter no parser reads.
  Verify the three original `.inf`, `-.inf` and `.NaN` values are back in the probe file f7.
  Verify the alerts of Vale match for a repeated key, a merge key, number and null keys and an alias.
  - Verify `hakkiri check` reports a block that no parser reads as a failure with its line, and exits 2.
    Verify it lints the rest of the file, and that a sequence key does neither.
  - 2026-11-12 (UTC): Vale at `66bbb85` ran again over the fixture folder and gave exactly the 896 rows of `vale.tsv`.
    They are the 590 of the first probe and 306 for `f11` to `f15`.
  - 2026-11-12 (UTC): Vale fails a file with a block no parser reads, and a block with a sequence key.
    It reports E201 on line 1 and lints none of the file.
  - 2026-11-12 (UTC): In a cloud box, `every_config_gives_the_alerts_vale_gives` passed.
    So did `front_matter_no_parser_reads_fails_the_file_and_leaves_the_rest_linted` and the 18 unit tests of `frontmatter`.

## 2. The surfaces

- [x] 2.1 Update `src/instructions.md`, `README.md` and `AGENTS.md`, drop the difference from the `block-token-ignores` change, and add the patch changeset.
  Verify `operator_docs` passes and the changeset lists `hakkiri: patch`.
  - 2026-11-12 (UTC): In the same box, the whole `cargo test` ran with every test binary passing.
    `cargo clippy --all-targets --all-features -D warnings` passed.
    The formatting nightly `cargo fmt` changed three files, which are in the commit.
  - 2026-11-12 (UTC): Not run here: `cargo deny`, `cargo machete`, the doc build and the musl clippy.
    The pull request's CI runs them.
  - 2026-11-12 (UTC): The release binary grew from 14,375,208 to 15,038,184 bytes (662,976, 4.6%).
    That is with ctl-core 0.7.3 and its `input-toml` feature, one build of each in the box.

## 3. Integration

- [x] 3.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.
  Evidence, 2026-11-14 (UTC): The CI of the stack that [pull request 121](https://github.com/victor-software-house/hakkiri/pull/121) heads passed, and the stack merged as `7aac10a`.
