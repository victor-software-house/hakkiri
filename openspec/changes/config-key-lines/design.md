# Design

## Context

`key_line` read the text of the config line by line, from a line the last search had reached, for a line that began with the key.
It missed a key inside a flow mapping, and it took a later line for an earlier key when two globs wrote one setting.

## Decisions

### 1. Faults place keys through `Input::place`

A fault builds the path of its key, one step for each key, such as `files`, `docs/**/*.md` and `Team.Hint`.
The config's `Input` gives the line.

A glob that holds `.`, `*`, `/`, `[` or `{`, and a setting such as `Team.Hint`, are one step each, so no text splits them.
Alternative: keep the text search and make it read flow style.
It lost because a text search finds a line and not a key, and ctl-core owns the location index for every CLI.

### 2. A fault about one item lands on that item

A fault about a name of a list, such as a `styles` name that is no style, a `vocab` name that is no vocabulary or a pattern that does not compile, takes the index step of that item.
`place` gives the line of the item, so a later item of a block list gets its own line and not the first.

A fault about the whole value of a key still lands where the value starts.
For a scalar or a flow value that is the line of the key, and for a block collection it is the first line below it.
The faults about the key itself take `Input::place_key`, which ctl-core adds in [CTC-054][ledger].
They are a glob that is no glob and a setting that names no rule.
They are also a key that a package's settings file may not hold, and a `path` entry in such a file.

### 3. A layer keeps one `Input`

The check of a layer's keys and the faults about them read the same `Input`, so the document is parsed once for both.

[ledger]: ../../../tasks.yaml
