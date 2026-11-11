# Changelog

## hakkiri 0.0.5

### Added

- A glob's empty `vocab` list drops only the lists that earlier configs gave the same glob text, as Vale's `NONE` clears only the section of that name.
  The other globs that match a file still add their lists ([`a855a45`](https://github.com/victor-software-house/hakkiri/commit/a855a45fb2963bcd52b598009a51c54b85c5e48e)).
- A package folder may hold `.ctl/hakkiri.yaml` in the format of the project config.
  hakkiri applies the settings of each package ahead of the project's own, so the project overrides a package and a later package overrides an earlier one.
  A package may list packages, pinned by `url` and `sha256`, to a depth of 10 ([`15c5e81`](https://github.com/victor-software-house/hakkiri/commit/15c5e813715f1148b2e047735ae7bbe68da5875f)).

## hakkiri 0.0.4

### Added

- A project names vocabularies in `vocab`, at the top level and in a glob of `files`, and hakkiri reads each from `config/vocabularies/<name>/` in a package.
  The accepted terms hold back matches in `capitalization`, `conditional`, `existence`, `repetition`, `substitution` and `spelling` rules, as in Vale, and a glob's list does so for the files it matches ([`5023925`](https://github.com/victor-software-house/hakkiri/commit/5023925ee456344843e79f56f9879fa3576c7a72)).

## hakkiri 0.0.3

### Added

- A package pinned by `url` and `sha256` is downloaded on the first run, checked against its hash, and kept in a cache folder, so a second run needs no network.
  A zip is a Vale package: one top folder that holds a `styles` folder, or is a style itself, kept as the zip laid it out.
  `--cache-dir`, or `HAKKIRI_CACHE_DIR`, moves the cache from its default in the per-user cache folder ([`51cdd62`](https://github.com/victor-software-house/hakkiri/commit/51cdd62651718a6377c915eeb7e450174ef87ddd)).
- A Vale style loads as it is: a rule whose `extends` names another rule starts from it, with `key+` and `key-`, as in Vale.
  A rule hakkiri cannot run, a `script` rule or a `doc()` scope, no longer stops its style from loading.
  The rest of the style runs, and the run exits 2 for that rule until the config sets it `off` ([`d4dc1f9`](https://github.com/victor-software-house/hakkiri/commit/d4dc1f9e29411884ddf002741e109dfa7507dacf)).

## hakkiri 0.0.2

### Added

- A key that names a rule and a level under a glob of `files` now runs that rule, though no glob lists its style, as a rule setting does in Vale.
  The word `on` turns a rule on at the level of its rule file, as Vale's `YES` does.
  An empty `styles` list also drops the rule settings before it ([`b3c8136`](https://github.com/victor-software-house/hakkiri/commit/b3c813682f5b9d9de10270df2dc91214882bb51f)).

### Changed

- The `linux-x64` release is a static musl binary, so it starts on a Linux machine with no glibc loader, such as NixOS without nix-ld.
  The musl build links mimalloc, because musl's own allocator made a parallel `check` about eight times slower.
  The asset name does not change, so a `mise` pin of `github:victor-software-house/hakkiri` needs no edit ([`b5ceac7`](https://github.com/victor-software-house/hakkiri/commit/b5ceac7227ac03a51489000c3f90de8b3e499baa)).

## hakkiri 0.0.1

### Added

- First release. `hakkiri check` lints Markdown, MDX and the comments of Rust, TypeScript and JavaScript under `.ctl/hakkiri.yaml`, in parallel, and exits 2, 1 or 0 as Vale does.
  `hakkiri test` runs the cases that rule files and `*.test.yml` files hold, and `--coverage` fails for a rule no case makes flag anything.
  Releases are GitHub Release tarballs for `linux-x64` and `darwin-arm64`, each with its checksum, and mise installs them with the `github:` backend ([`bf3b6d5`](https://github.com/victor-software-house/hakkiri/commit/bf3b6d559f07e06278646d7bbe587b05d7ca418c)).

