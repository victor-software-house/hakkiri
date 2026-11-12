---
hakkiri: patch
"verctl:group": added
---

The built-in style `Vale` holds `Vale.Spelling`, which reports a word the built-in dictionary lacks, at the level `error`, as Vale does.
Each package's `config/dictionaries` folder adds its dictionaries, and every `.txt` file under its `config/ignore` folder adds the words it lists.
A project that lists `Vale` in `styles` now gets spelling alerts, and turns them off with `Vale.Spelling: off`.
A config sets a field with `params`, such as `Vale.Spelling: {split: true}`.
It differs from Vale where several packages each hold settings, in a code span of a code comment, and in the name after `@param`.
A Markdown file now keeps a bare email address, a `www.` address and an angle-bracket email address in its text, as Vale does, where it dropped them as links.
