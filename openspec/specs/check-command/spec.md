# check-command Specification

## Purpose

`hakkiri check` lints the files and directories it is given, or the current directory, and prints one line per alert or a JSON object.  
It exits 0 for a clean run, 1 when an alert reaches `--fail-level`, and 2 when anything could not be run.

## Requirements

### Requirement: A path is a file or a directory

`hakkiri check` SHALL lint each file it is given and each file under each directory it is given, and SHALL lint the current directory when it is given none.  
A walk SHALL skip `node_modules`, `.git` and each folder a `packages` entry names, comparing the resolved path of each folder with the resolved package paths.  
It SHALL lint a file once, however many of the paths it is given reach that file.

#### Scenario: A directory and a file

- **WHEN** `hakkiri check docs README.md` runs and `docs` holds `a.md` and `sub/b.md`
- **THEN** hakkiri lints `docs/a.md`, `docs/sub/b.md` and `README.md`

#### Scenario: A file named two ways

- **WHEN** `hakkiri check ../a.md .. ../../docs/a.md` runs in `docs/sub/` and `docs/a.md` draws one warning
- **THEN** hakkiri prints that warning once, and counts one file

#### Scenario: No argument

- **WHEN** `hakkiri check` runs in a project whose `files` matches `*.md`
- **THEN** hakkiri lints every Markdown file under the current directory

#### Scenario: Folders a walk leaves alone

- **WHEN** a walk meets `node_modules/x.md`, `.git/y.md` and a `packages` folder `styles/` holding `z.md`
- **THEN** hakkiri lints none of them, and lints a file of `.github/` that a glob matches

#### Scenario: A walk from another folder

- **WHEN** `HAKKIRI_CONFIG=../.ctl/hakkiri.yaml hakkiri check ..` runs in `other/` and the package folder `styles/` holds `Team/notes.md`
- **THEN** hakkiri lints `docs/a.md` and not `styles/Team/notes.md`

### Requirement: A file that fails does not stop the run

hakkiri SHALL report a file it cannot read or parse as a failure, and SHALL still lint every other file.

#### Scenario: One broken MDX file

- **WHEN** `bad.mdx` has a syntax error and `good.md` draws one warning
- **THEN** hakkiri reports the warning and a failure naming `bad.mdx`, and exits 2

### Requirement: The pretty output is a line per alert

hakkiri SHALL print each alert as `path:line:column: level Rule: message`, in order of path, then line, then column, then each failure as `failed path: message`, and then one count line.  
The lines SHALL hold no colour and no frame, and two runs over the same files SHALL print the same text.

#### Scenario: A warning

- **WHEN** `docs/guide.md` draws `ProseStyle.Filler` on line 12, column 5, with the message `Remove 'very'.`
- **THEN** hakkiri prints `docs/guide.md:12:5: warning ProseStyle.Filler: Remove 'very'.` and then `1 alert in 1 file`

#### Scenario: A clean run

- **WHEN** no file draws an alert
- **THEN** hakkiri prints `0 alerts in 3 files` and exits 0

### Requirement: The JSON output is one object

`--format json` SHALL print one object with `alerts`, `failures` and `files`.  
Each alert SHALL hold `path`, `line`, `column`, `end_line`, `end_column`, `level`, `rule`, `message`, `description`, `link` and `match`.  
The output SHALL hold no ESC byte, whatever the colour variables hold.

#### Scenario: JSON with colour forced

- **WHEN** `hakkiri check --format json` runs with `CLICOLOR_FORCE=1` and a file draws an alert
- **THEN** the output parses as JSON with one alert, and holds no ESC byte

#### Scenario: The end of a match

- **WHEN** a file draws an alert for a match on line 1, columns 9 to 12
- **THEN** the alert holds `line` 1, `column` 9, `end_line` 1 and `end_column` 12

### Requirement: The exit status shows what happened

hakkiri SHALL exit 2 when a config or rule error stopped the run or a file failed, and otherwise 1 when a reported alert reaches `--fail-level`, and otherwise 0.  
`--fail-level` SHALL be `suggestion`, `warning` or `error`, and SHALL be `error` when unset.  
It SHALL see only the alerts that the config's `level` leaves in.

#### Scenario: A warning under the default

- **WHEN** a file draws one warning and `--fail-level` is unset
- **THEN** hakkiri exits 0

#### Scenario: A warning under a lower level

- **WHEN** a file draws one warning and `--fail-level warning` is given
- **THEN** hakkiri exits 1

#### Scenario: An alert hidden by the config

- **WHEN** `level` is `error`, a file draws one warning, and `--fail-level warning` is given
- **THEN** hakkiri prints no alert and exits 0

#### Scenario: A failure before an alert

- **WHEN** one file fails and another draws an error
- **THEN** hakkiri prints the alert and the failure, and exits 2

#### Scenario: A broken config

- **WHEN** `.ctl/hakkiri.yaml` holds an unknown key
- **THEN** hakkiri lints no file, prints the error with its line to stderr, and exits 2

### Requirement: Every flag has a variable

