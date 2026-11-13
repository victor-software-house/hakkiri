# Tasks

Every compile, lint and test step runs in a box or the pull request's CI, not on the authoring machine.

## 1. The pattern check

- [x] 1.1 Make `run_existence` read a code span as one `*` and drop a match only when it starts or ends on a span.
  Verify the integration test of a match across a span in `tests/alerts.rs` fails on the source before the change and passes after it.
  Verify the test of an edge on a span passes, and the departures of `tests/fixtures/html-block` list the two rows the change adds.
  Evidence, 2026-11-12 (UTC): in a box `cargo test --no-fail-fast` passes and `clippy --lib --bins --tests -D warnings` passes.
  `ClauseChain` gives 1,536 alerts where it gave 1,020, and Vale gives 1,492, with none that only Vale gives.

## 2. Integration

- [ ] 2.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.
