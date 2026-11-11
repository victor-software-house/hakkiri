# hakkiri

A prose linter for Markdown, MDX, and source comments, built on ctl-core.  
This file is the development contract.  
The operator contract is `hakkiri instructions` (`src/instructions.md`) and `skills/hakkiri/SKILL.md`.

## Decisions

1. **Vale rule files, own config.**
   hakkiri reads Vale rule files as they are, and its own `.ctl/hakkiri.yaml`, not `.vale.ini`.  
   A rule selects text with Vale's `scope`: its lowercase dotted names, such as `sentence` and `text.comment.doc`, and its operators `~`, `&` and lists.  
   Its checks are Vale's check types except `script`; a rate is a `metric` whose formula reads `occurrences`.  
   A Vale style folder loads with no conversion: a rule may extend another rule, as in Vale.  
   A rule hakkiri cannot run, such as a `script` rule, is reported and the rest of its style runs, until the config turns it `off`.  
   `hakkiri import vale` will convert a `.vale.ini` into the project config (row `HKR-035`).
2. **Structure first.**
   A rule selects text by its scope in the document tree; a pattern only narrows that selection.  
   A rule that needs a regular expression to find whether text sits in code, a link or a comment shows a missing scope.
3. **Language profiles.**
   Each source language has a profile that maps its comments and doc comments into the tree.  
   Every comment reads as Markdown, plain `//` comments included; TSDoc and JSDoc keep their tags.  
   String literals are never prose.  
   The first release ships Rust, TypeScript, JavaScript, Markdown and MDX.
4. **Matching.**
   The `regex` crate by default; `fancy-regex` with a backtrack limit only for a pattern that needs lookaround.  
   A limit reached is an error alert for that file, never a panic.
5. **Packages.**
   Rules ship as versioned packages that a project pins by version and hash.  
   hakkiri fetches and caches them; there is no separate sync step.  
   A package entry is a local `path` to a folder of style folders, or a `url` and its `sha256`.  
   A `url` names a zip that hakkiri downloads on the first run, checks against the hash, and unpacks whole into `<cache>/<sha256>/`, where the per-user cache folder or `--cache-dir` is the cache.  
   A later run reads that folder with no network.  
   The zip is a Vale package kept as it lays out: one top folder that holds a `styles` folder or is a style, so `HKR-007` and `HKR-008` find its files where Vale puts them.
   A package may hold vocabularies in `config/vocabularies/<name>/`, which the `vocab` key names at the top level and in a glob of `files` (operator, 2026-11-11).  
   The top level reaches every check that reads accepted terms, and a glob's list reaches the files it matches, as Vale's `Vocab` does.  
   Every `path` and `files` glob is relative to the project root, the folder that holds `.ctl`, so a file matches the same glob however it is named.  
   Vale matches the path as given; the departure is in the `vale-differences` spec.
6. **Exit status.**
   `hakkiri check` exits as Vale does: 2 when the config, a rule or a file failed, then 1 when a reported alert reaches `--fail-level` (default `error`), then 0.  
   `--fail-level` takes the place of Vale's `--no-exit`.  
   The pretty output is a line per alert with no colour and no frame, because tools read it.
7. **Test cases.**
   A rule file holds its cases under `tests`, and a `*.test.yml` file is a list of them. `check` loads cases and never runs them.  
   `hakkiri test` loads one rule alone for a case in a rule file or a case with `rule`, and lints any other case through the project config.  
   `--coverage` fails for a rule that flags nothing in any case.
8. **Release scope.**
   Release 1 is the CLI: `check` and `test`, run from a mise task, CI and a pre-commit hook.  
   Release 2 is the language server, on the same per-file engine.
9. **Releases are GitHub Release tarballs only.**
   `.github/workflows/release.yml` calls verctl's reusable release workflow, and `.ctl/ver.yaml` declares the package with `publisher: none`, so a release is a tag, a GitHub Release and two tarballs with their checksums.  
   Each tarball builds natively on a free GitHub-hosted runner: `linux-x64` on `ubuntu-latest`, and `darwin-arm64` on `macos-latest`.  
   The `linux-x64` binary is a static musl build (operator, 2026-11-11), so it starts on a machine with no glibc loader, NixOS without nix-ld included.  
   `mise run release:prepare` installs `musl-tools` and the Rust target before that build.  
   The musl build links mimalloc as its global allocator (operator, 2026-11-11), because musl's own allocator makes a parallel `check` about eight times slower.  
   No other build links it.  
   Each change that ships behaviour adds a `.changeset/` fragment; merging the Version PR is the release.  
   The README's install line and the skill's `version:` line are version pins, so never edit their numbers by hand.

## Workflow

- Work is queued in `tasks.yaml` and moved with qctl; never edit the ledger by hand.
- Each queued slice is an OpenSpec change under `openspec/changes/`, archived into `openspec/specs/` when it merges.
- The repository is rendered from the shared Copier template; change rendered files through a template update.

## Rules

- ctl's shared rules apply. Every change goes through a pull request, and `--no-verify` is never used.  
  ctl-core owns presentation, every flag has a `HAKKIRI_` variable, and nothing local leaks into a tracked file.
- The pull request's CI verifies. Do not build or test before a push.
- A change that makes hakkiri differ from Vale, or removes a difference, updates [`openspec/specs/vale-differences/spec.md`](openspec/specs/vale-differences/spec.md) in the same pull request.
- A behaviour change moves code, tests, `src/instructions.md`, `skills/hakkiri/SKILL.md`, this file and its OpenSpec change together (the ctl bundle contract).
- Prose in this repository follows ProseStyle: plain words, short sentences, no dashes as punctuation.
