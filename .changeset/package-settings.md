---
hakkiri: patch
"verctl:group": added
---

A package folder may hold `.ctl/hakkiri.yaml` in the format of the project config.
hakkiri applies the settings of each package ahead of the project's own, so the project overrides a package and a later package overrides an earlier one.
A package may list packages, pinned by `url` and `sha256`, to a depth of 10.
