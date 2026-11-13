---
hakkiri: patch
"verctl:group": fixed
---

A package that holds a style folder named `Vale` loads again: hakkiri skips that folder, as Vale does, and prints one warning that the built-in style `Vale` wins.
The other styles of the package lint as before.
