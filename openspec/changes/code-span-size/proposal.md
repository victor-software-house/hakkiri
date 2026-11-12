# Proposal

Queue rows: `HKR-051` and `HKR-063` in [tasks.yaml][ledger].

## Motivation

The ProseStyle port found two statements about a code span that look opposed.
The `vale-differences` spec holds that a code span counts one word in hakkiri and none in Vale.
ProseStyle `LongSentence` counts the words of a sentence with an `occurrence` rule, and under hakkiri it counts none for a code span.
A probe of both tools on one sentence shows both statements hold, for two different checks.

A project that wants Vale's counts has no way to get them.
The operator decided on 2026-11-12 that a code span counts as one word by default, and that a setting can change the number of words, to 0 to match Vale.

## Changes

- A test pins that a `metric` rule counts a code span as one word.
- A test pins that an `occurrence` rule counts no match that overlaps a code span, with a token that also reads Vale's mask.
- The `vale-differences` spec records the second case, with Vale's counts.
- The ProseStyle rule moves to a form hakkiri counts, in the prose-style package, as the last task describes.
- `code_span_size` sets how many words a code span counts as, in the package's settings file, the project config and a glob, with the flag `--code-span-size` and the variable `HAKKIRI_CODE_SPAN_SIZE`.
  It is 1 by default, and 0 gives Vale's counts.
  It applies only to word counts and to the metrics built on words.

## Capabilities

### Modified Capabilities

- `vale-differences`: an `occurrence` count of a code span, and how `code_span_size` matches Vale.
- `project-config`: the key `code_span_size`, at three levels.
- `check-command`: the flag and its variable.
- `metric-check`: a word count reads a code span as `code_span_size` words.

## Impact

- Changed: `src/text.rs`, `src/summary.rs`, `src/check.rs`, `src/engine.rs`, `src/config.rs`, `src/cli.rs`, `src/files.rs`, `src/run.rs`, `tests/metric.rs`, `tests/alerts.rs`, `tests/code_span_size.rs`.
- `hakkiri::engine::lint_accepting` and `hakkiri::summary::summarize` take the size as a last argument.
- No new dependency.

[ledger]: ../../../tasks.yaml
