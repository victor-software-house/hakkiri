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

The ProseStyle rules use `extends`, `message`, `level`, `scope`, `tokens`, `token`, `raw`, `nonword`, `exceptions`, `ignorecase`, `swap`, `max`, `formula` and `condition`; every other Vale field keeps its name too.  
Tests sit in the rule as `tests`, each with `name`, `input`, an optional `format` and `about`, and `contains`, `absent` or `want`, as Vale's [test suite][vale-suite] reads them.

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
Vale's code applies an exclusion to the element and its direct parent only; hakkiri applies it at any depth, as decision 7 lists.

Alternatives:

- ast-grep's rule object under `rule`: `kind`, `inside`, `has`, `not`, `all`, `any`. Lost: `kind` names a category without saying what varies, which ProseStyle's `KindAsNoun` rule flags, and the rule object replaces the scope every ProseStyle rule already uses.
- Vale's `scope` plus ast-grep's `inside`, `has`, `not`, `all` and `any`. Lost: two operator sets for one job, and no ProseStyle rule needs a deeper match than the scope operators give.
- Node names from mdast and nlcst, or from the tree-sitter grammars. Lost: they mix camelCase, PascalCase and snake_case, and tree-sitter's Markdown grammar has no node for a bare URL in text.

### 4. A rate is a metric that counts tokens

