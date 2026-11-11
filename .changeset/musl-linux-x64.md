---
hakkiri: patch
"verctl:group": changed
---

The `linux-x64` release is a static musl binary, so it starts on a Linux machine with no glibc loader, such as NixOS without nix-ld.
The musl build links mimalloc, because musl's own allocator made a parallel `check` about eight times slower.
The asset name does not change, so a `mise` pin of `github:victor-software-house/hakkiri` needs no edit.
