# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.  
Only the binary size measurement in task 1.2 runs on a box, because CI cannot.

## 1. Dependencies and the chassis

- [x] 1.1 Add `clap`, `anyhow` and `globset`, `serde_json` as a dev dependency, and ctl-core's `app` feature, with `fixture` in dev.  
  Verify CI passes with `cargo deny` and `cargo machete` clean.  
  Evidence, 2026-11-10 (UTC): `deny` reported bans, licenses and sources ok, and `cargo-machete` found no unused dependency in [the CI run][ci52].
- [x] 1.2 Measure the release binary before and after, and state both sizes in the pull request.  
  Verify the numbers are in the pull request description, taken from the same box and profile.  
  Evidence, 2026-11-10 (UTC): the [PR 52][pr52] description holds the four sizes in bytes.  
  No engine: 278,752.
  Engine and every grammar: 6,564,008.
  With the rule loader: 11,743,664.
  The CLI: 12,329,296.  
  All four were built on one box with the release profile.
- [x] 1.3 Put `#[command(env_prefix = "HAKKIRI")]` on the root parser, and add the `check` and `instructions` subcommands.  
  Verify a test sets `HAKKIRI_FAIL_LEVEL` and that a flag wins over it.  
  Evidence, 2026-11-10 (UTC): `a_variable_sets_a_flag_and_the_flag_wins` is one of the 41 `check` tests that pass in [the CI run][ci52].

## 2. The project config

- [x] 2.1 Read `.ctl/hakkiri.yaml`, or `--config`, through ctl-core's `input` reader, and refuse an unknown key with its line.  
  Verify the scenarios of the `project-config` spec for the file, the unknown key and `level` pass as integration tests.  
  Evidence, 2026-11-10 (UTC): `a_broken_config_stops_the_run` (unknown key, line 3), `a_missing_config_stops_the_run` and `a_level_hides_the_lower_alerts` pass in [the CI run][ci52].
- [x] 2.2 Read `packages` entries with a `path`, load each style folder, and refuse an entry with both `path` and `url`, and two folders that hold one style name.  
  Verify the three local package scenarios (a local folder, an entry with both, one style in two folders) and the config-from-another-folder scenario pass, and that `HAKKIRI_CONFIG` names a config.  
  The two download scenarios (a matching package, one that does not match) need `HKR-006` and are covered there.  
  Evidence, 2026-11-10 (UTC): `a_package_path_is_read_from_the_root`, `a_package_entry_with_a_path_and_a_url_is_refused`, `one_style_in_two_folders_is_refused` and `the_variable_names_the_config` pass in [the CI run][ci52].  
  The two download scenarios wait for `HKR-006`.
- [x] 2.3 Match `files` globs with `globset`, apply every matching glob in order, and read `styles`, rule levels and `off`.  
  Verify the scenarios for two globs, an unmatched file, an empty `styles` and a misspelled rule name pass.  
  Evidence, 2026-11-10 (UTC): `two_globs_apply_in_order`, `a_file_no_glob_matches_is_not_linted`, `an_empty_styles_list_skips_the_files` and `a_misspelled_rule_name_is_refused` pass in [the CI run][ci52].
- [x] 2.4 Resolve the project root, match globs and read package paths from it, and warn on a path outside it.  
  Verify these scenario groups pass.  
  - A dot-slash, an absolute and a parent argument.  
  - A root reached through a symlink, and a symlink to a file.  
  - A path outside the root with its warning, no path under the root, and a config elsewhere.  
  - A walk from another folder that skips the package folders.  
  - A file named two ways, which is linted once.  
  - A walk from a folder that holds the root, and a path that does not exist.  
  - The warning in JSON and quiet runs, and a mixed run.  
  - A missing package folder, and a walk past a symlink to a package folder.  
  Evidence, 2026-11-10 (UTC): The tests for each group pass in [the CI run][ci52], from `a_dot_slash_argument_matches_the_glob` to `a_walk_does_not_enter_a_symlink_to_a_package_folder`, and the `project-root` spec holds the scenarios.  
  The case-insensitive filesystem case has no test, because CI runs on a case-sensitive filesystem.

## 3. Finding and linting files

- [x] 3.1 Walk directories as design decision 2 says, and read languages by extension as decision 5 says.  
  Verify the scenarios for a directory and a file, no argument, the skipped folders, and both plain-text cases pass.  
  Evidence, 2026-11-10 (UTC): `a_directory_and_a_file`, `no_argument_lints_the_current_directory`, `folders_a_walk_leaves_alone` and `a_file_no_language_reads` pass in [the CI run][ci52].
