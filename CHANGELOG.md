# Changelog

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

