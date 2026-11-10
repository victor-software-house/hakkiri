# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.  
Only the binary size measurement in task 1.2 runs on a box, because CI cannot.

## 1. Dependencies and the chassis

- [ ] 1.1 Add `clap`, `anyhow` and `globset`, `serde_json` as a dev dependency, and ctl-core's `app` feature, with `fixture` in dev.  
  Verify CI passes with `cargo deny` and `cargo machete` clean.
- [ ] 1.2 Measure the release binary before and after, and state both sizes in the pull request.  
  Verify the numbers are in the pull request description, taken from the same box and profile.
- [ ] 1.3 Put `#[command(env_prefix = "HAKKIRI")]` on the root parser, and add the `check` and `instructions` subcommands.  
  Verify a test sets `HAKKIRI_FAIL_LEVEL` and that a flag wins over it.

## 2. The project config

- [ ] 2.1 Read `.ctl/hakkiri.yaml`, or `--config`, through ctl-core's `input` reader, and refuse an unknown key with its line.  
  Verify the scenarios of the `project-config` spec for the file, the unknown key and `level` pass as integration tests.
- [ ] 2.2 Read `packages` entries with a `path`, load each style folder, and refuse an entry with both `path` and `url`, and two folders that hold one style name.  
  Verify the three local package scenarios (a local folder, an entry with both, one style in two folders) and the config-from-another-folder scenario pass, and that `HAKKIRI_CONFIG` names a config.  
  The two download scenarios (a matching package, one that does not match) need `HKR-006` and are covered there.
- [ ] 2.3 Match `files` globs with `globset`, apply every matching glob in order, and read `styles`, rule levels and `off`.  
  Verify the scenarios for two globs, an unmatched file, an empty `styles` and a misspelled rule name pass.
- [ ] 2.4 Resolve the project root, match globs and read package paths from it, and warn on a path outside it.  
  Verify these scenario groups pass.
  - A dot-slash, an absolute and a parent argument.
  - A root reached through a symlink, and a symlink to a file.
  - A path outside the root with its warning, no path under the root, and a config elsewhere.
  - A walk from another folder that skips the package folders.

## 3. Finding and linting files

- [ ] 3.1 Walk directories as design decision 2 says, and read languages by extension as decision 5 says.  
  Verify the scenarios for a directory and a file, no argument, the skipped folders, and both plain-text cases pass.
- [ ] 3.2 Lint files on a pool of threads, keep each file's alerts with its path, and sort the report.  
  Verify a run over 200 generated files prints the same text twice, and that one broken MDX file reports a failure and still lints the rest.

## 4. Output and exit status

- [ ] 4.1 Build one result type that serializes to the JSON object and presents as the line form, with the count line.  
  Verify the scenarios for a warning, a clean run and the JSON pass, and that a test removes `CLICOLOR`, `CLICOLOR_FORCE`, `FORCE_COLOR`, `NO_COLOR` and `COLORTERM` from the child through one helper.
- [ ] 4.2 Add `--fail-level` and the exit codes 0, 1 and 2.  
  Verify the five exit scenarios pass, with a test that runs the binary for each code.
- [ ] 4.3 Run the binary with `CLICOLOR_FORCE=1`, `FORCE_COLOR=3` and `COLORTERM=truecolor`, and read its JSON for an ESC byte.  
  Verify the output holds none, with the same set for a parse error whose bad argument comes before `--format json`.

## 5. The bundle

- [ ] 5.1 Write `src/instructions.md` and `skills/hakkiri/SKILL.md`, and print the first from `hakkiri instructions`.  
  Verify a test compares the command's output with the file, and the skill names every flag, variable and exit code.
- [ ] 5.2 Update `README.md` and `AGENTS.md`: the status, the command, and the decision that `check` exits as Vale does.  
  Verify a grep for "Planned, not usable yet" and "fn main() {}" finds nothing.
- [ ] 5.3 Add the `vale-differences` requirement and the plain-text gap, and queue `HKR-030`.  
  Verify `openspec validate --all --strict` and `qctl check` pass.

## 6. Integration

- [ ] 6.1 Verify the pull request's CI passes: build, lint, tests, docs, deny, prose lint and OpenSpec.

## Workflow follow-up

- Archive this change after the pull request merges, then archive `HKR-005` with `qctl close-from-openspec`.
