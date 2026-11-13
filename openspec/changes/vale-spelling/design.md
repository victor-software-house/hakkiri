# Design

## Context

A `spelling` rule already runs in hakkiri, from a rule file, with Vale's tokenizer, filters, dictionaries and messages.
What was missing is the rule that Vale registers for its style `Vale`, with the defaults that only that rule has.

## Goals / Non-Goals

Goals:

- `styles: [Vale]` runs `Vale.Spelling` with the alerts Vale gives.
- A config sets its level, turns it off and sets its parameters as it does any rule.

Non-goals:

- A dictionary for another language.
- Reading `DICPATH`: hakkiri reads dictionaries from packages only.

## Decisions

### 1. The rule is a rule file read as text, named for Vale's definition

`VALE_SPELLING` holds Vale's definition ([definition.go][v-definition]) and `vale_spelling` reads it as `vale_repetition` reads its own.
The rule then takes parameters through the path of any rule file, and `Vale.Spelling[split] = YES` is `params: {Vale.Spelling: {split: true}}`.
Alternative: build the check in code.
It lost because a parameter would then need a second path.

### 2. The built-in rule reads the settings of every package

Vale reads `config/dictionaries` and `config/ignore` from its one styles path, and only for the rule named `Vale.Spelling` ([spelling.go][v-spelling], [multi.go][v-multi]).
hakkiri has a list of package folders, so the rule reads each folder in order, and the first folder that holds a file a parameter names is the one it reads.
The name `Vale.Spelling` is the test, as it is in Vale, and a style named `Vale` is refused, so no rule file can take the name.
Alternative: read the first package only.
It lost because a package that adds a word list would change with the order of the others.

### 3. A dictionary of a folder adds to the built-in one

Vale loads the built-in dictionary, and then each `.dic` in `config/dictionaries`, so a word any of them holds passes.
A rule that names `dictionaries`, `aff` or `dic` of its own reads those alone, plus the built-in one with `append`.
The probe at `66bbb85` shows both: a dictionary of two words in `config/dictionaries` adds them, and `Vale.Spelling[dictionaries] = team` leaves every other word misspelt.

### 4. Word lists are every `.txt` file under `config/ignore`

The lists are read from the folder and the folders below it, and a file with another extension is no list.
A word of a list passes in the case the list gives it and in the case of its capitalised form, as it does in a rule's own `ignore`.

### 5. What the probe shows and hakkiri differs on

The probe of Vale at `66bbb85` over four Markdown, MDX, Rust and TypeScript files and 11 configs gave 880 alerts.
hakkiri gives the same, with these exceptions, which it keeps on purpose:

- A word with a curly quote or `&rsquo;` is reported where it stands, where Vale gives it no position. A rule file that names its own dictionary has the same difference.
- A code span in a code comment is no word, where Vale reads its backticks as part of a word when `custom` turns the filters off.
- A name after `@param` is no prose, where Vale reads it, and reads the tag itself with `custom`.

### 6. The rule is built when a glob uses it

Vale builds a rule when a config uses it, so a dictionary that cannot load fails no run of a config that never spells.
hakkiri builds `Vale.Spelling` only when a glob of some config file lists `Vale` in `styles`, names a rule that starts with `Vale.`, or sets `params` for one.
A project with a package style and a stray `.dic` file in `config/dictionaries` loads and lints.
The same project fails with a fault that names the missing `.aff` file once a glob lists `Vale`.
Alternative: build it on every load.
It lost because a package with a broken dictionary then failed every run, including runs that never spell.

[v-definition]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/definition.go#L170-L182
[v-spelling]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/spelling.go#L142-L173
[v-multi]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/spell/multi.go#L138-L152