Each long flag of `check` SHALL read `HAKKIRI_<FLAG>` when it is not given, with the flag winning over the variable and an empty variable counting as unset.

#### Scenario: A level from the environment

- **WHEN** `HAKKIRI_FAIL_LEVEL=warning` is set, no flag is given, and a file draws one warning
- **THEN** hakkiri exits 1

#### Scenario: The flag wins

- **WHEN** `HAKKIRI_FAIL_LEVEL=warning` is set and `--fail-level error` is given
- **THEN** hakkiri exits 0

### Requirement: A walk skips what git ignores

Inside a git repository, a walk SHALL skip each file and folder that `.gitignore`, `.git/info/exclude` and the global git excludes ignore, unless `gitignore` is false.  
It SHALL apply the `.gitignore` of each folder from the repository root down.  
A file named on the command line SHALL be linted whether git ignores it or not.

#### Scenario: An ignored folder and file

- **WHEN** `.gitignore` holds `build/` and `draft.md`, and `a.md`, `draft.md` and `build/b.md` each draw one warning
- **THEN** `hakkiri check` lints `a.md` alone, and prints `1 alert in 1 file`

#### Scenario: The exclude file and the global excludes

- **WHEN** the same two names are in `.git/info/exclude`, or in the global excludes file
- **THEN** `hakkiri check` lints `a.md` alone

#### Scenario: A nested gitignore

- **WHEN** `docs/.gitignore` holds `skip.md`, and `docs/skip.md` and `skip.md` exist
- **THEN** a walk skips `docs/skip.md` and lints `skip.md`

#### Scenario: A named ignored file

- **WHEN** `hakkiri check draft.md build/b.md` runs and git ignores both
- **THEN** hakkiri lints both

#### Scenario: Git rules off

- **WHEN** `--no-gitignore`, `--gitignore=false`, `HAKKIRI_GITIGNORE=false` or `gitignore: false` is set
- **THEN** a walk lints `a.md`, `build/b.md` and `draft.md`

### Requirement: A named file that no glob selects draws a warning

A file named on the command line that no glob of `files` matches SHALL draw a warning that names it, or a failure with exit 2 when `on_unmatched` is `error`, or nothing when it is `ignore`.  
A file that a glob matches whose styles hold no rule, a file found by a walk, and a file named twice SHALL NOT draw more than the one warning.

#### Scenario: A warning

- **WHEN** the only glob is `docs/*.md` and `hakkiri check README.md docs` runs
- **THEN** hakkiri prints `warning README.md: no glob selects it, so it is not linted` on standard error, lints `docs/a.md`, and exits 0

#### Scenario: An error

- **WHEN** `hakkiri check --on-unmatched error README.md` runs under the same config
- **THEN** hakkiri prints `failed README.md: no glob selects it, so it is not linted` and exits 2

#### Scenario: Ignored

- **WHEN** `--on-unmatched ignore` is given
- **THEN** hakkiri prints nothing for `README.md` and exits 0

#### Scenario: No rule to run

- **WHEN** a glob matches `CHANGELOG.md` with `styles: []` and `hakkiri check CHANGELOG.md` runs
- **THEN** hakkiri prints no warning

#### Scenario: A walk finds an unselected file

- **WHEN** `hakkiri check` walks a folder that holds `README.md`, which no glob selects
- **THEN** hakkiri prints no warning

### Requirement: A reader is chosen by formats, then by extension

hakkiri SHALL read a file by the reader its `formats` entry gives it, and otherwise by the reader of its language: Markdown, MDX, Rust, TypeScript, TSX, JavaScript, Go, Swift, YAML, TOML, Shell or Text.  
A reader SHALL follow the langdex language of the file, so no table of extensions lists the files a reader takes.

#### Scenario: The reader of an extension

- **WHEN** `formats` holds `txt: md` and `a.txt` holds a fenced block that holds `very`
- **THEN** hakkiri reads `a.txt` as Markdown and reports nothing for the fence

#### Scenario: More extensions of a language

- **WHEN** `a.mts` holds `// A very good start.`, `a.markdown` holds `A very good start.`, and a rule flags `very`
- **THEN** hakkiri reads `a.mts` as TypeScript and `a.markdown` as Markdown, and reports `very` once in each

#### Scenario: An interpreter picks the reader

- **WHEN** `bin/tool` has no extension, its first line is `#!/usr/bin/env node`, its second is `// A very good start.`, and a rule flags `very`
- **THEN** hakkiri reads it as JavaScript and reports `very` on line 2

#### Scenario: A TOML file

- **WHEN** a glob selects `Cargo.toml`, which holds `name = "very" # very`
- **THEN** hakkiri reports one alert, in the comment, and none for the string

#### Scenario: Files of the data and shell languages

- **WHEN** `a.yml` holds `# A very good start.`, `a.bash` holds `# A very good start.`, `bashrc` holds `# A very good start.`, `bin/tool` starts with `#!/bin/sh` and holds `# A very good start.`, and a rule flags `very`
- **THEN** hakkiri reads the first as YAML and the other three as Shell, and reports `very` once in each

