# Proposal

Queue row: `HKR-041` in [tasks.yaml][ledger].
It follows [HKR-019][vocabularies] and [HKR-042][vocabulary-rules], which read `accept.txt` and `reject.txt` only.

## Motivation

Vale reads a vocabulary from `accept.txt`, `reject.txt`, `vocab.yml` and `vocab.yaml` in its folder ([vocab.go][v-vocab], [loadVocabulary][v-load]).
A YAML entry is a term, a term mapped to nothing, or a term mapped to the options `regex`, `ignorecase` and `flags`.
hakkiri skipped the YAML files, so a package that moves to them lost its accepted and rejected terms, and the alerts they held back came back.

## Changes

- A vocabulary folder may hold `vocab.yml` and `vocab.yaml`, besides the text lists, and hakkiri reads them all.
- The file holds `affix`, `accept` and `reject`.
  An entry is a term, a term mapped to nothing, or a term mapped to `regex`, `ignorecase` and `flags`.
- A plain term is literal.
  `regex: true` keeps a pattern, `flags` expands the term by the affix flags of a dictionary, and `ignorecase` puts `(?i)` before each pattern.
- `affix` names an affix file in the `config/dictionaries` folder of the package that holds the vocabulary, then of every other loaded package in order, and never in `$DICPATH`.
- The file parses through ctl-core's input, which refuses a key the file does not define.
  Every wrong entry is reported with its file and line, where Vale stops at the first.
- The `vale-differences` spec records the five places where hakkiri differs, and drops the gap for YAML vocabularies.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `vocabularies`: a vocabulary folder holds YAML files, and how their entries read.
- `vale-differences`: the gap closes, and its five differences are listed.

## Impact

- New: `src/vocabulary/yaml.rs`, `tests/yaml_vocabularies.rs` and its fixtures, with Vale's alerts and error lines recorded from Vale at `66bbb85`.
- Changed: `src/vocabulary.rs`, the config loader's call to it, `src/instructions.md`, the skill, `README.md` and `AGENTS.md`.
- No new dependency.

[ledger]: ../../../tasks.yaml
[v-load]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/ini.go#L106-L154
[v-vocab]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/vocab.go
[vocabularies]: ../archive/2026-11-11-vocabularies/proposal.md
[vocabulary-rules]: ../archive/2026-11-11-vocabulary-rules/proposal.md
