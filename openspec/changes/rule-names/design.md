# Design

## Context

A rule's name is its identity: a config key, an alert and a coverage report all use it.
Vale names a rule in one function, `CheckName`, and loads a style with a walk ([name][v-name], [walk][v-walk]).
hakkiri names one in `rule_name`, which the loader and `test` share, and loads only the files directly in a style folder.

## Decisions

### 1. A name ends at the first dot of the file name

`CheckName` splits the file name at its first dot, so `Terms.custom.yml` is `Team.Terms` ([name][v-name]).
hakkiri follows it, and `Foo.v2.yml` is `Team.Foo` (operator, 2026-11-10).

Alternative: keep the last dot, so the file keeps `v2` in its name.
Lost: a Vale style would give other names in hakkiri, so a config key such as `Team.Terms = NO` would miss its rule.

### 2. The loader reads every folder under a style

Vale walks the style folder and joins each folder into the name, so `Team/Sub/Bar.yml` is `Team.Sub.Bar` ([walk][v-walk]).
The walk skips a folder that starts with `.` or `_`, so drafts and retired rules stay inert, and a `*.test.yml` file, since it holds cases.
The walk takes the entries of each folder by name, so the rules come in Vale's order.

Alternative: read only the files directly in the folder.
Lost: a nested rule would load in Vale and not in hakkiri, in `check` and in `test` alike.

### 3. Two files with one name are refused

Once a name ends at the first dot, `Terms.yml` and `Terms.custom.yml` give one name.
The loader refuses the second file with an error that names both files and the rule name, when a glob lists the style.
Vale refuses it too once a style enables it: its registration fails with "the rule has already been added" ([register][v-register]), and a probe of Vale `66bbb85` gave that error for `Terms.yml` with `Terms.custom.yml`, and for `Sub/Bar.yml` with `Sub/Bar.v2.yml`.
The same probe gave no error when no `BasedOnStyles` named the style, and linted with exit 0.
hakkiri does the same: it reads the faults in the names of every style, and refuses those of a style a glob lists in `styles`.
A rule key set under a glob does not enable a style, since hakkiri runs a rule only from a listed style.

Alternative: let the first file win, silently.
Lost: a rule would vanish with no sign, and its tests would pass against the wrong file.

### 4. `test` finds the style folder as Vale does

Vale names an isolated rule file from the search paths of the config: the style is the first folder under a search path that holds the file, and the folders after it join the name.
A file under no search path takes the folder that holds it ([isolated name][v-isolated]).
`test` does the same with the packages of the project config, read only for this, and gives the style folder's parent to the rule as its package.

Alternative: always take the folder that holds the file.
Lost: `test` would name `Team/Sub/Bar.yml` as `Sub.Bar` and `check` as `Team.Sub.Bar`, so a coverage report and an alert would name one rule two ways.

### 5. A bracket in a name is refused

Vale refuses a name with `[` or `]`, since `Style.Rule[param]` is config syntax ([name][v-name]), and the probe gave "contains brackets, which rule names cannot" for `Weird[max].yml` in an enabled style.
The loader refuses it with an error that names the file, under the same condition as a clash.

Alternative: accept it.
Lost: a config key for such a rule would be ambiguous.

## Risks / Trade-offs

- [A project that relied on `Foo.v2` as a name] → hakkiri is not released, and no config of ours names such a rule.
- [A config that fails to load leaves `test` with no packages] → `test` falls back to the folder that holds the file, as Vale does with no search path.

[v-name]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/util.go#L218-L235
[v-walk]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/manager.go#L198-L315
[v-register]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/manager.go#L435-L462
[v-isolated]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/testsuite/run.go#L320-L346
