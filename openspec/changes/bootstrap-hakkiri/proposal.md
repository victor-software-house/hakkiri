# Proposal

Queue row: HKR-001 in [tasks.yaml][ledger].

## Motivation

Projects that use the [ProseStyle] package lint their prose with [Vale], and Vale sets limits we cannot change.  
It lints a list of files one after another, and its rules have no scope for prose outside code.  
ProseStyle's `BareURL` rebuilds fenced blocks and code spans out of lookbehinds, so a file of about 12,000 lines or a line of about 14,000 characters makes Vale panic.  
ProseStyle v0.0.6 took 249.5 s on one large project's prose files, and v0.0.7 still took 9.0 to 9.4 s after a rule redesign.  
[research.md][research] shows that no existing tool replaces Vale for this package, and that the Rust building blocks for a replacement are mature.

## Changes

- A new public repository for `hakkiri`, rendered from the shared Copier template as a Rust crate with OpenSpec, the Vale and ProseStyle prose lint, and the shared CI.
- A binary skeleton at version 0.0.0, so the first release is 0.0.1.
- The work queue in [tasks.yaml][ledger], one row per release 1 slice, each later slice planned as its own OpenSpec change.
- The research and the decisions from the 2026-11-09 interview, in [research.md][research] and [design.md][design].

## Capabilities

### New Capabilities

None. This change adds the repository and the plan, not lint behaviour.  
Each later slice adds the spec for the behaviour it builds.

### Modified Capabilities

None.

## Impact

- A new repository in the ctl group of CLIs, built on ctl-core.
- No change to any project yet. Projects keep Vale until the template switch in HKR-009.

[ProseStyle]: https://github.com/victor-software-house/prose-style
[Vale]: https://vale.sh
[ledger]: ../../../tasks.yaml
[research]: research.md
[design]: design.md