A habit rate is a `metric` rule with `tokens`, whose formula reads `occurrences`, the number of token matches (operator, 2026-11-09).  
`occurrences` is Vale's own word for a match in its `occurrence` check, and it is the one name this format adds to Vale's.  
With no scope, a metric measures the whole file, as [Vale's metric][vale-metric] does:

```yaml
extends: metric
message: "%s semicolons per 1,000 words, over 3: split the sentences."
level: warning
tokens:
  - ';'
formula: occurrences / words * 1000
condition: '> 3'
```

The [frequency research][habit-research] measured each habit per 1,000 words, so its numbers become thresholds directly.

Alternative: Vale's `occurrence` unchanged, at most N matches per paragraph or sentence.  
Lost: a 10-word and a 60-word paragraph get the same limit, and each threshold would need measuring again.

### 5. Release 1 implements every Vale check type but script

Release 1 implements the 11 [check types Vale defines][vale-types] other than `script` (operator, 2026-11-09).  
These are `existence`, `substitution`, `occurrence`, `metric`, `capitalization`, `consistency`, `conditional`, `repetition`, `sequence`, `spelling` and `readability`.  
`script` runs Tengo code, and a rule file stays data.  
This replaces the [bootstrap design][bootstrap] non-goal that left spelling, `conditional` and `sequence` out of release 1.

Alternative: only the four types ProseStyle uses today. Lost: a style that uses another type could not be imported.

### 6. The project config uses lowercase keys

`.ctl/hakkiri.yaml` holds the settings `.vale.ini` holds today, in lowercase keys like the other `.ctl/` configs and the rule files (operator, 2026-11-09):

```yaml
packages:
  - url: https://github.com/victor-software-house/prose-style/releases/download/v0.0.7/ProseStyle.zip
    sha256: 3d03b911…
level: warning
files:
  '*.{md,mdx,rs}':
    styles: [ProseStyle]
  '*.rs':
    ProseStyle.BareURL: off
  'target/**':
    styles: []
```

- `packages` is verctl's key; `url` and `sha256` are the key names in prose-style's [`upstream.toml`][upstream-toml]. A package whose download does not match its hash fails the run.
- `level` is the lowest level reported, Vale's `MinAlertLevel`.
- `files` maps a glob to its settings, as Vale's glob sections do.
- `styles` lists the styles a glob runs, Vale's `BasedOnStyles`; an empty list turns linting off for those files.
- A rule's name sets its level for those files, and `off` turns it off, where Vale writes `NO`.

Alternative: Vale's `.vale.ini` names unchanged in YAML, with each glob as a top-level key.  
Lost: the keys are PascalCase while the rule files are lowercase.

### 7. Intended differences from Vale

hakkiri differs from Vale 3.26.0 in these cases on purpose (operator, 2026-11-09).  
Each Vale behaviour below was measured with that release, or read in its source at `66bbb85`, which is 3.26.0 plus 16 commits.

| Case | Vale 3.26.0 | hakkiri |
|:--|:--|:--|
| A code span in a word count | Counts 0 words | Counts 1 word |
| `~` before a scope | Excludes the element and its direct parent | Excludes the element at any depth |
| Rust `/* */`, `/** */`, `/*! */` | Not read: [the Rust query][vale-rust] takes line comments only | Read as `text.comment.block` and `text.comment.doc.block` |
| String literals in code | Read by the `raw` scope with the rest of the file | Never prose |

Two measured cases need no difference.  
Link text counts as words and its target does not, in both.  
A bare URL or an autolink counts 0 words in Vale; ProseStyle's `BareURL` rule rejects either one anyway.

The string-literal case comes from the [bootstrap design][bootstrap].  
A sentence of two words, a four-word code span and one more word counts 3 words in Vale and 4 in hakkiri.

### 8. Every comment reads as Markdown

hakkiri reads every comment as Markdown, plain `//` and `/* */` comments included (operator, 2026-11-10).  
This replaces the [bootstrap design][bootstrap] rule that plain comments are text.  
A code span, a link and a bare URL then mean the same in a comment as in a `.md` file.  
ProseStyle's [`BareURLInComment`][bare-url-in-comment] test already expects a code span in a plain `//` comment to pass.

So four rules that rebuild code spans, fences and comments with `raw` and lookbehind become two scope rules:

| ProseStyle today | In hakkiri |
|:--|:--|
| `BareURL`, `BareURLInComment` | `BareURL`: `scope: link`, a token for a URL at the start of the link text |
| `BareWorkItem`, `BareWorkItemInComment` | `BareWorkItem`: `scope: text & ~link` |

GFM reads a bare URL as a link whose text is the URL, so `scope: link` finds bare URLs and autolinks and skips `[the page](…)`.

Alternative: plain comments stay text, and the comment rules stay separate.  
Lost: a backticked URL in a `//` comment would be flagged, which breaks ProseStyle's own test.

### 9. A table row is a scope

`table.row` holds the text of one table row, next to Vale's `table.header` and `table.cell` (operator, 2026-11-10).  
`TableRow` becomes a metric on Vale's `characters` variable:

```yaml
extends: metric
message: "This table row is over 250 characters: make it a list or a section."
level: warning
scope: table.row
formula: characters
condition: '> 250'
```

A row's text leaves out the pipes and padding, so the limit counts cell text only.

Alternative: `TableRow` keeps `scope: raw` and its regex. Lost: the rule would still find a row by parsing raw text.

## Risks / Trade-offs

- [YAML's implicit typing differs between 1.1 and 1.2] → hakkiri reads 1.2 only. Its JSON schema lets an editor flag a list item that a 1.1 tool would read as a boolean.
- [Front matter, link definitions, raw HTML and MDX have no scope] → the first rule that needs one adds a lowercase dotted name to the spec.
- [`sequence` needs part-of-speech tags and `spelling` needs dictionaries] → the engine slice names the crate or data for each before it starts.

[bootstrap]: ../archive/2026-11-09-bootstrap-hakkiri/design.md
[vale-suite]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/testsuite/suite.go#L28-L64
[ast-grep-tests]: https://github.com/ast-grep/ast-grep.github.io/blob/fe48cd89b24174a421505adf15604a49e740d3b2/website/guide/test-rule.md
[prose-style-rules]: https://github.com/victor-software-house/prose-style/tree/ee9f207279864e305d7372aef4a6c5dacc3eb846/ProseStyle/styles/ProseStyle
[vale-scope]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/scope.go#L68-L187
[vale-comments]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/code/query.go#L103-L107
[bare-url-in-comment]: https://github.com/victor-software-house/prose-style/blob/ee9f207279864e305d7372aef4a6c5dacc3eb846/ProseStyle/styles/ProseStyle/BareURLInComment.yml
[vale-rust]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/code/rs.go#L10-L16
[vale-metric]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/metric.go#L53-L63
[vale-types]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/definition.go#L119-L132
[upstream-toml]: https://github.com/victor-software-house/prose-style/blob/ee9f207279864e305d7372aef4a6c5dacc3eb846/upstream.toml
[habit-research]: https://github.com/victor-software-house/prose-style/blob/ee9f207279864e305d7372aef4a6c5dacc3eb846/openspec/changes/frequency-habit-rules/research.md
