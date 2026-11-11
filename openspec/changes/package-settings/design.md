# Design

## How Vale 3.26.0 merges a package's `.vale.ini`

Read at Vale commit `66bbb85`, and probed with a build of that commit (section "Probes").

1. `vale sync` unpacks a package and moves its `styles` folder into `StylesPath`.
   The `config` folder inside it is merged with the project's own `config` folder ([installPkg][v-install], lines 133 to 152).
2. It copies the package's `.vale.ini` to `StylesPath/.vale-config/<index>-<Name>.ini`, where the index is the package's place in `Packages` ([line 170][v-copy]).
   Every sync starts by deleting that folder ([lines 26 to 30][v-clean]).
3. If the package's `.vale.ini` holds `Packages`, `sync` installs those too ([lines 155 to 164][v-nested]).
4. At lint time Vale reads the fragments first, sorted by file name as text, then the user-level config, then the project's ([ReadPipeline][v-pipeline]; [pipeConfig][v-pipe]; the sort is [one line][v-sort]; the user-level file is added at [line 203][v-user] and the project's at [line 234][v-project]).
5. The sources merge key by key.
   The first source holds the key and each later value is added to it as a shadow ([shadowMerge][v-shadow]).
   How a key then resolves depends on the code that reads it:
   - a rule's level reads the last value, so the project wins over a package ([lastValue][v-last]);
   - `BasedOnStyles` and `Vocab` read all values and drop everything before a `NONE` ([valuesAfterNone][v-none]), so they add up;
   - `MinAlertLevel` reads the last value ([line 413][v-min]);
   - `WordTemplate` and the `[formats]` entries read the first value, so a package beats the project ([line 434][v-wt], [line 672][v-formats]).
6. A package's `StylesPath` is dropped on load ([expandPaths][v-expand], line 472).
7. A key Vale does not know is a warning, not an error ([line 654][v-unknown]).
8. A file's sections are visited in the order written and a later section's `BasedOnStyles` replaces the styles of an earlier one.
   When a project section replaces what a package section turned on, Vale warns once ([buildFile][v-build], [warnDroppedStyles][v-warn]).
   Sections with the same name merge before that, so their `BasedOnStyles` add up.

## Probes

A build of Vale `66bbb85` in a throwaway box, two local packages and a project, run with `vale sync`, `vale ls-config` and `vale`.
`PkgA` ships `[*.md] BasedOnStyles = PkgA`, `PkgA.Gamma = NO`, `PkgA.Bet = error`, `MinAlertLevel = error`, `Vocab = PkgA`, `WordTemplate = %s`, a `StylesPath` and an unknown key.
`PkgB` ships `[*.md] BasedOnStyles = PkgB`, `PkgA.Bet = suggestion` and `PkgB.Fil = error`.

| Case | Project | Observed |
|:--|:--|:--|
| Package alone | `[*.md] BasedOnStyles = Own` | styles `PkgA` and `Own` both run; `PkgA.Gamma` is off and `PkgA.Bet` is an error |
| Project overrides | `PkgA.Bet = suggestion`, `PkgA.Gamma = YES`, `MinAlertLevel = suggestion` | `Bet` is a suggestion, `Gamma` runs, the project's minimum level applies |
| Project is silent on the minimum | none | the package's `MinAlertLevel = error` applies |
| `NONE` | `BasedOnStyles = NONE, Own` | only `Own` runs, and the package's rule settings stay |
| Another section name | `[*.{md,txt}] BasedOnStyles = Own` | `Own` replaces `PkgA` for `a.md`, and Vale warns that the section replaces the styles of the PkgA package |
| Two packages | `Packages = PkgA, PkgB` | `PkgA.Bet` is a suggestion (the later package); reverse the list and it is an error |
| Scalar read first | `WordTemplate = \b(?:%s)\b` | the package's `%s` wins |
| Vocabularies | `Vocab = PkgB` with both packages | `PkgA` and `PkgB` both load; `Vocab = NONE` leaves none |
| Package keys | a `StylesPath` and an unknown key in `PkgA` | no error for the first, a warning for the second |

## Decisions

