---
hakkiri: patch
"verctl:group": added
---

The built-in style `Vale` holds `Vale.Spelling`, which reports a word the built-in dictionary lacks, at the level `error`, as Vale does.
Each package's `config/dictionaries` folder adds its dictionaries, and every `.txt` file under its `config/ignore` folder adds the words it lists.
A link under `config/ignore` that reaches outside the package is refused, as every other path of a package is.
A project that lists `Vale` in `styles` now gets spelling alerts, and turns them off with `Vale.Spelling: off`.
The rule is built only when a glob lists `Vale`, names a rule that starts with `Vale.`, or sets `params` for one, so a package with a dictionary that cannot load fails no run that does not use it.
A config sets a field with `params`, such as `Vale.Spelling: {split: true}`.
It differs from Vale where several packages each hold settings, in a code span of a code comment, and in the name after `@param`.
A Markdown file now keeps a bare email address, a `www.` address and an angle-bracket email address in its text, as Vale does, where it dropped them as links.
