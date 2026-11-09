# hakkiri

A prose linter for documentation and source comments, written in Rust.  
Its name is the Japanese はっきり, "clearly, plainly".

hakkiri checks Markdown, MDX, and the comments of Rust, TypeScript and JavaScript against rule packages, such as [ProseStyle].  
It is built to replace [Vale] in projects that use ProseStyle.

## Status

Planned, not usable yet.  
The first release lints what Vale lints today with ProseStyle 0.0.7, plus rules that count a habit per sentence or per number of words.  
An editor server follows in the second release.

## How it works

- A rule picks nodes in the document tree first, such as a heading, a sentence, prose outside code, or a doc comment, and a pattern then narrows that pick.  
  So a rule never rebuilds code fences or code spans out of regular expressions.
- Each source language has a profile that knows where its prose lives and in what syntax: rustdoc Markdown, TSDoc and JSDoc tags, plain comments.  
  String literals are never read as prose.
- A project pins rule packages by version and hash in `.ctl/hakkiri.yaml`; hakkiri fetches and caches them.
- Files are checked in parallel.

## Documentation

- [Research](docs/research.md): the field, the building blocks, and what ProseStyle needs.
- [Decisions](docs/decisions.md): what the design settled and why.

## License

[MIT](LICENSE)

[ProseStyle]: https://github.com/victor-software-house/prose-style
[Vale]: https://vale.sh
