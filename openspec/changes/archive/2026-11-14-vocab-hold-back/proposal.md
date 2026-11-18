# Proposal

Queue row: `HKR-073` in [tasks.yaml][ledger].

## Motivation

The vocabularies spec stated that a check drops a match only when the match is an accepted term or lies inside an accepted phrase.
Vale 3.26.0 and hakkiri both hold back more.
They also hold back a match that contains an accepted word anywhere in it, as a whole word with its case.

hakkiri builds that rule in [`exceptions_with`][build], and [`excepted`][check] matches it anywhere in the matched text.

The ProseStyle v0.0.16 alert diff showed a chain that contained `harness`.
Vale gave an alert for it and hakkiri did not.
A probe on 2026-11-14 found the cause in the config of that run.

The ProseStyle package ships its vocabulary for hakkiri in `.ctl/hakkiri.yaml` and has no `.vale.ini`, so Vale read no vocabulary.
With `Vocab = ProseStyle` at the top level or in a glob, Vale drops the chain's alert, as hakkiri does.

## Changes

- The vocabularies spec states the rule both tools follow, in the top-level and the glob requirements.
- A scenario and a test pin it down.
  A match that contains `harness` is held back.
  `Harness`, `harnessing` and a match with no accepted word give alerts, as they do in Vale 3.26.0.
- No code changes, and no difference from Vale is added or removed.

## Capabilities

### Modified Capabilities

- `vocabularies`: which matches an accepted word keeps from alerting.

## Impact

- Changed: `openspec/specs/vocabularies/spec.md` through this change, and `tests/vocabularies.rs`.

[ledger]: https://github.com/victor-software-house/hakkiri/blob/9cfe923fcc4cd9490cd3fa26db21bf363e5b3da6/tasks.yaml
[build]: https://github.com/victor-software-house/hakkiri/blob/9cfe923fcc4cd9490cd3fa26db21bf363e5b3da6/src/rule/build.rs#L206-L228
[check]: https://github.com/victor-software-house/hakkiri/blob/9cfe923fcc4cd9490cd3fa26db21bf363e5b3da6/src/check.rs#L157-L170
