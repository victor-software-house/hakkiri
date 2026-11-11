# hakkiri

A prose linter for documentation and source comments, written in Rust.  
Its name is the Japanese はっきり, "clearly, plainly".

hakkiri checks Markdown, MDX, and the comments of Rust, TypeScript and JavaScript against rule packages, such as [ProseStyle].  
It is built to replace [Vale] in projects that use ProseStyle.

## Status

`hakkiri check` lints Markdown, MDX and source comments under `.ctl/hakkiri.yaml`, in parallel, and exits 2, 1 or 0 as Vale does.  
`hakkiri test` runs the test cases of rule files and `*.test.yml` files, and `--coverage` fails for a rule no case makes flag anything.  
A package is a local folder of styles for now.  
The download pinned by hash is not ready.  
The first release lints what Vale lints today with ProseStyle 0.0.7.  
It adds rules that count a habit per sentence or per number of words.  
An editor server follows in the second release.

## Approach

- A rule picks nodes in the document tree first, such as a heading, a sentence, prose outside code, or a doc comment.  
  A pattern then narrows that pick, so a rule never rebuilds code fences or code spans out of regular expressions.
- Each source language has a profile that knows where its prose lives and in what syntax: Markdown in every comment, plus TSDoc and JSDoc tags.  
  String literals are never read as prose.
- A project pins rule packages by version and hash in `.ctl/hakkiri.yaml`; hakkiri fetches and caches them.
- Files are checked in parallel.

## Install

A release is a GitHub Release with a tarball and a checksum for `linux-x64` and `darwin-arm64`.  
mise installs it with the `github:` backend, pinned to the version:

```sh
mise use github:victor-software-house/hakkiri@0.0.0
```

## Use

```sh
hakkiri check
hakkiri check docs README.md
hakkiri check --format json
hakkiri test --coverage
hakkiri instructions
```

`hakkiri instructions` prints the operator contract: the config keys, the output, the exit status and the variables.  

## Documentation

- [Research](openspec/changes/archive/2026-11-09-bootstrap-hakkiri/research.md): the field, the building blocks, and what ProseStyle needs.
- [Design](openspec/changes/archive/2026-11-09-bootstrap-hakkiri/design.md): what the design settled, the alternatives, and the release 1 plan.
- [Work queue](tasks.yaml): one row per slice, each planned as an OpenSpec change.

## License

Copyright (c) 2026 Victor Software House.  
[AGPL-3.0-or-later](LICENSE): GNU Affero General Public License, version 3 or any later version.
The embedded part-of-speech model is [CC BY-SA 4.0](data/tagger/NOTICE.md), trained on UD English EWT, and stays a separate data file.
The embedded English dictionary is under the [terms of the English Speller Database](data/dictionary/NOTICE.md), which allow any use if its copyright notice ships with every copy.

[ProseStyle]: https://github.com/victor-software-house/prose-style
[Vale]: https://vale.sh
