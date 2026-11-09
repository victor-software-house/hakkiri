# hakkiri

A prose linter for Markdown, MDX, and source comments, built on ctl-core.  
This file is the development contract.  
The operator contract will be `hakkiri instructions` and `skills/hakkiri/SKILL.md`.

## Decisions

1. **Own format.**
   hakkiri reads its own rule files and `.ctl/hakkiri.yaml`, not `.vale.ini` or Vale rule files.  
   The format takes its structure terms from one source, ast-grep (`kind`, `inside`, `has`, `not`, `all`, `any`), and its prose checks from Vale's model: word lists, swaps, counts, metrics, plus rates.  
   Vale styles enter only through `hakkiri import vale`, which a package build runs.
2. **Structure first.**
   A rule selects nodes in the document tree; a pattern only narrows that selection.  
   A rule that needs a regular expression to find out whether text sits in code, a link or a comment is a missing node kind, not a rule to write.
3. **Language profiles.**
   Each source language has a profile that maps its comments and doc comments into the tree: rustdoc as Markdown, TSDoc and JSDoc with their tags, plain comments as text.  
   String literals are never prose.  
   The first release ships Rust, TypeScript, JavaScript, Markdown and MDX.
4. **Matching.**
   The `regex` crate by default; `fancy-regex` with a backtrack limit only for a pattern that needs lookaround.  
   A limit reached is an error alert for that file, never a panic.
5. **Packages.**
   Rules ship as versioned packages that a project pins by version and hash.  
   hakkiri fetches and caches them; there is no separate sync step.
6. **Release scope.**
   Release 1 is the CLI: `check` and `test`, run from a mise task, CI and a pre-commit hook.  
   Release 2 is the language server, on the same per-file engine.

## Workflow

- Work is queued in `tasks.yaml` and moved with qctl; never edit the ledger by hand.
- Each queued slice is an OpenSpec change under `openspec/changes/`, archived into `openspec/specs/` when it merges.
- The repository is rendered from the shared Copier template; change rendered files through a template update.

## Rules

- ctl's shared rules apply: always a pull request, never `--no-verify`, ctl-core owns presentation, every flag has a `HAKKIRI_` variable, and nothing local leaks into a tracked file.
- The pull request's CI verifies. Do not build or test before a push.
- A behaviour change moves code, tests, `src/instructions.md`, `skills/hakkiri/SKILL.md`, this file and its OpenSpec change together (the ctl bundle contract).
- Prose in this repository follows ProseStyle: plain words, short sentences, no dashes as punctuation.
