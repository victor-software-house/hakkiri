## ADDED Requirements

### Requirement: A path is a file or a directory

`hakkiri check` SHALL lint each file it is given and each file under each directory it is given, and SHALL lint the current directory when it is given none.  
A walk SHALL skip `node_modules`, `.git` and each folder a `packages` entry names.

#### Scenario: A directory and a file

- **WHEN** `hakkiri check docs README.md` runs and `docs` holds `a.md` and `sub/b.md`
- **THEN** hakkiri lints `docs/a.md`, `docs/sub/b.md` and `README.md`

#### Scenario: No argument

- **WHEN** `hakkiri check` runs in a project whose `files` matches `*.md`
- **THEN** hakkiri lints every Markdown file under the current directory

#### Scenario: Folders a walk leaves alone

- **WHEN** a walk meets `node_modules/x.md`, `.git/y.md` and a `packages` folder `styles/` holding `z.md`
- **THEN** hakkiri lints none of them, and lints a file of `.github/` that a glob matches

### Requirement: A glob matches the path under the project root

hakkiri SHALL match each `files` glob against the file's absolute path, with symlinks resolved, taken relative to the project root with `/` separators.  
A file outside the root SHALL match no glob, so hakkiri lints none of it and reports no failure for it.

#### Scenario: A dot-slash argument

- **WHEN** `files` holds `'docs/**'` and `hakkiri check ./docs` runs
- **THEN** hakkiri lints `docs/a.md` with the styles of that glob

#### Scenario: An absolute argument

- **WHEN** `files` holds `'docs/**'` and `hakkiri check` is given the absolute path of `docs/a.md`
- **THEN** hakkiri lints it with the styles of that glob

#### Scenario: A parent argument

- **WHEN** `files` holds `'docs/**'` and `HAKKIRI_CONFIG=../.ctl/hakkiri.yaml hakkiri check ../docs/a.md` runs in `other/`
- **THEN** hakkiri lints `docs/a.md` with the styles of that glob

#### Scenario: A file outside the root

- **WHEN** `files` holds `'**'` and `hakkiri check ../outside.md` runs from the root
- **THEN** hakkiri reports no alert and no failure for it

### Requirement: A language is read by extension

hakkiri SHALL read a `.md`, `.mdx`, `.rs`, `.ts`, `.tsx` or `.js` file as Markdown, MDX, Rust, TypeScript, TSX or JavaScript.  
It SHALL skip any other file a walk finds, and SHALL refuse one named as an argument, with an error that names it.

#### Scenario: A file no language reads, found by a walk

- **WHEN** a walk finds `logo.png` and the glob `*` matches it
- **THEN** hakkiri skips it and reports nothing about it

#### Scenario: A file no language reads, named

- **WHEN** `hakkiri check notes.txt` runs
- **THEN** hakkiri reports a failure naming `notes.txt`, and exits 2

### Requirement: A file that fails does not stop the run

hakkiri SHALL report a file it cannot read or parse as a failure, and SHALL still lint every other file.

#### Scenario: One broken MDX file

- **WHEN** `bad.mdx` has a syntax error and `good.md` draws one warning
- **THEN** hakkiri reports the warning and a failure naming `bad.mdx`, and exits 2

### Requirement: The pretty output is a line per alert

hakkiri SHALL print each alert as `path:line:column: level Rule: message`, in order of path, then line, then column, and then one count line.  
Two runs over the same files SHALL print the same text.

#### Scenario: A warning

- **WHEN** `docs/guide.md` draws `ProseStyle.Filler` on line 12, column 5, with the message `Remove 'very'.`
- **THEN** hakkiri prints `docs/guide.md:12:5: warning ProseStyle.Filler: Remove 'very'.` and then `1 alert in 1 file`

#### Scenario: A clean run

- **WHEN** no file draws an alert
- **THEN** hakkiri prints `0 alerts in 3 files` and exits 0

### Requirement: The JSON output is one object

`--format json` SHALL print one object with `alerts`, `failures` and `files`.  
Each alert SHALL hold `path`, `line`, `column`, `end_column`, `level`, `rule`, `message`, `description`, `link` and `match`.  
The output SHALL hold no ESC byte, whatever the colour variables say.

#### Scenario: JSON with colour forced

- **WHEN** `hakkiri check --format json` runs with `CLICOLOR_FORCE=1` and a file draws an alert
- **THEN** the output parses as JSON with one alert, and holds no ESC byte

### Requirement: The exit status says what happened

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
