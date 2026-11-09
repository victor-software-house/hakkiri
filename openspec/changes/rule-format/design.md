# Design

## Context

The [bootstrap design][bootstrap] fixed the rule model.  
A rule selects text, then runs a Vale-style check or a rate over it.  
The bootstrap took the selector from ast-grep; decision 3 replaces it with Vale's `scope`.  
The project config lives in `.ctl/hakkiri.yaml`.

Each decision below was shown to the operator with every candidate side by side, written as the same four ProseStyle rules.

## Goals / Non-Goals

**Goals:**

- One format that holds every enabled ProseStyle rule and every frequency-habit rule, with no lookbehind that rebuilds structure.
- Field and scope names taken from Vale, as ProseStyle's rules already write them, cited.

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

### 2. Vale's field names

Every field takes Vale's name (operator, 2026-11-09).  
A file holds one rule, and its file name is the rule name, as in Vale: `ProseStyle/rules/Filler.yml` is `ProseStyle.Filler`.

The fields are `extends`, `message`, `level`, `description`, `link`, `scope`, `tokens`, `exceptions`, `ignorecase`, `swap`, `max` and `condition`.  
Tests sit in the rule as `tests`, each with `name`, `input`, and `contains`, `absent` or `want`, as Vale's [test suite][vale-suite] reads them.

The [25 ProseStyle rules][prose-style-rules] already use these names, so the importer maps most fields one to one.  
`extends` names the check type, and Vale's levels have no `off`; turning a rule off belongs in `.ctl/hakkiri.yaml`.

Alternative: ast-grep's names for the envelope (`id`, `severity`, `note`, `url`, `files`, `ignores`) with several rules per file.  
Lost: ast-grep has no names for prose checks, so the check fields would still come from Vale.  
Its tests are [`valid` and `invalid` lists][ast-grep-tests] that show only whether a rule fired, with the alerts in a separate snapshot file.

### 3. Vale's scope selects the text

A rule selects text with Vale's `scope` key, Vale's scope names and Vale's operators, and nothing else (operator, 2026-11-09).  
Names are lowercase and dotted, as the ProseStyle rules write them: `sentence`, `paragraph`, `heading`, `list`, `table.header`, `raw`, `raw.rs`.  
Code comments are `text.comment.line` and `text.comment.block`, with `doc` added for doc comments, as in [Vale's comment scopes][vale-comments].

`~` before a name excludes that element and the text inside it, `&` requires both parts, and a list matches any item ([scope.go][vale-scope]):

```yaml
scope: sentence & ~blockquote
```

Vale has no scope names for front matter, link definitions, raw HTML or MDX.  
No ProseStyle rule needs one, so the spec adds none until a rule does.  
Vale's code applies an exclusion to the element and its direct parent only.  
Whether hakkiri applies it at any depth is settled with the other intended differences.

Alternatives:

- ast-grep's rule object under `rule`: `kind`, `inside`, `has`, `not`, `all`, `any`. Lost: `kind` names a category without saying what varies, which ProseStyle's `KindAsNoun` rule flags, and the rule object replaces the scope every ProseStyle rule already uses.
- Vale's `scope` plus ast-grep's `inside`, `has`, `not`, `all` and `any`. Lost: two operator sets for one job, and no ProseStyle rule needs a deeper match than the scope operators give.
- Node names from mdast and nlcst, or from the tree-sitter grammars. Lost: they mix camelCase, PascalCase and snake_case, and tree-sitter's Markdown grammar has no node for a bare URL in text.

## Risks / Trade-offs

- [YAML's implicit typing differs between 1.1 and 1.2] → hakkiri reads 1.2 only. Its JSON schema lets an editor flag a list item that a 1.1 tool would read as a boolean.
- [Front matter, link definitions, raw HTML and MDX have no scope] → the first rule that needs one adds a lowercase dotted name to the spec.

[bootstrap]: ../archive/2026-11-09-bootstrap-hakkiri/design.md
[vale-suite]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/testsuite/suite.go#L28-L64
[ast-grep-tests]: https://github.com/ast-grep/ast-grep.github.io/blob/fe48cd89b24174a421505adf15604a49e740d3b2/website/guide/test-rule.md
[prose-style-rules]: https://github.com/victor-software-house/prose-style/tree/ee9f207279864e305d7372aef4a6c5dacc3eb846/ProseStyle/styles/ProseStyle
[vale-scope]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/scope.go#L68-L187
[vale-comments]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/code/query.go#L103-L107
