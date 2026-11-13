# Proposal

Queue row: `HKR-065` in [tasks.yaml][ledger].

## Motivation

The splitter read a code span as the word `x`, so a sentence that began or ended on a code span did not split where a reader and Vale split it.  
It also kept `92 ms. Tests` as one sentence, ended a sentence at `vs.`, and read a date before a full stop as a list number.  
On 2026-11-12 these places caused most of the LongSentence and SentenceLength alerts that hakkiri gave and Vale did not, over four public repositories.  
On 2026-11-13 the ProseStyle v0.0.16 alert diff still found 75 LongSentence alerts that only hakkiri gave, 68 of them false, and the operator chose to land this change and run the diff again before the gate closes.

## Changes

- The splitter takes the byte ranges of the code spans: a code span begins a sentence after a full stop, ends one before a full stop, and is never an abbreviation or an initial.
- A unit of measure after a number ends a sentence before a capital word, with the units and prefixes listed from their sources.
- `vs.` ends no sentence, and a date is no list number.
- The `vale-differences` spec records the places where Vale's segmenter splits otherwise, each with a fixture that pins Vale 66bbb85 and hakkiri.

## Capabilities

### Modified Capabilities

- `sentence-splitting`: code spans, units, `vs.` and dates.
- `vale-differences`: a numbered heading or item, a version, a decimal or a task number before a capital, and `vs.`.

## Impact

- Changed: `src/splitter/`, `src/text.rs` and the callers that split text with code in it.
- New: `tests/sentence_differences.rs` and its fixtures.
- No new dependency and no new key.

[ledger]: ../../../tasks.yaml