### 1. Package settings are the earliest layer of the project's config

The settings of each package, in the order of `packages`, are applied as if written ahead of the project's own `files` entries.
The existing rules then give the result: a later glob wins for a rule, and the project is last.

Alternative: a separate merge pass with its own precedence.
It lost because the order rules of `files` would exist twice, and the two would drift.

### 2. The file's name and place in the package

Candidates:

| | Path | For | Against |
|:--|:--|:--|:--|
| A | `.ctl/hakkiri.yaml` | the project's own path; Vale reads a package's `.vale.ini` under the project's name; the schema and the error text are the same | a package holds a `.ctl` folder |
| B | `hakkiri.yaml` | one file at the top of the package | a second name for the same schema |
| C | `settings.yaml` | plain | a second name, and it says nothing of hakkiri |

The specs write A, provisional.
The operator names it.

### 3. Which keys a package may set

`files`, `level` and `vocab`, the keys of the project config except `packages`.
Vale applies a package's file as a project's, with only `StylesPath` dropped, so a package may set anything there.

A package that holds `packages` is an error.
Vale installs the nested packages, but a nested `url` needs a base for relative `path` entries and a place in the order, and no package needs it yet.

Alternative: install nested packages.
It would give a package a fetch of its own and a second cache entry no project listed.
It stays a row of its own if a package asks for it.

### 4. A `styles` list under a glob the package also writes

Candidates:

| | Rule | Result for `'*.md': {styles: [Own]}` after a package's `'*.md': {styles: [PkgA]}` |
|:--|:--|:--|
| A | The project's list replaces the package's, as any later `styles` list does | `Own` only |
| B | Under the same glob text the two lists add up, as Vale merges sections of one name; under another glob the later list replaces | `PkgA` and `Own` |

The probes show Vale doing B (cases "Package alone" and "Another section name").
The specs write B.
A is the smaller rule, and the existing replacement rule would apply as it stands, but a project that adds one style would then drop every package style without a word.
Vale's warning for that case is not copied (see `vale-differences`).

### 5. Scalars: the project wins

`level` is the one scalar.
The project's value wins, and the package's applies when the project sets none, as `MinAlertLevel` does in Vale.
Vale's keys that read the first value, such as `WordTemplate`, let a package beat the project.
hakkiri has no such key, and a future one follows the same rule: the project wins.

### 6. Order across packages

Listed order, the later package winning.
Vale sorts the fragment files as text (`10-` comes before `2-`), so ten or more packages are read in a different order than listed.
hakkiri uses the list.

### 7. Errors name the package file

A fault in a package's file names the package as the project wrote it (its `path` or `url`), the file and the line, so the person who owns the package can fix it.
The key is checked as the project's is, by the same type.

### 8. `vocab`

The package vocabularies and the project's add up, in package order and then the project's.
An empty `vocab` in the project drops what the packages named, as `NONE` does in Vale.
`HKR-019` fixes the shape of the key, so this requirement is built after it.

## Risks

- A package changes what a project lints without a change in the project's own file.
  A project pins the package by hash, so the change shows in the hash.
- A package that sets `files` for `*.md` runs its styles on every project that lists it.
  That is the point, and a project turns a rule off by name.

[v-build]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/file.go#L185-L228
[v-clean]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/cmd/vale/sync.go#L26-L30
[v-copy]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/cmd/vale/sync.go#L166-L180
[v-expand]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/ini.go#L460-L493
[v-formats]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/ini.go#L672
[v-install]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/cmd/vale/sync.go#L121-L185
[v-last]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/ini.go#L227-L234
[v-min]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/ini.go#L409-L428
[v-nested]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/cmd/vale/sync.go#L154-L164
[v-none]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/ini.go#L249-L261
[v-pipe]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/config.go#L431-L451
[v-pipeline]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/source.go#L31-L59
[v-project]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/source.go#L231-L234
[v-shadow]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/ini.go#L495-L516
[v-sort]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/source.go#L146
[v-unknown]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/ini.go#L646-L668
[v-user]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/source.go#L198-L204
[v-warn]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/file.go#L921-L943
[v-wt]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/ini.go#L433-L436
