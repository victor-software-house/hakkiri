# Proposal

Queue row: `HKR-028` in [tasks.yaml][ledger].  
It is the second of three changes that ship the command line, after [`check-command`][check] and before [`release-lane`][release].

## Motivation

ProseStyle runs `vale test --coverage` in its CI: every rule carries `tests:` cases, and the run fails when a rule fires in no case.
`HKR-008` moves ProseStyle onto hakkiri, and its acceptance is that every case passes, so hakkiri needs the command that runs them.
The rule-files spec already says how a case reads and passes, but the loader refuses a `tests` field as unknown, and no command runs a case.

## Changes

- `hakkiri test` runs the `tests` of rule files and the cases of `*.test.yml` files, and exits 1 when a case fails or, with `--coverage`, a rule fires in none.
- A rule file may carry `tests`, which `check` loads and does not run.
- The `vale-differences` spec lists where the command differs from `vale test`.

## Capabilities

### New Capabilities

- `test-command`: which cases `test` finds, how each runs, and how the report and the exit status read.

### Modified Capabilities

- `vale-differences`: the test command requirement is added.

## Impact

- No new dependency.
- `src/rule/schema.rs` gains a `tests` field on every check type, and new modules hold the case reader and runner.
- `src/instructions.md` and `skills/hakkiri/SKILL.md` gain the command.

[ledger]: ../../../tasks.yaml
[check]: ../check-command/proposal.md
[release]: ../release-lane/proposal.md
