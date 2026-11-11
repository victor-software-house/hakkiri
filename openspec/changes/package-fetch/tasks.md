# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.  
This change follows [`project-config`][config], which holds the grammar of a `url` entry.

## 1. The fetch

- [ ] 1.1 Add `src/package.rs`.  
  It fetches a `url` with a timeout and the 32 MiB limit, checks the SHA-256, unpacks into a temporary folder with the checks of design decision 3, finds `styles`, and renames the folder to `<cache>/<sha256>/`.  
  Verify `tests/package_fetch.rs` fails before the change and passes after it.  
  Its tests cover the first fetch, the cache hit, the offline hit and the offline miss.  
  They also cover the hash mismatch, a 404, a redirect, a body that is not a zip, a name that leaves the package, a package with no `styles` folder and a body over the limit.
- [ ] 1.2 Load a `url` entry in `load_packages` as the `styles` folder of its cache folder, and check the `url` scheme and the `sha256` shape in `entry_ok`.  
  Verify the scenarios for a hash of the wrong shape and for a url that is not http pass, and the existing refusals of an entry with both and of a `url` with no `sha256` still pass.
- [ ] 1.3 Add `--cache-dir` and `HAKKIRI_CACHE_DIR` to `check` and `test`, with the per-user default.  
  Verify the scenarios for the variable, the option and the default folder pass, and `hakkiri test` loads a fetched package.

## 2. The dependencies

- [ ] 2.1 Add `ureq`, `zip` and `sha2` with their licences in `deny.toml`, and check the musl binary grows by what design decision 5 says.  
  Verify `cargo deny` passes licenses, bans and sources, `cargo tree --target aarch64-apple-darwin` still builds, and the measured size is in the pull request.

## 3. The surfaces

- [ ] 3.1 Update `src/instructions.md`, `skills/hakkiri/SKILL.md`, `AGENTS.md` and the `vale-differences` spec for the fetched package, and add the patch changeset.  
  Verify `operator_docs` passes, `hakkiri instructions` no longer says a `url` entry is not downloaded, and the changeset names `hakkiri: patch`.

## 4. Integration

- [ ] 4.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.

[config]: ../../specs/project-config/spec.md
