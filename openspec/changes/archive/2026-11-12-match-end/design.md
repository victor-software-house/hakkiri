# Design

## Context

`Alert` held `line`, `first_column` and `last_column`.
For a match that wrapped a line, `last_column` was the end of the first line, so the end of the match was in no field.
Vale prints the column where the match would end if it stayed on its first line, so a wrapped match ends past the line.

## Goals / Non-Goals

Goals:

- An alert names the line and column of the last character of its match.
- A tool that reads the JSON gets both without parsing the match.

Non-goals:

- The pretty line: it keeps `path:line:column`, and tools that parse it are unaffected.
- A column that counts anything but characters.

## Decisions

### 1. The end names the last character

`end_line` and `end_column` name the last character of the match in the file, so markup inside the match counts, and a match on one line has `end_line` equal to `line`.
The operator chose this over Vale's count and over the old end of the first line, because hakkiri does not copy a flaw of Vale's to match it (operator, 2026-11-11).
Alternative: Vale's count, the first column plus the number of characters of the match.
It lost because it points past the end of the line for a wrapped match, where no character of the match is.
Alternative: keep the end of the first line.
It lost because it names no character of the match past the first line, and no field holds the line the match ends on.

### 2. The comparison with Vale maps the end

Vale's end for a wrapped match is the first column plus the characters of the match, line breaks included.
The comparison in `tests/vocabulary_rules.rs` computes that count from hakkiri's `column` and `match`, and compares it with Vale's recorded column.
A second test pins the true end of the two wrapped matches in the fixture.
Alternative: leave the end of a wrapped match out of the comparison.
It lost because the queue row requires the comparison to check it, and the mapping is one line of the test.
