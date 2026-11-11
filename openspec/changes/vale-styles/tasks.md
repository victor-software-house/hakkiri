# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.

## 1. Measure and probe

- [x] 1.1 Run hakkiri on main over the pinned Std, Voices and Readability styles as they are, and Vale `66bbb85` over 13 chains of inheritance.  
  Verify which rules fail to load, and that Vale's overlay and refusals match the scenarios of the `rule-loading` delta.
  - 2026-11-11 (UTC), Std loaded with all 29 cases passing, and Voices with 66 passing and four rules failing: `Direct.Length` and three GenZ rules, which inherit.  
    Readability refused as a whole style, for its one script rule.
  - The 13 Vale chains showed `+` adding, `-` removing, a bare key replacing, a parent in a `_` or a nested folder resolving, and six refusals that end the run with exit 2.

## 2. Load

- [x] 2.1 Resolve `extends: Style.Rule` in the loader, with the overlay and the refusals of Vale, in `src/rule/inherit.rs` and `src/rule.rs`.  
  Verify the inheritance tests fail on the loader of main and pass on the new one.
  - 2026-11-11 (UTC), the 22 tests of `tests/vale_styles.rs` failed on `src/` of [fe3eea0][c-base] (0 passed), in a throwaway Linux box.  
    All 22 pass in [b928e2e][c-code].
- [x] 2.2 Report a rule hakkiri cannot run apart, run the rest of its style, fail the run for it until the config turns it `off`, and report its cases as not run, in `src/config.rs`, `src/run.rs` and `src/suite.rs`.  
  Verify the tests for a listed style, an unlisted style, `off`, a level and `on`, and `test` fail before and pass after.
  - 2026-11-11 (UTC), those tests are among the 22 above, and fail and pass with them.
- [x] 2.3 Commit the Voices, Std and Readability styles with their licences and the tag, and run them.  
  Verify `hakkiri test --coverage` passes every case of Voices and Std, and Readability runs with its script rule turned off.
  - 2026-11-11 (UTC), 103 cases in 48 files pass with no uncovered rule, and Readability fails once for `Readability.Polysyllables` until it is `off`, in [b928e2e][c-code].
- [x] 2.4 Load a fetched package that holds inheriting rules.  
  Verify a test pins ProseStyle v0.0.14 by hash, serves it from a local port, and lints with `Direct.Length`, and that its six script rules fail the run until they are `off`.
  - 2026-11-11 (UTC), both tests of `tests/package_fetch.rs` failed on `src/` of [06feef5][c-pkg], and pass in [ace5b1a][c-fetch].  
    The first run failed on the search path: the loader looked for `Std` only in packages written as a `path`, so the fixture found the fault.
- [x] 2.5 Gate in a throwaway Linux box.  
  Verify test, lint, format, doc, deny and deps pass.
  - 2026-11-11 (UTC), all six pass on the tree of [ace5b1a][c-fetch], rebased onto [06feef5][c-pkg], with 29 test targets passing.

## 3. Record

- [x] 3.1 Update the specs through the deltas, list each unrun Vale feature as a Known gap with its row, and update `AGENTS.md`, `src/instructions.md` and the skill.  
  Verify `openspec validate --all --strict` and the budget check pass.
  - 2026-11-11 (UTC), both pass on the Mac.
- [x] 3.2 Rewrite `HKR-007` and add the rows `HKR-035` to `HKR-039` with `qctl`.  
  Verify `qctl check` passes.
  - 2026-11-11 (UTC), `qctl check` passes.

## 4. Gate

- [ ] 4.1 Pull request CI passes on the head of the pull request.  
  Verify the checks of the head commit pass in the run on the pull request.

[c-base]: https://github.com/victor-software-house/hakkiri/commit/fe3eea00f56d4831ebf3ffc5341b30570e771c1a
[c-code]: https://github.com/victor-software-house/hakkiri/commit/b928e2ea08e61b3273a5838fb8e440a6cb80884a
[c-pkg]: https://github.com/victor-software-house/hakkiri/commit/06feef55308c2b6178e93942820b0f73da1904e4
[c-fetch]: https://github.com/victor-software-house/hakkiri/commit/ace5b1a7129121f5d52abf43a2b45f4439bdcbc4
