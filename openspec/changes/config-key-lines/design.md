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

### 2. A multi-line value lands on the first line

`place` gives the line where the value of the path starts.
For a scalar or a flow value that is the line of the key.
For a block mapping or a block list it is the first line below the key.
A fault about a block `styles` list lands on its first item, which is the name that failed.

### 3. A layer keeps one `Input`

The check of a layer's keys and the faults about them read the same `Input`, so the document is parsed once for both.
