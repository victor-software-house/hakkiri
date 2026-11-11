---
hakkiri: patch
"verctl:group": added
---

A package pinned by `url` and `sha256` is downloaded on the first run, checked against its hash, and kept in a cache folder, so a second run needs no network.
A zip is a Vale package: one top folder that holds a `styles` folder, or is a style itself, kept as the zip laid it out.
`--cache-dir`, or `HAKKIRI_CACHE_DIR`, moves the cache from its default in the per-user cache folder.
