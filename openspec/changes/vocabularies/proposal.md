# Proposal

Queue row: `HKR-019` in [tasks.yaml][ledger].
It adds the rows `HKR-040`, `HKR-041` and `HKR-042` for what it leaves out, and `HKR-042` has its own change, [vocabulary-rules][rules].

## Motivation

A Vale package ships a vocabulary: `ProseStyle` holds `styles/config/vocabularies/ProseStyle/accept.txt`, and its `.vale.ini` sets `Vocab = ProseStyle` ([prose-style][prose-style-ini]).
Vale reads the accepted words as exceptions in most checks.
hakkiri skips `config`, loads a rule's `vocab` field and does nothing with it, so a project that moves from Vale gets alerts on words its vocabulary accepts.
The [vale-differences spec][differences] lists this as a known gap.

## Changes

- The config names the vocabularies a project uses, and hakkiri reads each from `config/vocabularies/<name>/` in a loaded package: every `accept.txt` and `reject.txt` below it.
- A line of a list is one term, which is a pattern, with Vale's rule for a plain term.
- Accepted terms hold back matches in `capitalization`, `conditional`, `existence`, `repetition`, `substitution` and `spelling` rules, as in Vale, and `vocab: false` turns that off where Vale lets it.
- An unknown vocabulary, two packages that hold one name and a term that is not a pattern are errors that name the file or line.
- The `vale-differences` spec records what still differs and lists three gaps with their rows.
- The config key that names vocabularies waits for the operator's choice: [design decision 2][key].

## Capabilities

### New Capabilities

- `vocabularies`: how a vocabulary is found and read, and which checks its accepted terms reach.

### Modified Capabilities

- `project-config`: the config names vocabularies, and a package may hold a vocabulary folder.
- `vale-differences`: the vocabularies gap narrows, and its differences and three gaps are listed.

## Impact

- New: `src/vocabulary.rs`, `tests/vocabularies.rs` and its fixtures, with Vale's alerts for them recorded from Vale at `66bbb85`.
- Changed: the rule builders in `src/rule/build.rs`, `src/strcase.rs`, the config loader, `src/instructions.md`, the skill, `AGENTS.md` and `README.md`.
- No new dependency.

[ledger]: ../../../tasks.yaml
[rules]: ../vocabulary-rules/proposal.md
[differences]: ../../specs/vale-differences/spec.md
[key]: design.md#2-the-config-key-that-names-vocabularies-is-open
[prose-style-ini]: https://github.com/victor-software-house/prose-style/blob/90c7c8e673b92f836cce257427724fc79da997e8/ProseStyle/.vale.ini
