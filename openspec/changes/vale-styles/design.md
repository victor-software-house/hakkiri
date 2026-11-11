# Design

## Context

A Vale rule file is YAML with Vale's field names, and hakkiri reads the same names.
Run unchanged against the pinned styles, hakkiri passes all 29 cases of Std and 66 of Voices, and fails only on four rules that inherit and on the one script rule of Readability.
The probes ran Vale `66bbb85` in a throwaway Linux box, over 13 chains that cover each edit and each refusal ([inheritance][v-inherit], [scripts][v-script]).
Vale resolves `extends: Style.Rule` by path, overlays the child's keys, and fails the whole run on a bad chain.

## Decisions

### 1. The loader resolves inheritance

A rule whose `extends` holds a dot is a rule name.
The loader reads the child into a generic tree, reads each ancestor the same way, overlays them, and builds the rule from the result.
The overlay is Vale's: a bare key replaces, `key+` appends to a list or merges into a mapping, `key-` removes entries by their text or keys by name, and the parent's `tests` are dropped at every level.
The root's `extends` gives the check type, and the child's `extends` is consumed.

Timing, measured with the repository's release profile in a throwaway Linux box:

- `hakkiri check` on a one-line file over Voices and Std, with the four inheriting rules written out by hand, takes 35.5 ms (200 runs).
  That run is the load without inheritance, since the written-out files hold what inheritance gives.
- The extra work with inheritance, parsing each child and its parent into a tree and overlaying them for the four rules, takes 131.7 microseconds (20,000 rounds).
  That is 0.4% of the run, about 33 microseconds a rule.
- The written-out rules pass all 103 cases of the two styles, so the overlay reproduces them.

Alternative: convert the style files with a command, so the loader never sees inheritance.
Lost: the operator asked for the loader, a converter would hold a second copy of Vale's overlay rules, and every style file would need a conversion before it ran (operator, 2026-11-11).

### 2. The merged tree is written as JSON and read by the same reader

The loader needs a typed rule from the merged tree.
It writes the tree as JSON text, which is valid YAML 1.2, and reads that text through the reader every rule file uses, so a field reads with the same checks and the same messages.
An error in a merged rule names the file and the field and no line, since the text has no line a person wrote.
A rule with no `+` or `-` and no dot in `extends` is read from its own text and keeps its lines.

Alternative: a serializer for the tree, or a `Deserializer` for the tree.
Lost: the first adds a dependency for output nobody reads, and the second duplicates the checks of the reader.

### 3. A parent is found in any loaded package

Vale looks a parent up on its search path, which holds every styles folder, and the parent need not be enabled.
hakkiri puts every package folder of the config on the search path before it reads the first style, so `Direct.Length` finds `Std` whichever package holds it.
The first segment of the name is a style folder, and the rest are folders and a file name.
A `_` folder and a nested folder both resolve, as `GenZ._shared.Slang` needs.
`hakkiri test` searches the packages of the config, and the rule's own package when the config does not name it.

Alternative: look only in the rule's own package.
Lost: Voices extends a rule of Std, which another package holds.

### 4. A rule hakkiri cannot run is reported apart

Today one script rule refuses its style, which is worse than Vale, since Vale runs the script.
The loader now sorts a refusal in two.
A rule that is invalid, such as an unknown field or a bad pattern, still refuses the load.
A rule that is valid Vale and that hakkiri does not run, a `script` rule or a rule with a `doc()` scope, gives no rule, keeps its name, and lets the rest of the style load.
A rule that extends a script rule is one too.

The config then decides:

| | L1 (before) | L2 (this change) | L3 |
|:--|:--|:--|:--|
| the rest of the style | refused with it | runs | runs |
| the rule, in a listed style | the whole style fails | a `failed` line and exit 2, until the config sets it `off` | a warning line, exit unchanged |
| `Readability` with `Polysyllables` | unusable | 7 rules run, one `failed` line a run until it is `off` | 7 rules run, a line each run |
| catch | worse than Vale | a project opts out by name | a rule that never ran passes silently |

L2 fails closed and loses nothing else, and `off` is the explicit acknowledgement (operator, 2026-11-11).
The rule counts as enabled the way a rule does: its style is listed, or a level or `on` turns it on.
In a style no glob lists it is ignored, which is how a clash in an unlisted style is already treated.
A level or `on` does not make it run, so it still fails the run.
Each such rule fails a run once, however many files the config runs it on.
`hakkiri test` reports the cases of such a rule as not run, and a case through the config that runs one as not run.

### 5. `import vale` is for the `.vale.ini`

With the loader reading Vale styles, a command for the rule files has nothing left to do.
The name `hakkiri import vale` stays for converting the `.vale.ini` into `.ctl/hakkiri.yaml`, which is a row of its own with its command shape decided: `hakkiri import vale [SOURCE]`, Vale's upward lookup with no argument, the file `--config` names as output, `--force` to overwrite, and `--dry-run` to preview (operator, 2026-11-11).

### 6. What hakkiri still does not run

Each is a Known gap with a row:

| Vale feature | row | size | use found |
|:--|:--|:--|:--|
| `script` rules (Tengo) | `HKR-036` | about 2,000 to 2,500 lines for the subset Vale scripts use; no maintained Rust Tengo exists | 1 of 161 rule files in eight popular styles, and 5 of ProseStyle's own, which `HKR-008` ports to native rules |
| a metric formula past arithmetic | `HKR-037` | about 150 lines | 0 of 10 formulas |
| the built-in `Vale.Repetition` | `HKR-038` | one embedded rule | not measured |
| `doc()` scopes and Views | `HKR-039` | large: Markdown rendered to HTML as Vale renders it, and a CSS selector engine | 0 |
| a `.vale.ini` | `HKR-035` | the conversion, with a command | every Vale project |
| `action` | `HKR-011` | none until fixes exist | loads, applies nothing |

[v-inherit]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/inherit.go
[v-script]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/script.go
