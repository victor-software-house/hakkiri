# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.

## 1. Probe Vale

- [x] 1.1 Build Vale `66bbb85` in a throwaway Linux box, and run it over styles that hold the cases below.  
  The cases are a dotted file name, a nested rule at two levels, two files with one name, a bracket name, a `*.test.yml` file, and folders that start with `.` and `_`.  
  Verify the rule names, the loaded rules and the errors match the scenarios of the `rule-loading` delta, and record what differs in `vale-differences`.
  - 2026-11-11 (UTC), `Terms.custom.yml` gave `Team.Terms`, and `Sub/Deep/Baz.yml` gave `Team.Sub.Deep.Baz`.  
    `Terms.yml` with `Terms.custom.yml`, and a bracket name, each failed with error E100 in an enabled style.  
    Neither failed when no `BasedOnStyles` named the style, and Vale linted with exit 0.  
    `vale test --coverage` named a nested rule `Team.Sub.Deep.Bar` and left a rule under `_draft` out of the coverage.

## 2. Name and load

- [x] 2.1 Cut a rule name at the first dot, join the folders under the style, refuse a bracket, and walk every folder under a style as Vale does, in `src/rule.rs`.  
  Verify the `check` tests for a dotted name, a nested rule and a bracket fail on the old loader and pass on the new one.
  - 2026-11-11 (UTC), the three tests failed on the loader of [de3e392][c-base] and pass in [a1e07da][c-code], in a throwaway Linux box.
- [x] 2.2 Refuse two files that give one rule name, with an error that names the rule and both files.  
  Verify the clash test fails on the old loader and passes on the new one.
  - 2026-11-11 (UTC), the same box run shows it failing on the old loader and passing on [a1e07da][c-code].
- [x] 2.3 Name a rule in `test` and in `test --coverage` through the same function, with the style found from the packages of the config.  
  Verify the dotted and nested coverage tests fail on the old loader and pass on the new one.
  - 2026-11-11 (UTC), both `test_command` tests failed on the old loader and pass on [a1e07da][c-code].  
    The dotted test now holds `absent: Team.Foo.v2`, since `contains: Team.Foo` also matched the old name.
- [x] 2.4 Refuse a name clash or a bracket only for a style a glob lists, as Vale does for an enabled style, and lint an unused style with no error.  
  Verify the `check` tests for an unused style with a clash and with a bracket fail on the loader of [3e368a0][c-prior] and pass on the new one, and the tests for a listed style still pass.
  - 2026-11-11 (UTC), both unused-style tests failed with exit status 2 on that loader, and pass in [eae1653][c-lazy], in a throwaway Linux box.
- [x] 2.5 Find the package of a rule as the folder above it whose real path is one of the config's, and list a name that two rule files share once, with both files.  
  Verify the tests for a linked project root and for an uncovered shared name fail on the loader of [ffba7c2][c-gap] and pass on the new one.  
  The tests for a covered shared name and for a clash in a style a later glob lists pass on both.
  - 2026-11-11 (UTC), the linked-root test failed with `uncovered Sub.Bar` and the shared-name test listed `Team.Terms` twice on that loader, in a throwaway Linux box.  
    Both pass in [a19b4de][c-link].  
    Vale `66bbb85` gave no error for two files with one name in `vale test`, and listed an unflagged name twice.
- [x] 2.6 Print an uncovered rule in JSON as an object with the rule name, and with `files` only when two rule files give the name.  
  Verify the JSON test fails on the loader of [f7a7742][c-shared] and passes on the new one.  
  - 2026-11-11 (UTC), it failed on the string entry in a throwaway Linux box, and passes in [97241a8][c-json].

## 3. Record

- [x] 3.1 Update `src/instructions.md`, the `rule-loading` spec, and the unarchived `test-command` change.  
  Verify `openspec validate --all --strict` and the budget check pass.
  - 2026-11-11 (UTC), both pass on the Mac for the docs of [eeafcb0][c-docs] with two sentences split.

## 4. Integration

- [x] 4.1 Verify the pull request's CI passes: build, lint, tests, prose lint, OpenSpec and the Template check.
  - 2026-11-11 (UTC), CI and the Template check passed on the head of [PR 60][pr60], [58f7975][c-head], in [run 42947844816][ci60].  
    Main CI passed after the merge, on [4979c2a][c-main], in [run 42948574268][ci-main].

## Workflow follow-up

- Archive this change after the pull request merges, then archive `HKR-032` with `qctl close-from-openspec`.

[c-base]: https://github.com/victor-software-house/hakkiri/commit/de3e392ed578ede50c86d753ec7c1e6114dbc9ad
[c-code]: https://github.com/victor-software-house/hakkiri/commit/a1e07daf0072c127692d6f8979263bbde2e7b338
[c-prior]: https://github.com/victor-software-house/hakkiri/commit/3e368a0cf0793030a21d33e10c884956f7a72bd1
[c-lazy]: https://github.com/victor-software-house/hakkiri/commit/eae16539ebdb1c6b9b7e1123a2ac57ac951d17a2
[c-docs]: https://github.com/victor-software-house/hakkiri/commit/eeafcb035627fee88eea11feb2232f123292868a
[c-gap]: https://github.com/victor-software-house/hakkiri/commit/ffba7c2fb7e099cafd2c1117ec3f350f39651678
[c-link]: https://github.com/victor-software-house/hakkiri/commit/a19b4de4b4f5773413d7483d8a78daf6b5baacf9
[c-shared]: https://github.com/victor-software-house/hakkiri/commit/f7a77425a0a0c1a735458c342e46764c5597355b
[c-json]: https://github.com/victor-software-house/hakkiri/commit/97241a82f24b89ef208e30792a6865f603a78cf1
[pr60]: https://github.com/victor-software-house/hakkiri/pull/60
[c-head]: https://github.com/victor-software-house/hakkiri/commit/58f79758ba9973b4e31025cba8a989f5355124c6
[ci60]: https://github.com/victor-software-house/hakkiri/actions/runs/42947844816
[c-main]: https://github.com/victor-software-house/hakkiri/commit/4979c2aab5de774e24e5684cd9553720733588a0
[ci-main]: https://github.com/victor-software-house/hakkiri/actions/runs/42948574268
