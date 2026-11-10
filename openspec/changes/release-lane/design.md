# Design

## Context

qctl and verctl release through verctl's [reusable workflow][v-readme], and rgbctl, a binary with no crates.io crate, is the nearest shape: a tag, a GitHub Release and tarballs.
hakkiri's template answers say `published: false` and list the platforms `linux-x64` and `macos-arm64`.
Those two are what every consumer locks, and `mise lock` needs an asset for each platform it locks.
The latest verctl release is 0.7.4, and the README of its main branch describes 0.8.0, so this design pins a released verctl and not main.

## Goals / Non-Goals

Goals:

- A merged Version PR publishes `0.0.1` with no manual step.
- `mise` installs it on a Mac and on Linux x86-64 from a pinned version and a locked checksum.

Non-goals:

- A crates.io crate: `publish = false` stays, so no registry token or trusted publisher is set up.
- Static musl and `aarch64` Linux builds, until a consumer locks them (decision 3).
- Signed checksum sets.

## Decisions

### 1. The release is verctl's reusable workflow

`.github/workflows/release.yml` calls verctl's reusable workflow, pinned by commit with its tag in a comment, on every push to `main` and on dispatch.
It passes the App secrets by name in its `secrets:` map, and never `secrets: inherit`.
`.ctl/ver.yaml` declares one package, `hakkiri`, with `publisher: none`, so a release is a tag, a GitHub Release and its assets.
`mise.toml` pins the same verctl release as a tool and includes its `ver` task at the same tag.
The organization's `CHANGESET_VERSION_APP_CLIENT_ID` and `CHANGESET_VERSION_APP_PRIVATE_KEY` reach the repository already (checked through the Actions API).

Alternative: `version-pr.yml` and `publish.yml` as qctl has them.
Lost: verctl's README makes the reusable workflow the one caller and names the old pair a migration.

### 2. The first release is `0.0.1`

One fragment under `.changeset/` names `hakkiri` with a patch bump and says the release ships `check` and `test`.
The Version PR moves `Cargo.toml` from `0.0.0` to `0.0.1`, and `prepare.after` regenerates `Cargo.lock` so the commit stays clean.
Later behaviour changes add their own fragment.

### 3. Two assets, built in

`assets` lists `darwin-arm64` and `linux-x64`, verctl's built-in records, as qctl does.
The Linux build is the default `x86_64-unknown-linux-gnu`.
A release then has the two platforms the template locks and no other.

Alternative: static musl for `x86_64` and `aarch64`, as rgbctl has.
Lost for now: hakkiri links tree-sitter's C grammars, so a musl build needs a musl C compiler that the release job does not declare, and an `aarch64` Linux consumer does not exist yet.
The first release run shows whether the glibc binary starts on the machines that lint; a row follows if the choice turns to musl.

### 4. mise installs the asset

A consumer pins `"github:victor-software-house/hakkiri" = "0.0.1"` and runs `mise lock` for `linux-x64` and `macos-arm64`, which records each asset's checksum.
verctl attaches each tarball and its `.sha256`, and the `github:` backend picks the asset by platform name ([asset names][v-readme]).
The proof is an install in an empty folder on each platform: the pin, the lock, then `hakkiri --version` and a `check` of one file.

### 5. Two texts carry the version

The skill's `version:` line and the README's install line name the release.
Two patterns, each `once`, and a `pins` entry for each file let the Version PR move both with the crate, so no one edits the number.
A pattern that matches nowhere or twice stops the release.

## Risks / Trade-offs

- [The glibc binary may not start on NixOS] → The install proof runs on a NixOS machine before consumers switch; a failure turns decision 3 to musl.
- [The workflow ref is a commit, so it ages] → Moving it is a deliberate change, made with the verctl pin in `mise.toml`.
- [A Version PR opened with `GITHUB_TOKEN` starts no CI] → The App token opens it.

[v-readme]: https://github.com/victor-software-house/verctl/blob/bc30ea6f0924edd8554ef5444280d9ca52dd1851/README.md
