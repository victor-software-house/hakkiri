---
hakkiri: patch
"verctl:group": added
---

A vocabulary folder may hold `vocab.yml` and `vocab.yaml` besides `accept.txt` and `reject.txt`, as in Vale.
A YAML entry is a term, a term mapped to nothing, or a term mapped to the options `regex`, `ignorecase` and `flags`, with a plain term read as literal text.
An `affix` key names an affix file in the `config/dictionaries` folder of the package that holds the vocabulary, then of every other package, and never in `$DICPATH`.
A key the file does not define is an error, and every wrong entry is reported with its file and line.
