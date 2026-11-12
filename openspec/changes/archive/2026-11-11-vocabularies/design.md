# Design

## Context

A Vale vocabulary is a folder `config/vocabularies/<Name>/` that holds `accept.txt` and `reject.txt`, one term per line.
Vale reads the accepted terms as exceptions in most checks, and builds `Vale.Terms` and `Vale.Avoid` from the lists.
The [vocabulary-rules change][rules] builds those two rules, so this change reads the lists and applies the exceptions.
`HKR-006` fetches a package whole and keeps its `styles/config` folder, and the loader skips that folder as a style.
`HKR-007` loads Vale styles as they are.
So a vocabulary is already on disk and nothing reads it.

## Goals / Non-Goals

Goals:

- A project names vocabularies at the top level and per glob, and hakkiri reads them from its packages.
- The accepted terms reach the checks that Vale reaches, and hold back the matches Vale holds back.

Non-goals:

- The YAML vocabulary format (design decision 8).
- `Vale.Terms` and `Vale.Avoid`, and the rules named `Vale.<Name>.Terms` for a glob, which are `HKR-042`.
- `Vale.Spelling` and `Vale.Repetition`, which are `HKR-026` and `HKR-038`.
- Reading a package's `.vale.ini`, which is `HKR-035`.

## Decisions

### 1. Vale's code and a probe are the reference

Vale at `66bbb85` reads vocabularies in these places.
The probe ran a Vale build of that commit on 2026-11-11, in two rounds: nine small projects, then five configs over one package in `tests/fixtures/vocabularies`, whose `vale.tsv` holds every alert.

| Part | Where in Vale | What it does |
|:--|:--|:--|
| Top-level `Vocab` | [ini.go][v-top-vocab] | Reads each named vocabulary, and adds its terms to the project's accepted and rejected lists |
| A section's `Vocab` | [ini.go][v-section-vocab], [file.go][v-file-vocab], [definition.go][v-vocab-for] | Holds the terms per file, from each section that matches, and checks them as each rule runs |
| Finding one | [ini.go][v-load-vocab], [config.go][v-search-paths] | Takes the first search path that holds `config/vocabularies/<Name>`, walks it, and reads each `accept.txt` and `reject.txt` |
| A line | [ini.go][v-word-list] | Trims it, skips a blank line and one that starts with a hash and a space |
| A term as a pattern | [definition.go][v-term-pattern] | Escapes the periods of a plain term, and makes each run of whitespace match any run |
| Exceptions | [definition.go][v-update-exceptions], [definition.go][v-phrase] | Sorts the terms longest first, gives each `(?-i)`, and holds back a match inside an accepted phrase |
| The checks | [capitalization][v-capitalization], [conditional][v-conditional], [existence][v-existence], [repetition][v-repetition], [substitution][v-substitution], [spelling][v-spelling] | Each adds the accepted terms to its exceptions |

The probe found these, which the spec states as scenarios:

- A line that starts with `#` and no space is a term: `#nocomment` is accepted as the text `#nocomment`.
- A term of `C++` stops the run: Vale reports `error parsing regexp: invalid nested repetition operator` with no file and no line.
- A `words.txt` is ignored, and an `accept.txt` in a subfolder is read.
- A plain term with a period holds back its own text only: `Node.js` leaves `Nodexjs` flagged.
- A phrase that wraps across a line is held back whole, in `existence` and in `spelling`.
- At the top level, `existence`, `repetition`, `substitution`, `conditional`, `capitalization` and `spelling` all take the accepted terms, and `vocab: false` turns it off in the first five and `nonword` in `existence`.
- A `capitalization` rule keeps its own `exceptions` and the accepted terms together.
- A section's list reaches `existence`, `repetition`, `substitution` and `spelling`, and not `capitalization` or `conditional`.
- A section's list ignores `vocab: false` and `nonword`: `Filler0` and `Raw` hold back `GitHub` there and not at the top.
- A file takes the lists of every section that matches it, and the top-level lists besides.
- A section's list is read case-sensitively in `spelling`: with `zorbl` accepted, the top level holds back `Zorbl` and a section does not.
- A section's `Vocab` also turns on `Vale.<Name>.Terms` for its files, which is `HKR-042`.

