# Design

## Context

A package is a folder of style folders.
The `url` form already parses: the entry holds a `url` and a `sha256`, and `load_packages` in `src/config.rs` turns each entry into a folder of styles.
prose-style publishes `ProseStyle.zip` with the layout `ProseStyle/.vale.ini` and `ProseStyle/styles/<Style>/*.yml`, with vocabularies under `styles/config`.
That is a Vale package, and `HKR-007` makes the loader read Vale styles as they are, inheritance included.
So a fetched package is a Vale package used without conversion.

## Goals / Non-Goals

Goals:

- A `url` entry loads like a `path` entry once its zip is in the cache.
- The cache keeps the package whole, as the zip laid it out, so the loader and `HKR-008` find `.vale.ini`, `meta.json` and the vocabularies where Vale puts them.
- A tampered or truncated download never reaches a rule.
- A cached package lints with no network.

Non-Goals:

- Reading the package's `.vale.ini` or its vocabularies, which `HKR-008` and `HKR-019` do.
- A `sync` command, a git source, a version range or a lockfile.
- Pruning the cache: a package costs about 120 KB.

## Decisions

### 1. Vale's sync is the reference for the source

Probed on Vale 3.26.0 at `66bbb85`:

| Case | Vale |
|:--|:--|
| Source | `Packages` takes an HTTP zip, a local zip or a local folder, and no git URL |
| Request | A plain GET that follows redirects, with no timeout, size limit or hash |
| Store | The styles go into `StylesPath`, which is the only cache, and `vale sync` replaces each style folder whole |
| Lint | Never fetches: after a sync it lints with the server down, and with no sync it fails with E201 |
| Failure | A 404, a bad URL, a file that is not a zip and a server that is down each exit 2 with a message |
| Tampering | A changed zip is accepted |

hakkiri keeps the zip source and the layout.
It differs by requiring the hash, by fetching on the first run instead of in a `sync` step, and by keeping the unpacked package outside the project.

### 2. The cache folder is named by the hash

The operator chose an own folder store over a crate (operator, 2026-11-11).

`<cache>/<sha256>/` holds the unpacked package whole, as the zip laid it out, such as `<cache>/<sha256>/ProseStyle/styles/ProseStyle/`.
The same bytes are unpacked once whatever `url` served them, and a new pin is a new folder.
A run that finds the folder reads it and makes no request.
The download is unpacked into a temporary folder in the cache and renamed into place, so a run that stops half way leaves no partial folder.
Two runs that fetch together both unpack, one rename wins, and the other run discards its copy.
Each run unpacks into a new folder whose name holds its process id and a random number.
It makes the folder with `create_dir`, so a name in use fails, and it removes only that folder.
Runs that share a cache and a process id, such as containers on one cache volume, never touch each other's copy.
Rule files stay on disk, so an error about a rule names a real path.

Alternative: store each package as a JSON value in `unixd-cache`.
Lost: the crate's freshness horizons serve responses that go stale, and a hash-pinned package never does.
A JSON package would also leave no rule file on disk to name.

### 3. The checks a download passes

A download passes these checks in order:

- The `url` starts with `https://` or `http://`.
- The body is at most 32 MiB, and its SHA-256 equals the entry's.
- The body is a zip, every name stays inside the package, and the unpacked size stays under 128 MiB.
- The zip holds one top folder, as Vale's packages do.
- That folder holds a `styles` folder, which is the package folder, or else it is itself a style, as in Vale's style-only packages, and the package folder is the unpacked zip.

A `sha256` is 64 lowercase hex digits, and the config load reports a wrong shape with the entry's line.
A fault names the `url`, and a mismatch also names the expected and the actual hash.

Alternative: check the hash of the unpacked tree on every run.
Lost: it costs a walk of the package on each run, and the cache folder belongs to the user who runs hakkiri.

### 4. The cache folder is an option

The operator chose the name `--cache-dir` (operator, 2026-11-11), as `uv` calls it.
`--cache-dir` takes a folder, and `HAKKIRI_CACHE_DIR` sets it, as every option has a variable.
Its default is `~/Library/Caches/hakkiri` on macOS and `$XDG_CACHE_HOME/hakkiri` or `~/.cache/hakkiri` elsewhere, and an unset or relative base is ignored.
With no usable base the run is an error that names the option.

### 5. The client

`ureq` makes the request and follows redirects with bundled root certificates, so it needs no system store, which a static musl binary lacks (operator, 2026-11-11).
`zip` reads the archive without its compressors, and `sha2` hashes the body.
`deny.toml` gains the ISC licence for `ring`, `rustls-webpki` and `untrusted`, and CDLA-Permissive-2.0 for `webpki-roots`, through `copier update --data deny_licenses`.

Sizes, from builds under the repository's own `[profile.release]`: `opt-level = "s"`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"` and `strip = "symbols"`.
The rustc flags of the final crate show each setting.
Each is `cargo build --release --locked --target x86_64-unknown-linux-musl` with mimalloc, in `ubuntu:24.04`.

| Build | Binary | Delta | Crates |
|:--|:--|:--|:--|
| main, before this change | 12,781,136 | | 120 |
| `ureq`, `zip` and `sha2`, this change | 14,186,128 | +1.41 MB (+11.0%) | 182 |
| `reqwest` 0.13.5 async with tokio, as verctl declares it, `zip` and `sha2` | 15,390,608 | +2.61 MB (+20.4%) | 341 |
| `reqwest` 0.13.5 `blocking`, `zip` and `sha2` | 15,415,184 | +2.63 MB (+20.6%) | 346 |

The `reqwest` rows measure a probe that downloads and unpacks one zip, and the `ureq` row measures the shipped code.
verctl is the one of our crates that uses an HTTP client, and its `reqwest` costs twice what `ureq` does, with tokio and hyper.

Alternative: `reqwest`, as verctl uses it.
Lost: +2.6 MB and an async runtime in a CLI that has none.
Alternative: `minreq` adds 2.0 MB and needs more licences.
Alternative: `curl` as a child process needs curl on every machine.

## Risks / Trade-offs

- [A hash-pinned URL that moves breaks a run] → The error names the `url` and both hashes, and the cache keeps working for a pin it already holds.
- [A zip bomb] → The unpacked size and the body size are capped.
- [Another process fills the cache folder] → A folder named by a hash is trusted, as the user's own cache.
