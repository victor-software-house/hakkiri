## MODIFIED Requirements

### Requirement: A release is a tag, a Release and two tarballs

A merged Version PR SHALL publish a tag and a GitHub Release for the new version, and attach one tarball for each of `linux-x64` and `darwin-arm64`.  
The `linux-x64` tarball SHALL hold a statically linked `x86_64-unknown-linux-musl` binary.  
GitHub records each tarball's SHA-256 digest, which a lock reads.  
It SHALL publish nothing to crates.io.

#### Scenario: The first release

- **WHEN** the Version PR that moves `Cargo.toml` from `0.0.0` to `0.0.1` merges
- **THEN** the repository has the tag `v0.0.1` and a Release that holds the two tarballs, and crates.io has no `hakkiri` crate

#### Scenario: A platform missing

- **WHEN** the build of one platform fails
- **THEN** the release run fails, and a rerun attaches the tarballs the Release lacks

#### Scenario: A static binary

- **WHEN** `file` reads the `hakkiri` binary of the `linux-x64` tarball
- **THEN** it reports a static or static-pie linked executable, and `readelf -l` lists no interpreter

## ADDED Requirements

### Requirement: The linux-x64 binary needs no loader

The `linux-x64` binary SHALL start on a Linux machine that has no glibc loader, such as NixOS with nix-ld off.

#### Scenario: A root with no files

- **WHEN** the binary runs in a root folder that holds no file but the binary, with `check` on a Markdown file that has the word `very` and a rule that flags it
- **THEN** it prints `1 alert in 1 file` and exits 0, and `--version` prints the released version

### Requirement: The linux-x64 binary keeps its speed

The `linux-x64` binary SHALL use mimalloc as its global allocator, and no other build SHALL link it.  
A `check` over many files on four CPUs SHALL take at most twice as long as the same release built for glibc.

#### Scenario: Many files

- **WHEN** `check` lints 3000 Markdown files of 18 MB in total on four CPUs, with the musl build and the glibc build of one release
- **THEN** the musl build takes at most twice the time of the glibc build

#### Scenario: Other builds

- **WHEN** `cargo tree --target aarch64-apple-darwin` and `cargo tree --target x86_64-unknown-linux-gnu` list the dependencies
- **THEN** neither lists `mimalloc`