### 2. The key is `vocab`, at the top level and per glob

The key is `vocab`, a list of vocabulary names, and it sits at the top level and in a glob of `files` (operator, 2026-11-11).
The top level is Vale's `Vocab` before any section, and a glob's key is Vale's `Vocab` in a section.
A glob's list adds its vocabularies to the files that glob matches.

```yaml
packages:
  - path: .vale/styles
vocab: [Team]
files:
  '*.md': {styles: [Team]}
  'docs/**/*.md': {vocab: [Docs]}
```

| | Top level | Per glob |
|:--|:--|:--|
| Vale form | `Vocab = Team` before any section | `Vocab = Team` under `[*.md]` |
| Accepted terms reach | every check that reads them, in every file | `existence`, `repetition`, `spelling` and `substitution`, in the glob's files |
| Held | once, when each rule builds | per file, from the globs that match it |

Alternative: the top level alone.
It lost because a package that ships two vocabularies could not use one for its docs only.
Alternative: a glob alone.
It lost because ProseStyle names one vocabulary for every file ([prose-style][prose-style-ini]), and a glob key would repeat it on each glob.
The other name considered, `vocabularies`, lost to Vale's `vocab`.

### 3. A vocabulary is a folder in a loaded package

A package folder is the `styles` folder of its zip, or the `path` folder, so a vocabulary is `<package folder>/config/vocabularies/<name>/`.
hakkiri walks it and reads every `accept.txt` and `reject.txt` below it, as Vale does, and no other file.
It reads the accepted terms in the order the files walk and the vocabularies are named.

An unknown name is an error on the key's line: `` `Nope` is not a vocabulary in any package ``.
A name of the top level is refused by the config's own validation, so its line comes from the YAML location index and not from a search of the text.
Vale's message names the search paths and has no line ([ini.go][v-load-vocab]).

Two packages that hold one vocabulary name are an error that names both, as for a style.
Alternative: the first package wins, as Vale's search does.
It lost because a second package would then change nothing and say nothing.
Alternative: the last package wins, as `vale sync` does when it moves the folders into one `StylesPath`.
It lost for the same reason, and because it makes the order of `packages` decide silently.

Vale also looks in a per-user styles folder.
hakkiri has none, because a project's packages are all in its config (intended difference).

### 4. A line of a list is a pattern

hakkiri trims a line, skips a blank one and one that starts with `# `, and reads the rest as one term.
A term with no pattern operator other than a period is plain: its periods are literal, so `Node.js` does not match `nodexjs`.
Any other term is a pattern in hakkiri's regex dialect, so `harness(?:es)?` matches two forms.
In both, a run of whitespace matches any run of whitespace, so a phrase that wraps still matches ([definition.go][v-term-pattern]).

A term that is not a valid pattern is an error that names the vocabulary file and the line.
Vale reports the failure with no line of the vocabulary file, and `C++` stops every run (intended difference, so the error is one a person can find).
hakkiri's regex dialect reads `C++` as a pattern with a possessive plus, so it holds back `C` and `CC` and stops nothing; a term is written `C\+\+` to mean the text.
Alternative: escape every term unless it is marked as a pattern.
It lost because Vale's vocabularies hold patterns that way (`harness(?:es)?` in ProseStyle), so an escaped term would stop matching.

### 5. Accepted terms are exceptions

When a rule builds, hakkiri adds the accepted terms to its exceptions before it rewrites and sorts them, as [updateExceptions][v-update-exceptions] does.
The `case-and-sequence-checks` change records the order: the terms go in before `exceptions` is rewritten.