- [x] 3.2 Lint files on a pool of threads, keep each file's alerts with its path, and sort the report.  
  Verify a run over 200 generated files prints the same text twice, and that one broken MDX file reports a failure and still lints the rest.  
  Evidence, 2026-11-10 (UTC): `a_run_over_many_files_prints_the_same_text_twice` and `one_broken_mdx_file` pass in [the CI run][ci52].

## 4. Output and exit status

- [x] 4.1 Build one result type that serializes to the JSON object and presents as the line form, with the count line.  
  Verify the scenarios for a warning, a clean run and the JSON pass, and that a test removes `CLICOLOR`, `CLICOLOR_FORCE`, `FORCE_COLOR`, `NO_COLOR` and `COLORTERM` from the child through one helper.  
  Evidence, 2026-11-10 (UTC): `a_directory_and_a_file`, `a_clean_run` and the JSON tests pass in [the CI run][ci52], and every test starts the binary through one `binary()` helper that removes the five colour variables.
- [x] 4.2 Add `--fail-level` and the exit codes 0, 1 and 2.  
  Verify the five exit scenarios pass, with a test that runs the binary for each code.  
  Evidence, 2026-11-10 (UTC): `the_exit_status_follows_the_fail_level`, `a_failure_wins_over_an_alert`, `an_alert_the_config_hides_does_not_fail`, `a_broken_config_stops_the_run` and `a_clean_run` run the binary for exit 0, 1 and 2 and pass in [the CI run][ci52].
- [x] 4.3 Run the binary with `CLICOLOR_FORCE=1`, `FORCE_COLOR=3` and `COLORTERM=truecolor`, and read its JSON for an ESC byte.  
  Verify the output holds none, with the same set for a parse error whose bad argument comes before `--format json`.  
  Evidence, 2026-11-10 (UTC): `json_holds_no_escape_byte_under_forced_colour` covers the JSON and the `--bogus` parse error before `--format json`, and passes in [the CI run][ci52].

## 5. The bundle

- [x] 5.1 Write `src/instructions.md` and `skills/hakkiri/SKILL.md`, and print the first from `hakkiri instructions`.  
  Verify a test compares the command's output with the file, and the skill names every flag, variable and exit code.  
  Evidence, 2026-11-10 (UTC): `the_command_prints_the_instructions_file` and `both_documents_name_every_flag_variable_and_exit_code` pass in [the CI run][ci52].
- [x] 5.2 Update `README.md` and `AGENTS.md`: the status, the command, and the decision that `check` exits as Vale does.  
  Verify a grep for "Planned, not usable yet" and "fn main() {}" finds nothing.  
  Evidence, 2026-11-10 (UTC): `rg` finds neither phrase in `README.md` or `AGENTS.md` at [the merge][merge].  
  Both still appear in a proposal, a test fixture and a spec example, as history and sample text, not as status.
- [x] 5.3 Add the `vale-differences` requirement and the plain-text gap, and queue `HKR-030`.  
  Verify `openspec validate --all --strict` and `qctl check` pass.  
  Evidence, 2026-11-10 (UTC): The `specs` task validates every spec and change in [the CI run][ci52] (19 passed, including `change/check-command`).  
  `qctl check` and `openspec validate --all --strict` exited 0 on a clean checkout of [9b0ff1d][c9b0ff1], not in that run's log.

## 6. Integration

- [x] 6.1 Verify the pull request's CI passes: build, lint, tests, docs, deny, prose lint and OpenSpec.  
  Evidence, 2026-11-10 (UTC): [The CI run][ci52] on [9b0ff1d][c9b0ff1] passed Verify (Build and test, Static checks), Secrets, Template and PR template.

## Workflow follow-up

- Archive this change after the pull request merges, then archive `HKR-005` with `qctl close-from-openspec`.

[pr52]: https://github.com/victor-software-house/hakkiri/pull/52
[ci52]: https://github.com/victor-software-house/hakkiri/actions/runs/42927031780
[c9b0ff1]: https://github.com/victor-software-house/hakkiri/commit/9b0ff1dca1c1ea69c3ca8106dfc9e7c3d8173eeb
[merge]: https://github.com/victor-software-house/hakkiri/commit/0e14935
