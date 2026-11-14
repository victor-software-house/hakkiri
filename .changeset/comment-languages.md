---
hakkiri: patch
"verctl:group": fixed
---

hakkiri reads the comments of Go, Swift, YAML, TOML and shell files, through new `go`, `swift`, `yaml`, `toml` and `sh` readers that `formats` takes.
It leaves out the comments that tools read, in every language.
It draws no alert in the link or name of a Go, Rust or JSDoc doc comment.
A line comment of a Go, YAML, TOML or shell file keeps its indentation after the first space.
A file with one of the new extensions that a glob selects gives its comments only, where it was plain text.
