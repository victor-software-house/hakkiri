# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.  
This change follows [`project-config`][config], which holds the grammar of a `url` entry.

## 1. The fetch

- [x] 1.1 Add `src/package.rs`.  
  It fetches a `url` with a timeout and the 32 MiB limit, checks the SHA-256, unpacks into a temporary folder with the checks of design decision 3, finds `styles`, and renames the folder to `<cache>/<sha256>/`.  
  Verify `tests/package_fetch.rs` fails before the change and passes after it.  
  Its tests cover the first fetch, the cache hit, the offline hit and the offline miss.  
  They also cover the hash mismatch, a 404, a redirect, a body that is not a zip, a name that leaves the package, a zip with no top folder, a style-only package and a body over the limit.  
  Evidence, 2026-11-11 (UTC): in `ubuntu:24.04` in a box, the 17 tests failed before the change, each for the reason it names, and passed after it.  
  [The CI run of PR 75][ci-pr75] passed with them.
- [x] 1.2 Load a `url` entry in `load_packages` as the package folder its zip lays out, and check the `url` scheme and the `sha256` shape in `entry_ok`.  
  Verify the scenarios for a hash of the wrong shape and for a url that is not http pass, and the existing refusals of an entry with both and of a `url` with no `sha256` still pass.  
  Evidence, 2026-11-11 (UTC): the scenarios for a hash of the wrong shape and for a url that is not http pass in the same box run.  
  The full suite, 529 tests, passed there, so the refusals of an entry with both and of a `url` with no `sha256` still pass.
- [x] 1.3 Add `--cache-dir` and `HAKKIRI_CACHE_DIR` to `check` and `test`, with the per-user default.  
  Verify the scenarios for the variable, the option and the default folder pass, and `hakkiri test` loads a fetched package.  
  Evidence, 2026-11-11 (UTC): `the_cache_folder_defaults_to_the_per_user_folder`, `the_cache_dir_option_moves_the_cache` and `hakkiri_test_loads_the_fetched_package_for_a_case_through_the_config` pass in [the CI run of PR 75][ci-pr75].
- [x] 1.4 Skip a folder named `config` when listing a package's styles, as Vale does, and bound a download by its lookup, connect, header and body phases, not by one global timeout.  
  Verify the scenario for a folder of settings passes, and a 32 MiB body has 300 seconds to arrive.  
  Evidence, 2026-11-11 (UTC): Kody's two findings on PR 75 led to this task.  
  In a box, the two new tests for a folder of settings failed before the change, one with ``the style `config` is in both`` and one with no refusal of a `config` style, and passed after it, in [commit 4f3fe9f][commit-4f3fe9f].  
  Not verified: a body that takes 300 seconds, because a test of it needs the real wait.
- [x] 1.5 Unpack each fetch into a folder it creates under a unique name and removes only that folder, so runs that share a cache and a process id cannot wipe each other.  
  Verify `package::tests::two_installs_of_one_package_at_once_leave_one_complete_folder` fails before the change and passes after it.  
  Evidence, 2026-11-11 (UTC): Kody's finding on the staging folder led to this task.  
  In a box, the test failed before the change with "No such file or directory", where one run's folder was removed by the other, and passed after it, with the full suite at 556 tests.  
  The fix landed in [commit 83609b6][commit-83609b6] on PR 76.

## 2. The dependencies

- [x] 2.1 Add `ureq`, `zip` and `sha2` with their licences in `deny.toml`, and check the musl binary grows by what design decision 5 says.  
  Verify `cargo deny` passes licenses, bans and sources, `cargo tree --target aarch64-apple-darwin` still builds, and the measured size is in the pull request.  
  Evidence, 2026-11-11 (UTC): `cargo deny` passed licenses, bans, sources and advisories in a box.  
  `cargo tree --target aarch64-apple-darwin` lists `ureq` 3.5.1, `zip` 9.5.1 and `sha2` 0.11.1 for hakkiri.  
  The musl release build grew from 12,781,136 to 14,186,128 bytes, +1.41 MB, and the table in the design holds the other candidates.  
  Not verified: a build for `aarch64-apple-darwin`, which the release job on `macos-latest` makes.

## 3. The surfaces

- [x] 3.1 Update `src/instructions.md`, `skills/hakkiri/SKILL.md`, `AGENTS.md` and the `vale-differences` spec for the fetched package, and add the patch changeset.  
  Verify `operator_docs` passes, `hakkiri instructions` no longer says a `url` entry is not downloaded, and the changeset names `hakkiri: patch`.  
  Evidence, 2026-11-11 (UTC): `operator_docs` passes, no operator document holds the line that a `url` entry is not downloaded, and the changeset names `hakkiri: patch`.  
  A later commit added the `.unpacking-` folder note to `src/instructions.md` and the skill, after the pull request.

## 4. Integration

- [x] 4.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.  
  Evidence, 2026-11-11 (UTC): [the CI run of PR 75][ci-pr75] and [the CI run of PR 76][ci-pr76] passed, with the Template check.  
  [The CI run on main][ci-main] after the merge passed too.

[config]: ../../../specs/project-config/spec.md
[ci-pr75]: https://github.com/victor-software-house/hakkiri/actions/runs/43030640283
[ci-main]: https://github.com/victor-software-house/hakkiri/actions/runs/43036488965
[ci-pr76]: https://github.com/victor-software-house/hakkiri/actions/runs/43035312799
[commit-4f3fe9f]: https://github.com/victor-software-house/hakkiri/pull/75/commits/4f3fe9f41e9eff3931aca8471201c2bce3b9dac2
[commit-83609b6]: https://github.com/victor-software-house/hakkiri/pull/76/commits/83609b6ac9539e9e310b22a69b21fa9fb5fe6e8a
