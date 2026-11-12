# Design

## Context

Vale reads a YAML vocabulary with yaml.v3 into a struct, checks each entry as the struct unmarshals it, and stops at the first problem ([vocab.go][v-vocab]).
An entry gives the patterns that a line of a text list would, and the file's lists join those of the text files.
hakkiri reads the text lists in `src/vocabulary.rs`, and the config loader finds each vocabulary folder.

## Goals / Non-Goals

Goals:

- A YAML vocabulary gives the alerts that Vale gives, for every entry shape.
- A wrong entry is refused with its file and line.

Non-goals:

- A reader of YAML of our own: ctl-core's input parses the file and places each problem.
- `$DICPATH`: no environment variable decides what a project accepts.

## Decisions

### 1. The file parses through ctl-core's input

`Input::parse` reads the file with serde-saphyr, so a key the file does not define is refused with its line, and a limit of the input applies.
yaml.v3 ignores a key it does not define, so a misspelt `accpt:` drops every term of the list without a word.
hakkiri refuses it, which finds the typo.
Alternative: ignore unknown keys, as Vale does.
It lost because the failure is silent and the rule that needs the term then fires on every file.

### 2. An entry is checked on its node, and every problem is reported

An entry is a scalar, or a mapping of one key to nothing or to options.
A small `Node` type reads the entry's shape from serde-saphyr, and the checks that Vale does while it unmarshals run on it: one key, options that are a mapping, a known option name, a boolean for `regex` and `ignorecase`, then a term, `regex` with `flags`, a pattern that compiles and flags that expand.
Each problem is placed through `Input::place` or `Input::place_key`, on the line Vale names: the entry for a shape or term fault, the value for a mapping to a string or a list, the option key for an unknown option.
One run lists every wrong entry, where Vale stops at the first.
A syntax error of the whole file is one problem, on the line the parser gives.
For a list `[a` that opens on line 3 and never closes, that is line 3, and yaml.v3 names line 2, which holds a complete entry.
Alternative: an untagged enum for the entry.
It lost because serde reports only that no variant matched, with no word of what is wrong with the entry.

### 3. A plain term is literal

A term with no regex option stands for itself.
A term whose only special character is a period stays as written, which a text list reads as plain text with a literal period.
Any other term is quoted, as Go's `QuoteMeta` quotes it, so `C++` matches `C++` and not a run of `C`.
`regex: true` keeps the term as a pattern and compiles it as a `.txt` line would.
`flags` expands the term and quotes each form, and `ignorecase` puts `(?i)` before each pattern.
The patterns are then the lines a text list would hold, so the rules built from a vocabulary read them as before.
Alternative: read a plain YAML term as a text-list line, which treats `C++` as a pattern.
It lost because Vale quotes it, and the alerts of `C++` differ.

### 4. A scalar that is not a string keeps its spelling

`- 3.10` reaches serde-saphyr as a float, and a float prints as `3.1`.
Vale takes the node's text, so the term is `3.10`.
hakkiri reads the spelling of a number or boolean entry from the file at the place of the node, up to a comment or a flow delimiter.
Alternative: print the parsed value.
It lost because a version such as `3.10` would accept `3.1`.

### 5. Null and the words of YAML 1.1

yaml.v3 never calls the entry's reader for a null, so `-` alone or `- ~` adds nothing, and hakkiri skips it too.
yaml.v3 reads `yes`, `no`, `on`, `off`, `y` and `n` as booleans when the target is a boolean, so `regex: yes` turns the option on, and hakkiri does the same for `regex` and `ignorecase`.
Alternative: serde-saphyr's strict booleans, which read only `true` and `false`.
It lost because a package written for Vale would then be refused.

### 6. The affix file comes from the packages, never from `$DICPATH`

Vale looks for `<name>.aff` in the dictionaries folder of each search path, then in `$DICPATH`, and uses its built-in dictionary for no name and for `en_US-web`.
hakkiri looks in `config/dictionaries` of the package that holds the vocabulary, then of every other loaded package in order, and uses the built-in dictionary for the same two names.
A name that is not one file name, such as `../x`, is refused, since packages come from URLs.
Alternative: read `$DICPATH` too.
It lost because the result would depend on the shell that runs hakkiri, and a package carries what it needs.

### 7. Every file of the folder joins

A folder that holds `accept.txt`, `vocab.yml` and `vocab.yaml` reads all three, in name order, as Vale's walk does.
The text lists keep their rule for a line.

[v-vocab]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/vocab.go
