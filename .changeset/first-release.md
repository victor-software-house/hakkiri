---
hakkiri: patch
"verctl:group": added
---

First release. `hakkiri check` lints Markdown, MDX and the comments of Rust, TypeScript and JavaScript under `.ctl/hakkiri.yaml`, in parallel, and exits 2, 1 or 0 as Vale does.
`hakkiri test` runs the cases that rule files and `*.test.yml` files hold, and `--coverage` fails for a rule no case makes flag anything.
Releases are GitHub Release tarballs for `linux-x64` and `darwin-arm64`, each with its checksum, and mise installs them with the `github:` backend.
