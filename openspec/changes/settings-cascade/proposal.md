# Proposal

Queue row: `HKR-069` in [tasks.yaml][ledger].

## Motivation

hakkiri applies the globs that match a file in the order written, packages first, and a later glob wins for the same key, as Vale applies its sections ([file.go][vale-file]).  
So a package glob `'*.md': {code_span_size: 0}` beats the project's top-level `code_span_size: 2`, and a project glob loses to any package glob written after it in the same rank.  
The Kody thread on pull request 141 found the first case.  
The operator ruled on 2026-11-13 that settings resolve as CSS rules do: every setting that matches a file merges, and only settings of the same key compete.  
For one key the most specific glob wins, the project breaks a tie with a package, and the later glob breaks the rest.  
The project may force a value with the YAML tag `!override`, the spelling of the Compose specification's merge tag, and a forced value beats every unforced one.

## Changes

- Each glob has a specificity, read from its text, and the globs that match a file apply from the least specific to the most, so the more specific wins for the same key.  
  The top level ranks below every glob.
- At equal specificity a project glob beats a package glob, a later package beats an earlier one, and a later glob beats an earlier one in the same file.
- A value tagged `!override` in the project's config applies after every value without the tag, in the same order among themselves.  
  A package's settings file that holds the tag is an error, and so is any other tag.
- The lists of `vocab`, `block_ignores` and `token_ignores` still add up over every matching glob; a forced list replaces what the unforced ones gave.
- Turning a rule on or off and its level resolve as two keys, so a more specific `off` keeps a level set by a less specific glob, as a later `off` keeps it today.
- An empty `styles` list drops the rule settings of the globs that rank below it.
- The `project-config` and `vale-differences` specs state the cascade, and the open `code-span-size` and `rule-parameters` changes drop their written-order wording.

## Capabilities

### Modified Capabilities

- `project-config`: the order in which the settings of the matching globs apply, and the forced values.
- `vale-differences`: hakkiri ranks the matching globs by specificity, where Vale applies its sections in the order written.

## Impact

- Changed: `src/config.rs`, the config tests, `src/instructions.md`, the skill and `AGENTS.md`.
- ctl-core 0.7.5 re-exports serde-saphyr's `Tagged<T>`, which reads the tag of a setting beside its value; hakkiri moves its ctl-core pin to it.
- A config whose globs overlap can resolve differently: a more specific glob now wins wherever it is written.  
  The scenarios that showed written order are rewritten to show the cascade.

[ledger]: ../../../tasks.yaml
[vale-file]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/file.go#L185-L215
