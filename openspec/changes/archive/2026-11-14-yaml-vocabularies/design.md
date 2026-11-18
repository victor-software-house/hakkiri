# Design

## Context

Vale reads a YAML vocabulary with yaml.v3 into a struct.  
It checks each entry as the struct unmarshals it, and stops at the first problem ([vocab.go][v-vocab]).  
An entry gives the patterns that a line of a text list would, and the file's lists join those of the text files.

hakkiri reads the text lists in `src/vocabulary.rs`, and the config loader finds each vocabulary folder.

## Goals / Non-Goals

Goals:

- A YAML vocabulary gives the alerts that Vale gives, for every entry shape.
- hakkiri rejects a wrong entry and shows its file and line.

Non-goals:

- A reader of YAML of our own: ctl-core's input parses the file and places each problem.
- `$DICPATH`: no environment variable changes what a project accepts.

## Decisions

### 1. The file parses through ctl-core's input

`Input::parse` reads the file with serde-saphyr.  
So hakkiri rejects a key the file does not define and shows its line, and a limit of the input applies.  
yaml.v3 ignores a key it does not define, so a misspelt `accpt:` drops every term of the list with no error.  
hakkiri rejects it, which finds the typo.

Alternative: ignore unknown keys, as Vale does.  
It lost because the failure is silent, and the rule that needs the term then fires on every file.

### 2. An entry is checked on the node, and every problem is reported

An entry is a scalar, or a mapping of one key to nothing or to options.  
A small `Node` type reads the entry's shape from serde-saphyr.  
The checks that Vale does while it unmarshals run on it, in this order:

1. one key;
2. options that are a mapping;
3. a known option name;
4. a boolean for `regex` and `ignorecase`;
5. a term;
6. `regex` with `flags`;
7. a pattern that compiles, and flags that expand.

hakkiri places each problem through `Input::place` or `Input::place_key`, on the line where Vale reports it:

- the entry, for a shape or term fault;
- the value, for a mapping to a string or a list;
- the option key, for an unknown option.

One run lists every wrong entry, where Vale stops at the first.  
A syntax error of the whole file is one problem, on the line the parser gives.  
Take a list `[a` that opens on line 3 and never closes.  
hakkiri reports line 3, and yaml.v3 reports line 2, which contains a complete entry.

Alternative: an untagged enum for the entry.  
It lost because serde reports only that no variant matched, and not what is wrong with the entry.

### 3. A plain term is literal

A term with no regex option stands for itself.  
A term whose only special character is a period stays as written.  
In a text list, such a line reads as plain text with a literal period.

With `ignorecase`, the pattern starts with `(?i)`.  
A text list then reads the line as a regular expression, so the period would match any character.  
So hakkiri quotes a term with `ignorecase` whole, and `Node.js` matches `Node.js` and not `Nodexjs`.  
Vale keeps the period of such a term as a wildcard, and hakkiri differs there.

hakkiri quotes any other term as Go's `QuoteMeta` does, so `C++` matches `C++` and not a run of `C`.  
`regex: true` keeps the term as a pattern and compiles it as a `.txt` line would.  
`flags` expands the term and quotes each form.  
`ignorecase` puts `(?i)` before each pattern, after the quoting.

The patterns are then the lines a text list would contain.  
So the rules built from a vocabulary read them as before.

Alternative: read a plain YAML term as a text-list line, which treats `C++` as a pattern.  
It lost because Vale quotes it, and the alerts of `C++` differ.

### 4. A scalar that is not a string keeps the spelling written in the file

`- 3.10` reaches serde-saphyr as a float, and a float prints as `3.1`.  
Vale takes the node's text, so the term is `3.10`.  
For a number or boolean entry, hakkiri reads the spelling where the node sits in the file.  
It reads up to a comment or a flow delimiter.

Alternative: print the parsed value.  
It lost because a version such as `3.10` would accept `3.1`.

### 5. Null and the words of YAML 1.1

yaml.v3 never calls the entry's reader for a null.  
So `-` alone or `- ~` adds nothing, and hakkiri skips it too.  
When the target is a boolean, yaml.v3 reads `yes`, `no`, `on`, `off`, `y` and `n` as booleans.  
So `regex: yes` turns the option on, and hakkiri does the same for `regex` and `ignorecase`.

Alternative: serde-saphyr's strict booleans, which read only `true` and `false`.  
It lost because hakkiri would then reject a package written for Vale.

### 6. The affix file comes from the packages, never from `$DICPATH`

Vale looks for `<name>.aff` in the dictionaries folder of each search path, then in `$DICPATH`.  
It uses its built-in dictionary for no name and for `en_US-web`.

hakkiri first looks in `config/dictionaries` of the package that contains the vocabulary.  
Then it looks in every other loaded package in order.  
It uses the built-in dictionary for the same two names.  
It rejects a name that is not one file name, such as `../x`, since packages come from URLs.

Alternative: read `$DICPATH` too.  
It lost because the result would depend on the shell that runs hakkiri, and a package carries what it needs.

### 7. Every file of the folder joins

A folder with `accept.txt`, `vocab.yml` and `vocab.yaml` reads all three, in name order, as Vale's walk does.  
The text lists keep their rule for a line.

[v-vocab]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/vocab.go
