# hakkiri

A prose linter for documentation and source comments, written in Rust.  
Its name is the Japanese はっきり, "clearly, plainly".

hakkiri checks Markdown, MDX, and the comments of Rust, TypeScript and JavaScript against rule packages, such as [ProseStyle].  
It is built to replace [Vale] in projects that use ProseStyle.

## Status

Planned, not usable yet.  
The first release lints what Vale lints today with ProseStyle 0.0.7.  
It adds rules that count a habit per sentence or per number of words.  
An editor server follows in the second release.

## Approach

- A rule picks nodes in the document tree first, such as a heading, a sentence, prose outside code, or a doc comment.  
  A pattern then narrows that pick, so a rule never rebuilds code fences or code spans out of regular expressions.
- Each source language has a profile that knows where its prose lives and in what syntax: rustdoc Markdown, TSDoc and JSDoc tags, plain comments.  
  String literals are never read as prose.
- A project pins rule packages by version and hash in `.ctl/hakkiri.yaml`; hakkiri fetches and caches them.
- Files are checked in parallel.

## Documentation

- [Research](openspec/changes/bootstrap-hakkiri/research.md): the field, the building blocks, and what ProseStyle needs.
- [Design](openspec/changes/bootstrap-hakkiri/design.md): what the design settled, the alternatives, and the release 1 plan.
- [Work queue](tasks.yaml): one row per slice, each planned as an OpenSpec change.

## License

[MIT](LICENSE)

[ProseStyle]: https://github.com/victor-software-house/prose-style
[Vale]: https://vale.sh
