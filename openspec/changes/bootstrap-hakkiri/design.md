# Design

## Context

The motivation is in [proposal.md][proposal]; the evidence is in [research.md][research].

ProseStyle turns on 42 rules of four Vale types: existence, substitution, occurrence and metric.  
Projects see 37 of them, because the template sets `MinAlertLevel = warning` and five rules are suggestions.  
The Tengo script rule and the conditional rule ship in the package and stay off.  
The operator settled the decisions below on 2026-11-09.

## Goals / Non-Goals

**Goals:**

- Release 1 gives projects the alerts Vale and ProseStyle v0.0.7 give them, plus rate rules.
- The intended differences from Vale: Rust block comments are linted, string literals are never read, and no file makes a run panic.
- The rule format is as clear and as powerful as the best tools in the field, built on them rather than on Vale's syntax.

**Non-Goals:**

- A language server in release 1.
- Reading `.vale.ini` or Vale rule files at run time.
- Human-language profiles, spelling dictionaries, script rules, and Vale's `conditional` and `sequence` types.
- A Vale issue about parallel file arguments.

## Decisions

### 1. Own format, built from the best prior art

hakkiri reads its own rule files and `.ctl/hakkiri.yaml`.  
Structure terms come from [ast-grep's rule object][ast-grep]: `kind`, `inside`, `has`, `not`, `all`, `any`.  
Prose checks come from Vale's model: word lists, swaps, counts, metrics, plus rates.

File globs and rule levels follow flat lint configs such as ESLint and oxlint.  
Vale styles enter only through an importer that a package build runs.  
The format slice, [`HKR-002`][ledger], settles the field names, with every candidate shown side by side.

Alternatives:

- Read Vale files as they are. Lost: `.vale.ini`, the rule schema and packages are Vale's own, read by nothing else, and the library holds 27 packages. Keeping them keeps `BareURL`'s lookbehinds.
- Vale files plus our extensions. Lost: a rule that uses an extension stops running under Vale anyway, so it buys no compatibility and keeps Vale's syntax.
- Own format, plus a Vale reader at run time. Lost: two readers, and the second one must track every Vale schema change.

### 2. Structure first

A rule selects nodes in the document tree, and a pattern only narrows that selection.  
A rule that needs a regular expression to learn whether text sits in code, a link or a comment points to a missing node kind.

Alternative: patterns first with scopes, as Vale does. Lost: it is the cause of the lookbehind rules, the panics and the run time in the proposal.

### 3. Release 1 is parity plus rate rules

Release 1 reaches parity on Markdown, MDX and Rust comments, and adds rate rules: a habit counted per sentence or per number of words.  
The frequency-habit rules in prose-style ship with it.

Alternatives:

- Parity only. Lost: the frequency-habit rules would wait for a later release, and the format would gain a rule type after projects use it.
- Markdown first, then grow. Lost: projects would run two linters side by side for several releases.

### 4. Profiles for programming languages

Each source language has a profile that knows where its prose lives and in what syntax.  
Rust doc comments are Markdown; plain comments are text; TSDoc and JSDoc have a summary and tag text; string literals are never prose.  
Release 1 ships Rust, TypeScript, JavaScript, Markdown and MDX; more languages follow as profiles on tree-sitter grammars.

Alternatives:

- Human-language profiles, such as Portuguese next to English. Lost: every project we lint today is written in English.
- Both kinds. Lost: a larger design with no user yet.

### 5. Rules ship as versioned packages

prose-style stays its own repository and publishes a package per tag.  
A project pins it by version and hash in `.ctl/hakkiri.yaml`, and hakkiri fetches and caches it with no separate sync step.

Alternatives:

- Rules built into the binary. Lost: every rule fix would need a linter release, and other teams could not publish rules.
- Built in, with packages allowed. Lost: two ways to get the same rules.

### 6. The CLI first, the editor next

Release 1 is `hakkiri check` and `hakkiri test`, run from the `lint:prose` mise task, CI and a pre-commit hook on staged files.  
The engine keeps a per-file entry point, and `hakkiri lsp` is release 2.

Alternative: a language server in release 1. Lost: no project uses a prose check in an editor today, and it would delay the switch.

### 7. The name is hakkiri

はっきり means "clearly, plainly", the result the rules push toward.  
The name is free on crates.io and Homebrew, and 9 GitHub repositories carry it, none of them a linter.  
The config lives at `.ctl/hakkiri.yaml`, beside the other ctl CLIs' files.

Alternatives: `prosectl` (the operator ruled out a `ctl` suffix), `shuhitsu`, `tadasu`, `naosu`, `akaji` and `shiteki`.  
Names already taken on crates.io were left out: `tensaku`, `suiko`, `akapen`, `fude`, `kotoba`.

### 8. Two regex engines, with a limit

Patterns compile with the `regex` crate, which runs in linear time.  
Only a pattern that needs lookaround uses `fancy-regex`, with a backtrack limit; reaching it is an error alert for that file.

Alternative: one backtracking engine for every pattern, as Vale does with regexp2. Lost: its limit panics the whole run.

### 9. Templates, OpenSpec and qctl

The repository is rendered from the shared Copier template.  
Each slice is an OpenSpec change named by one row in [tasks.yaml][ledger], moved with qctl.  
verctl releases start with the CLI slice, `HKR-005`, because prose-style CI and the template install hakkiri through mise.

Alternative: a free-form plan file. Lost: OpenSpec and qctl already hold the plan, the evidence and the queue, and `qctl check` keeps them in step.

## Risks / Trade-offs

- [markdown-rs has had no release since 2025-04] → It is the only Rust parser with MDX and positions on every node.
  The tree slice, `HKR-003`, tests its positions on tables and list items first, and comrak is the fallback for `.md`.
- [Sentence boundaries decide `LongSentence` and `Direct.Length`] → The engine slice, `HKR-004`, tests the drift against Vale, and the alert diff in `HKR-008` shows the effect.
- [Converting upstream styles ties each upstream bump to the importer] → The importer slice, `HKR-007`, covers only the types those styles use.
- [Whether `fancy-regex` compiles ProseStyle's six lookbehind patterns unchanged is not verified] → The format slice, `HKR-002`, should remove most of them with structure-first rules.

## Migration Plan

Each slice is a row in [tasks.yaml][ledger].

| Row | Slice | Proof |
|:--|:--|:--|
| `HKR-001` | This change: repository, queue, research, design | CI passes on the rendered template; `qctl check` and `openspec validate --strict` pass |
| `HKR-002` | Rule and config format | The operator approves the format; every ProseStyle rule is written in it |
| `HKR-003` | Document tree and language profiles | Snapshot tests of trees and spans |
| `HKR-004` | Rule engine | Unit tests per check; no panic on a 50,000-character line |
| `HKR-005` | `check` and `test`, released through verctl | Integration tests; a release mise installs |
| `HKR-006` | Packages pinned by hash | Fetch, cache and mismatch tests |
| `HKR-007` | Vale importer | Converted rules pass their own Vale cases |
| `HKR-008` | ProseStyle on hakkiri, with rate rules | Every case passes; the alert diff shows only intended differences |
| `HKR-009` | Template switch | The Template check and a pilot project pass |
| `HKR-010` | Release 1 | Every adopted project passes CI |

Rollback: until the template switch, `HKR-009`, no project depends on hakkiri; after it, a project returns to Vale by taking the previous template tag.

[proposal]: proposal.md
[research]: research.md
[ledger]: ../../../tasks.yaml
[ast-grep]: https://ast-grep.github.io/reference/rule.html
