# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.

## 1. The probe

- [x] 1.1 Probe Vale at `66bbb85` for the built-in rule on Markdown and Rust files, under five configs.  
  Record its alerts in `tests/fixtures/builtin-repetition`.  
  Verify `vale.tsv` contains the alerts of every scenario of the requirements, and the test cites the Vale commit.
  - 2026-11-12 (UTC): a Vale build of `66bbb85` ran in a cloud box over three Markdown files, one Rust file and five configs.  
    It gave 86 alerts under the plain config and 87 under the two vocabulary configs.  
    Those configs add `Vale.Terms` alerts and drop three repeats.  
    The first probe showed one case hakkiri missed, `ΣΑΣ σασ`.

## 2. The rule

- [x] 2.1 Build `Vale.Repetition` into the style `Vale`, and let accepted terms hold repeats back.  
  Compare words by simple case folding.  
  Verify `tests/builtin_repetition.rs` fails before the change and passes after it.
  - 2026-11-12 (UTC): in the same box, `every_config_gives_the_alerts_vale_gives` gave Vale's alerts under all five configs, once case folding replaced the whole-word lowercase.  
    `the_style_vale_runs_the_rule_with_no_vocabulary` passed.
  - The two differ in one place: a repeat in a doc comment that Markdown reads as a link, `[link link](u)`.  
    hakkiri reads every comment as Markdown, and the fixture leaves that case out.

## 3. The surfaces

- [x] 3.1 Turn `Vale.Repetition` off in the older Vale-style configs of the fixtures.  
  Update `src/instructions.md`, the skill, `README.md` and `AGENTS.md`.  
  Add the patch changeset, and queue `HKR-059` for rule parameters.  
  Verify `operator_docs` passes and the changeset contains `hakkiri: patch`.
  - 2026-11-12 (UTC): in the same box, the whole `cargo test` ran with every test binary passing, and `cargo clippy --all-targets` passed.  
    On the authoring machine, `openspec validate --all --strict` and `qctl check --no-git` passed.  
    Not run here: `cargo deny`, `cargo machete`, the doc build and the musl clippy, which the pull request's CI runs.

## 4. Integration

- [x] 4.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.  
  Evidence, 2026-11-14 (UTC): The CI of the stack that [pull request 121](https://github.com/victor-software-house/hakkiri/pull/121) heads passed, and the stack merged as `7aac10a`.
