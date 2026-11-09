# Design

## Context

The [bootstrap design][bootstrap] fixed the rule model.  
A rule selects nodes with ast-grep's terms: `kind`, `inside`, `has`, `not`, `all`, `any`.  
It then runs a Vale-style check or a rate over them.  
The project config lives in `.ctl/hakkiri.yaml`.  
Each decision below was shown to the operator with every candidate side by side, written as the same four ProseStyle rules.

## Goals / Non-Goals

**Goals:**

- One format that holds every enabled ProseStyle rule and every frequency-habit rule, with no lookbehind that rebuilds structure.
- Field and node names taken from one established source each, cited.

**Non-Goals:**

- Engine code, the tree, or the importer: later slices build them against these specs.

## Decisions

### 1. Rule files are YAML

Rule files are YAML 1.2, read by the same reader as `.ctl/hakkiri.yaml` (operator, 2026-11-09).  
ast-grep and Vale both write rules in YAML, so ProseStyle's authors already know it.  
A `|` block holds a Markdown test input as it is, lists are unambiguous, and a pattern fits in single quotes with its backslashes kept.

Some costs stay.  
A token holding a colon before a space, or a `#` after a space, needs quotes.  
A `'` inside a single-quoted pattern is written `''`, and indentation carries structure.  
YAML 1.1 readers turn `yes`, `no`, `on` and `off` into booleans; hakkiri reads 1.2, and its JSON schema types each field for editors.

Alternatives, each written as the same four rules and parsed with its own reader:

- KDL 2.0. Lost: it has no list syntax, so only a schema tells an empty or one-item list apart. Version 2 is from 2024-12.
- TOML 1.1. Lost: a multi-line inline table needs 1.1, which Python 3.14's reader rejected, and test cases repeat `[[rules.tests]]` headers.
- HCL. Lost: no raw strings, so every pattern doubles its backslashes, and `${` or `%{` in a message starts interpolation.
- Pkl and CUE. Lost: programming languages for a data file; an untyped empty list in Pkl evaluated to an object.
- RON. Lost: tooling exists only in Rust, and its raw strings keep indentation.
- JSON5, JSON and JSONC. Lost: no block strings, so a Markdown test input is one escaped string.
- Jsonnet, Nickel, Starlark and Dhall. Lost: the file runs as code before it is data.
- An own syntax. Lost: it needs its own parser, formatter and grammar, and GitHub would show it uncoloured until it is in wide use.

### 2. Vale's field names outside the selector

Every field outside the selector takes Vale's name (operator, 2026-11-09).  
The selector keeps ast-grep's terms under one key, `rule`, in place of Vale's `scope`.  
A file holds one rule, and its file name is the rule name, as in Vale: `ProseStyle/rules/Filler.yml` is `ProseStyle.Filler`.

The fields are `extends`, `message`, `level`, `description`, `link`, `tokens`, `exceptions`, `ignorecase`, `swap`, `max` and `condition`.  
Tests sit in the rule as `tests`, each with `name`, `input`, and `contains`, `absent` or `want`, as Vale's [test suite][vale-suite] reads them.

ProseStyle's 42 rules already use these names, so the importer maps most fields one to one.  
`extends` names the check type, and Vale's levels have no `off`; turning a rule off belongs in `.ctl/hakkiri.yaml`.

Alternative: ast-grep's names for the envelope (`id`, `severity`, `note`, `url`, `files`, `ignores`) with several rules per file.  
Lost: ast-grep has no names for prose checks, so the check fields would still come from Vale.  
Its tests are [`valid` and `invalid` lists][ast-grep-tests] that show only whether a rule fired, with the alerts in a separate snapshot file.

## Risks / Trade-offs

- [YAML's implicit typing differs between 1.1 and 1.2] → hakkiri reads 1.2 only. Its JSON schema lets an editor flag a list item that a 1.1 tool would read as a boolean.

[bootstrap]: ../archive/2026-11-09-bootstrap-hakkiri/design.md
[vale-suite]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/testsuite/suite.go#L28-L64
[ast-grep-tests]: https://github.com/ast-grep/ast-grep.github.io/blob/fe48cd89b24174a421505adf15604a49e740d3b2/website/guide/test-rule.md
