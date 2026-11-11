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

- A project names vocabularies, and hakkiri reads them from its packages.
- The accepted terms reach the checks that Vale reaches, and hold back the matches Vale holds back.

Non-goals:

- A vocabulary named per glob, and the YAML vocabulary format (design decision 7).
- `Vale.Terms` and `Vale.Avoid`, which are `HKR-042`.
- `Vale.Spelling` and `Vale.Repetition`, which are `HKR-026` and `HKR-038`.
- Reading a package's `.vale.ini`, which is `HKR-035`.

## Decisions

### 1. Vale's code and a probe are the reference

Vale at `66bbb85` reads vocabularies in these places.
The probe ran a Vale build of that commit on 2026-11-11 in nine small projects, and the cases below quote what it reported.

| Part | Where in Vale | What it does |
|:--|:--|:--|
| Top-level `Vocab` | [ini.go][v-top-vocab] | Reads each named vocabulary, and adds its terms to the project's accepted and rejected lists |
| Finding one | [ini.go][v-load-vocab], [config.go][v-search-paths] | Takes the first search path that holds `config/vocabularies/<Name>`, walks it, and reads each `accept.txt` and `reject.txt` |
| A line | [ini.go][v-word-list] | Trims it, skips a blank line and one that starts with a hash and a space |
| A term as a pattern | [definition.go][v-term-pattern] | Escapes the periods of a plain term, and makes each run of whitespace match any run |
| Exceptions | [definition.go][v-update-exceptions], [definition.go][v-phrase] | Sorts the terms longest first, gives each `(?-i)`, and holds back a match inside an accepted phrase |
| The checks | [capitalization][v-capitalization], [conditional][v-conditional], [existence][v-existence], [repetition][v-repetition], [substitution][v-substitution], [spelling][v-spelling] | Each adds the accepted terms to its exceptions |

The probe found these, which the spec states as scenarios:

- A line that starts with `#` and no space is a term: `#nocomment` is accepted as the text `#nocomment`.
- A term of `C++` stops the run: Vale reports `error parsing regexp: invalid nested repetition operator` with no file and no line.
- A `words.txt` is ignored, and an `accept.txt` in a subfolder is read.

Not probed yet, and task 1.1 probes them before the code: the `conditional`, `repetition` and `substitution` exceptions, the `vocab: false` flag on `existence`, a plain term with a period as an exception, and a phrase that wraps across a line.

### 2. The config key that names vocabularies is open

The operator chooses where the key sits and what it is called.
The snippets use the working name `vocab`, Vale's word for the key.
The other name in play is `vocabularies`, the folder's name.

| | A. Top level | B. Per glob | C. Both |
|:--|:--|:--|:--|
| Config | `vocab: [Team]` beside `packages` | `vocab: [Team]` beside `styles` in a glob | A, and B for more |
| Vale's form | `Vocab = Team` before any section | `Vocab = Team` under `[*.md]` | both |
| Accepted terms reach | every rule that reads accepted terms, in every file, `capitalization` and `conditional` included | `existence`, `repetition`, `spelling` and `substitution` in the files of the glob, and not `capitalization` or `conditional` | the union |
| Rules built | `Vale.Terms` and `Vale.Avoid` | `Vale.Team.Terms` and `Vale.Team.Avoid`, on for the glob's files | both sets |
| How they are held | once, when each rule builds | per file, from the file's globs | both |

Snippet A, the top level:

```yaml
packages:
  - path: .vale/styles
vocab: [Team]
files:
  '*.md': {styles: [Team, Vale]}
```

Snippet B, per glob:

```yaml
packages:
  - path: .vale/styles
files:
  '*.md': {styles: [Team], vocab: [Team]}
  'docs/**/*.md': {vocab: [Team, Docs]}
```

Snippet C, both:

```yaml
packages:
  - path: .vale/styles
vocab: [Team]
files:
  '*.md': {styles: [Team, Vale]}
  'docs/**/*.md': {vocab: [Docs]}
```

Recommendation: A.
A package sets the top-level `Vocab` in Vale, as [prose-style does][prose-style-ini], and the acceptance names the effect that only the top level has: accepted terms reach `capitalization`.
A builds each rule's exceptions once, where B needs exceptions held per file at run time, which is Vale's `vocabFor` ([definition.go][v-vocab-for]) and its own row, `HKR-040`.
C adds a key to A later with no change to A's behaviour, so A closes nothing.
Alternative: B alone.
It lost because it leaves `capitalization` and the generated `Vale.Terms` name out of reach for the package that ProseStyle ships.

### 3. A vocabulary is a folder in a loaded package

A package folder is the `styles` folder of its zip, or the `path` folder, so a vocabulary is `<package folder>/config/vocabularies/<name>/`.
hakkiri walks it and reads every `accept.txt` and `reject.txt` below it, as Vale does, and no other file.
It reads the accepted terms in the order the files walk and the vocabularies are named.

An unknown name is an error on the key's line: `` `Nope` is not a vocabulary in any package ``.
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
Alternative: escape every term unless it is marked as a pattern.
It lost because Vale's vocabularies hold patterns that way (`harness(?:es)?` in ProseStyle), so an escaped term would stop matching.

### 5. Accepted terms are exceptions

When a rule builds, hakkiri adds the accepted terms to its exceptions before it rewrites and sorts them, as [updateExceptions][v-update-exceptions] does.
The case-and-sequence-checks change records the order: the terms go in before `exceptions` is rewritten.

| Check | Accepted terms reach it | Vale |
|:--|:--|:--|
| `capitalization` | the exceptions pattern and the words the converters keep, unless `vocab: false` | [capitalization.go][v-capitalization] |
| `conditional`, `repetition`, `substitution` | the exceptions and the phrases, unless `vocab: false` | [conditional.go][v-conditional], [repetition.go][v-repetition], [substitution.go][v-substitution] |
| `existence` | the same, and not when the rule is `nonword` | [existence.go][v-existence] |
| `spelling` | always, as whole words matched without regard to case, with a phrase masked | [spelling.go][v-spelling], [spelling.go][v-spelling-run] |

No other check reads them.
The `vocab` field loads today with no effect, and it now decides.
A match inside an accepted phrase is held back, so `mea` in `mea culpa` is held back and a lone `mea` is not.

### 6. Vocabularies are read before rules build

A rule builds its exceptions when it loads, so the terms must exist first.
The loader reads all package folders, then the vocabularies, then the styles with the accepted terms in hand.
Today it reads each package's styles as it visits the package.
Alternative: patch each rule after it loads.
It lost because the exceptions are compiled patterns sorted and rewritten at build.

### 7. What stays different

- Per-glob vocabularies and the rules named `Vale.<Name>.Terms`: `HKR-040`.
- The YAML vocabulary, `vocab.yml`, with `regex`, `ignorecase`, `flags` and an affix file ([vocab.go][v-yaml-vocab]): `HKR-041`.
- `Vale.Terms` and `Vale.Avoid`: `HKR-042`, in the [vocabulary-rules change][rules].
- A vocabulary named in a package's own `.vale.ini`: `HKR-035`.
- The accepted terms as spelling suggestions ([spelling.go][v-spelling-suggest]), which needs `Vale.Spelling` and an applied action.
- The per-user styles folder, two packages with one vocabulary name and an invalid term: decisions 3 and 4.

[v-top-vocab]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/ini.go#L449-L457
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
[rules]: ../vocabulary-rules/proposal.md
