# Proposal

Queue row: `HKR-026` in [tasks.yaml][ledger].
It follows [HKR-038][vale-repetition], which built `Vale.Repetition` and left `Vale.Spelling` for this row.

## Motivation

Vale builds two rules into its style `Vale`: `Vale.Repetition` and `Vale.Spelling` ([manager.go][v-manager]).
A project that lists `Vale` gets both, and turns the second off with `Vale.Spelling = NO`.
hakkiri built only the first, so a project that moves from Vale loses its spelling alerts, and the same project turns spelling off in Vale only to see no change in hakkiri.

## Changes

- The style `Vale` holds `Vale.Spelling`, a `spelling` rule with Vale's definition: the message `Did you really mean '%s'?`, the level `error` and the scope `text`.
  It uses the built-in dictionary, and a config sets its level, turns it off, and sets its parameters as it does any rule.
- It adds every dictionary in the `config/dictionaries` folder of each package, and accepts the words of each `.txt` file under the `config/ignore` folder of each package, as Vale reads them from its styles path ([spelling.go][v-spelling], [multi.go][v-multi], [config.go][v-config]).
- The `vale-differences` spec drops the gap, and records three places where hakkiri differs.
  It reads each package of the project where Vale has one styles path.
  It reads no word in a code span of a code comment, and no name after `@param`.
- The fixtures of the other `Vale` tests turn the rule off in the hakkiri configs, as their Vale configs already do.
- A Markdown file keeps a bare email address, a `www.` address and an angle-bracket email address in its text.
  Vale's converter keeps them there, since it writes a destination that differs from their text, and hakkiri dropped them as links, so the probe showed `custom` spelling a bare email address in Vale alone.
  The `vale-differences` spec drops the scenario that recorded it as a difference.

## Capabilities

### Modified Capabilities

- `spelling-check`: the built-in rule and its folders.
- `vale-differences`: the gap closes, three differences are recorded, and one is removed.
- `node-segments`: an address that is not its own destination stays in the text around it.

## Impact

- New: `tests/vale_spelling.rs` and `tests/fixtures/vale-spelling`, with 880 alerts that Vale at `66bbb85` gave under 11 configs, and the 35 alerts that only one side gives.
- Changed: `src/rule.rs` (`vale_spelling`), `src/rule/package.rs`, `src/rule/build.rs`, `src/config.rs`, `src/markdown.rs`, `src/instructions.md`, the skill and `AGENTS.md`.
- A project that lists the style `Vale` now gets spelling alerts at `error`, as it does in Vale.
- No new dependency.

[ledger]: ../../../../tasks.yaml
[vale-repetition]: ../2026-11-14-vale-repetition/proposal.md
[v-manager]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/manager.go#L498-L521
[v-spelling]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/spelling.go#L142-L173
[v-multi]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/spell/multi.go#L138-L152
[v-config]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/config.go#L125-L128