### Requirement: A file no reader reads is plain text

A file that a glob selects and whose language no reader reads, or that has no language, SHALL be read as plain text, unless `on_unknown` says otherwise.  
A Text file SHALL be read as plain text whatever `on_unknown` says.

#### Scenario: A file no reader reads, found by a walk

- **WHEN** a walk finds `notes.txt`, `Dockerfile` and `Makefile`, and the glob `*` selects them
- **THEN** hakkiri lints each as plain text

#### Scenario: A file no reader reads, named

- **WHEN** `hakkiri check notes.txt` runs and a glob selects `notes.txt`
- **THEN** hakkiri lints it as plain text

#### Scenario: A Text file under on_unknown warn

- **WHEN** `--on-unknown warn` is given and `*` selects `notes.txt` and `x.cfg`
- **THEN** hakkiri lints `notes.txt` as plain text and warns only for `x.cfg`

### Requirement: Plain text is read as written

A plain-text file SHALL have one `raw` scope over the whole file, and one `paragraph` for each run of lines between blank lines.  
No marker, escape or entity SHALL change its text.

#### Scenario: Paragraphs

- **WHEN** `notes.txt` holds `This is very good.`, a blank line and `A second very paragraph.` and a rule flags `very`
- **THEN** hakkiri reports `notes.txt:1:9` and `notes.txt:3:10`

#### Scenario: A fence is text

- **WHEN** `a.txt` holds a fence around `very`
- **THEN** hakkiri reports `very` on line 2, column 1

### Requirement: A binary file is skipped

A file read as plain text that holds a NUL byte in its first 8192 bytes SHALL be skipped with no alert and no message.
This SHALL hold whether a walk finds the file or the command line names it.
A byte that is not valid UTF-8 SHALL read as one `?` and SHALL NOT make the file binary.

#### Scenario: A picture

- **WHEN** `logo.png` holds a NUL byte and the glob `*` selects it
- **THEN** hakkiri skips it, counts no file for it, and exits 0

#### Scenario: A Latin-1 file

- **WHEN** `latin.txt` holds a word that has the byte `E9`, then a space and `very`, and no NUL byte, and a rule flags `very`
- **THEN** hakkiri reports `very` at the column that counts the bytes, and Vale gives the same column

### Requirement: on_unknown sets what a selected file with no reader gets

For a file that a glob selects and that no reader reads, `on_unknown` SHALL be one of four words.
`text`, the default, reads it as plain text, and `warn` skips it with a warning that names it.
`error` fails the run with exit 2, and `ignore` skips it.  
A file that no glob selects SHALL draw no message.

#### Scenario: Warn

- **WHEN** `--on-unknown warn` is given and `*` selects `x.cfg`
- **THEN** hakkiri prints ``warning x.cfg: no reader reads this file, so it is not linted: map it in `formats` `` and does not lint it

#### Scenario: Error

- **WHEN** `--on-unknown error` is given
- **THEN** hakkiri prints ``failed x.cfg: no reader reads this file, so it is not linted: map it in `formats` `` and exits 2

#### Scenario: Ignore

- **WHEN** `--on-unknown ignore` is given
- **THEN** hakkiri skips `x.cfg` and prints nothing

#### Scenario: A file no glob selects

- **WHEN** `*.md` is the only glob and `--on-unknown warn` is given
- **THEN** hakkiri prints no warning for `x.cfg`

### Requirement: A file has a language

A file SHALL have the langdex language of its path, read by its file name and then its extension.  
A file whose path gives none SHALL have the language of the interpreter on its first line, read from at most its first 4096 bytes, and otherwise no language.

#### Scenario: A file name, an extension and an interpreter

- **WHEN** a walk finds `LICENSE`, `notes.txt`, `Cargo.toml`, `README`, and `bin/tool` whose first line is `#!/usr/bin/env bash`
- **THEN** `LICENSE` and `notes.txt` are Text, `Cargo.toml` is TOML, `README` has no language, and `bin/tool` is Shell

### Requirement: A file has its own language, and its reader's when formats chose it

For the scopes of the rules that run on it, a node outside a fenced code block SHALL be in the file's own language.  
It SHALL also be in the language of the file's reader when a `formats` entry chose that reader.  
A scope part that names a language SHALL hold for either, and a negated part SHALL leave a block out for either.

#### Scenario: A mapped file

- **WHEN** `formats` maps `txt: md`, `a.txt` holds `# beta`, a blank line and `beta alpha gamma.`, and three rules have `text.md` with `alpha`, `text & ~heading.md` with `beta`, and `text.txt` with `gamma`
- **THEN** hakkiri reports `beta`, `alpha` and `gamma` on line 3, and none in the heading

#### Scenario: A file read as plain text by on_unknown

- **WHEN** a rule has `scope: text.txt` and `tokens: [marker]`, the glob `*` selects `notes.txt`, `guide.rst` and `README`, and each holds `A marker.`
- **THEN** hakkiri reports one alert, in `notes.txt`
