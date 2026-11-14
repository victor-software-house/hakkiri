# Proposal

Queue row: `HKR-071` in [tasks.yaml][ledger].

## Motivation

With the settings cascade of [settings-cascade][settings-cascade] (row `HKR-069`), the most specific glob wins for a key.  
Two parts remain: what breaks a tie between a package and the project, and how the project keeps a value that a more specific package glob would replace.

The Kody thread on pull request 141 found that a package glob `'*.md': {code_span_size: 0}` beats the project's top-level `code_span_size: 2`, and the project had no way to keep its value.  
The operator ruled on 2026-11-13 that the project breaks a tie with a package, as an author's stylesheet beats a library's in CSS.  
The project may force a value with the YAML tag `!override`, the spelling of the Compose specification's merge tag, and a forced value beats every unforced one.

## Changes

- At equal specificity a project glob beats a package glob, and a later package beats an earlier one. The top level ranks below every glob.
- A value tagged `!override` in the project's config applies after every value without the tag, in rank order among themselves.  
  A package's settings file that holds the tag is an error, and so is any other tag.
- The lists of `vocab`, `block_ignores` and `token_ignores` add up over every matching glob; a forced list replaces what the unforced ones gave.
- The `project-config` and `vale-differences` specs state both.

## Capabilities

### Modified Capabilities

- `project-config`: the origin of a setting and the forced values.
- `vale-differences`: a package glob and the project's top level, and a forced value.

## Impact

- Changed: `src/config.rs`, the config tests, `src/instructions.md`, the skill and `AGENTS.md`, in the same pull request as `settings-cascade`.
- ctl-core 0.7.5 re-exports serde-saphyr's `Tagged<T>`, so a setting keeps its tag beside its value.  
  hakkiri moves its ctl-core pin to that release.
- The design is sections 2 to 4 of the [settings-cascade design][design].

[ledger]: ../../../../tasks.yaml
[settings-cascade]: ../2026-11-14-settings-cascade/proposal.md
[design]: ../2026-11-14-settings-cascade/design.md