| Check | Accepted terms reach it | Vale |
|:--|:--|:--|
| `capitalization` | the exceptions pattern and the words the converters keep, unless `vocab: false` | [capitalization.go][v-capitalization] |
| `conditional`, `repetition`, `substitution` | the exceptions and the phrases, unless `vocab: false` | [conditional.go][v-conditional], [repetition.go][v-repetition], [substitution.go][v-substitution] |
| `existence` | the same, and not when the rule is `nonword` | [existence.go][v-existence] |
| `spelling` | always, as whole words matched without regard to case, with a phrase masked | [spelling.go][v-spelling], [spelling.go][v-spelling-run] |

No other check reads them.
The `vocab` field loads today with no effect, and it now decides.
A match inside an accepted phrase is held back, so `mea` in `mea culpa` is held back and a lone `mea` is not.

### 6. A glob's lists are held per file

hakkiri collects a file's lists from every glob that matches it, in the order written.
It builds one set of exceptions from them with the builder that a rule's `exceptions` use, and asks it of each match of an `existence`, `repetition`, `spelling` and `substitution` rule, after the rule's own exceptions ([definition.go][v-vocab-for]).
A set is built once for each distinct list of vocabulary names.
The top-level terms stay in each rule's own exceptions.
So a file's glob lists add to the top-level ones, and they ignore `vocab: false` and `nonword`, as the probe shows.

The terms of a glob are read case-sensitively in `spelling`, where the top level reads them without regard to case ([spelling.go][v-spelling-run]).
`capitalization` and `conditional` do not read them.

Alternative: compile each rule again for each distinct list.
It lost because it multiplies the compiled rules by the number of lists, and Vale's own check is a second pass over the matches.

### 7. Vocabularies are read before rules build

A rule builds its exceptions when it loads, so the terms must exist first.
The loader reads all package folders, then the vocabularies, then the styles with the accepted terms in hand.
Today it reads each package's styles as it visits the package.
Alternative: patch each rule after it loads.
It lost because the exceptions are compiled patterns sorted and rewritten at build.

### 8. What stays different

- The rules named `Vale.<Name>.Terms` and `Vale.<Name>.Avoid` for a glob's vocabularies: `HKR-042`, with the top-level pair.
- The YAML vocabulary, `vocab.yml`, with `regex`, `ignorecase`, `flags` and an affix file ([vocab.go][v-yaml-vocab]): `HKR-041`.
- `Vale.Terms` and `Vale.Avoid`, in the [vocabulary-rules change][rules].
- A vocabulary named in a package's own `.vale.ini`: `HKR-035`.
- The accepted terms as spelling suggestions ([spelling.go][v-spelling-suggest]), which needs `Vale.Spelling` and an applied action.
- The per-user styles folder, two packages with one vocabulary name and an invalid term: decisions 3 and 4.

[v-top-vocab]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/ini.go#L449-L457
[v-section-vocab]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/ini.go#L283-L297
[v-file-vocab]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/file.go#L182-L226
[v-load-vocab]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/ini.go#L94-L154
[v-word-list]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/ini.go#L156-L174
[v-search-paths]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/config.go#L345-L351
[v-term-pattern]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/definition.go#L62-L81
[v-update-exceptions]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/definition.go#L473-L507
[v-phrase]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/definition.go#L509-L554
[v-vocab-for]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/definition.go#L28-L60
[v-capitalization]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/capitalization.go#L52-L70
[v-conditional]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/conditional.go#L94-L99
[v-existence]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/existence.go#L52-L63
[v-repetition]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/repetition.go#L41-L46
[v-substitution]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/substitution.go#L71-L76
[v-spelling]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/spelling.go#L89-L110
[v-spelling-run]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/spelling.go#L195-L242
[v-spelling-suggest]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/spelling.go#L391-L430
[v-yaml-vocab]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/vocab.go#L18-L181
[prose-style-ini]: https://github.com/victor-software-house/prose-style/blob/90c7c8e673b92f836cce257427724fc79da997e8/ProseStyle/.vale.ini
[rules]: ../2026-11-11-vocabulary-rules/proposal.md
