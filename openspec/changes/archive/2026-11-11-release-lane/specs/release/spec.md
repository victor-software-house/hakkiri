## ADDED Requirements

### Requirement: A release is a tag, a Release and two tarballs

A merged Version PR SHALL publish a tag and a GitHub Release for the new version, and attach one tarball for each of `linux-x64` and `darwin-arm64`.  
GitHub records each tarball's SHA-256 digest, which a lock reads.  
It SHALL publish nothing to crates.io.

#### Scenario: The first release

- **WHEN** the Version PR that moves `Cargo.toml` from `0.0.0` to `0.0.1` merges
- **THEN** the repository has the tag `v0.0.1` and a Release that holds the two tarballs, and crates.io has no `hakkiri` crate

#### Scenario: A platform missing

- **WHEN** the build of one platform fails
- **THEN** the release run fails, and a rerun attaches the tarballs the Release lacks

### Requirement: mise installs the release

`mise` SHALL install the released binary from `github:victor-software-house/hakkiri` pinned to the version, on `linux-x64` and `macos-arm64`, with the checksum its lock records.

#### Scenario: A pinned install

- **WHEN** an empty project pins `"github:victor-software-house/hakkiri" = "0.0.1"` and runs `mise lock` and `mise install`
- **THEN** `hakkiri --version` prints `0.0.1` on a Mac and on Linux x86-64, and `hakkiri check` lints a Markdown file

### Requirement: Two texts carry the version

The skill's `version:` line and the README's install line SHALL name the released version, and the Version PR SHALL move both with the crate.  
A pattern that matches nowhere or more than once SHALL stop the release.

#### Scenario: A version bump

- **WHEN** the Version PR bumps the crate to `0.0.1`
- **THEN** `skills/hakkiri/SKILL.md` holds `version: 0.0.1` and the README installs `hakkiri@0.0.1`
