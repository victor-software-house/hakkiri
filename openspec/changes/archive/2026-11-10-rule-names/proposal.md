# Proposal

Queue row: `HKR-032` in [tasks.yaml][ledger].

## Motivation

hakkiri names a rule file differently from Vale, and `check` and `test` can name one file differently.
Vale cuts a file name at its first dot, so `Terms.custom.yml` in `Team` is `Team.Terms`, and hakkiri cuts at the last dot, so it is `Team.Terms.custom`.
Vale loads the folders under a style and joins them into the name, so `Team/Sub/Bar.yml` is `Team.Sub.Bar`, and hakkiri reads only the files directly in the folder.
`test` names a nested file by its own folder, `Sub.Bar`, where `check` would name it `Team.Sub.Bar` once nested folders load ([review thread][thread]).
A style written for Vale gives other rule names in hakkiri, so a setting such as `Team.Terms = NO` misses its rule.
The operator decided hakkiri names rules as Vale does (operator, 2026-11-10).

## Changes

- A rule is named by its style folder, the folders under it, and its file name up to the first dot, joined by dots.
- The loader reads every `.yml` file under a style folder, and skips a folder that starts with `.` or `_` and a `*.test.yml` file.
- It refuses two files that give one name with an error that names both, when a glob lists the style.
- `check`, `test` and `test --coverage` name a rule through one function, and `test` finds the style folder from the packages of the config, as Vale finds it from the `StylesPath`.
- A name with a bracket is refused under the same condition, as Vale refuses it.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `rule-loading`: the files a style folder gives, and the name each has.
- `vale-differences`: hakkiri refuses a clash or a bracket in any style it loads, and Vale refuses it for an enabled style.

## Impact

- `src/rule.rs` (`rule_name`, `load_style`) and `src/suite.rs` change. No new dependency.
- The [`test-command`][test] change, now archived, names a rule as the loader does, so its design and spec change in this pull request, from `Team.Foo.v2` to `Team.Foo`.

[ledger]: ../../../../tasks.yaml
[test]: ../2026-11-10-test-command/proposal.md
[thread]: https://github.com/victor-software-house/hakkiri/pull/57#discussion_r4532307394
