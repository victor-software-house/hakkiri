# Design

## Context

Vale builds an alert's message with `FormatMessage`, which calls `CondSprintf`: the format gets one more verb, `%[N]s` for the number of values plus one, and then goes to Go's `Sprintf` ([util.go][v-format]).
The extra verb makes Go read the format as reordered, so a value that no verb takes is dropped where Go would print `%!(EXTRA ...)`.
Every value is a string, so `%s` and `%[n]s` are the verbs a rule can use.
`%d` reads as `%s` in the `occurrence` rule only, and hakkiri already does that.

## Goals / Non-Goals

Goals:

- A rule message with `%[n]s` gives the text Vale gives.
- The ProseStyle v0.0.14 cases of `NounStack` and `ThingsSpeakNoun` pass.

Non-goals:

- Verbs other than `%s`, `%d` in an `occurrence` rule and `%%`: Vale prints `%!q(string=...)` and the like for them, and no rule uses them.
- Go's width and precision flags.

## Decisions

### 1. Fill follows Go's rule for the next value

`%[n]s` takes the `n`th value and the next unindexed verb takes the value after it ([fmt docs][go-fmt]).
The probe of Vale at `66bbb85` (2026-11-11) printed `green blue red green.` for `%[2]s %s %[1]s %s.` over `red green blue`, and `red, red and green.` for `%s, %[1]s and %s.` over `red green`.
Alternative: read `%[n]s` as a fixed value that leaves the plain verbs where they were.
It lost because the probe shows Go moves its place, and a message that mixes the two forms would differ.

### 2. A verb with no value is nothing

The probe printed `[][%!s(BADINDEX)][%!s(BADINDEX)][%!s(MISSING)]...` for `[%[3]s][%[4]s][%[0]s][%s]...` over two values.
Vale's extra value is an empty string, so `%[3]s` is empty, and the verbs after the values are `BADINDEX` and `MISSING`.
hakkiri already read a plain `%s` with no value as nothing, and an indexed verb with no value does the same.
Alternative: print Go's `%!s(MISSING)` text.
It lost because it puts a Go error string into a message that a reader sees, and no rule that works in Vale depends on it.

[v-format]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/util.go#L153-L163
[go-fmt]: https://pkg.go.dev/fmt#hdr-Explicit_argument_indexes
