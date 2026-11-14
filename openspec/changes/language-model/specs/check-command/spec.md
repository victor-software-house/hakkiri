## ADDED Requirements

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

## MODIFIED Requirements

### Requirement: A reader is chosen by formats, then by extension

hakkiri SHALL read a file by the reader its `formats` entry gives it, and otherwise by the reader of its language: Markdown, MDX, Rust, TypeScript, TSX, JavaScript or Text.  
A file that a glob selects and whose language no reader reads, or that has no language, SHALL be read as plain text, unless `on_unknown` says otherwise.  
A Text file SHALL be read as plain text whatever `on_unknown` says.

#### Scenario: A file no reader reads, found by a walk

- **WHEN** a walk finds `notes.txt`, `Cargo.toml` and `Makefile`, and the glob `*` selects them
- **THEN** hakkiri lints each as plain text

#### Scenario: A file no reader reads, named

- **WHEN** `hakkiri check notes.txt` runs and a glob selects `notes.txt`
- **THEN** hakkiri lints it as plain text

#### Scenario: The reader of an extension

- **WHEN** `formats` holds `txt: md` and `a.txt` holds a fenced block that holds `very`
- **THEN** hakkiri reads `a.txt` as Markdown and reports nothing for the fence

#### Scenario: More extensions of a language

- **WHEN** `a.mts` holds `// A very good start.`, `a.markdown` holds `A very good start.`, and a rule flags `very`
- **THEN** hakkiri reads `a.mts` as TypeScript and `a.markdown` as Markdown, and reports `very` once in each

#### Scenario: An interpreter picks the reader

- **WHEN** `bin/tool` has no extension, its first line is `#!/usr/bin/env node`, its second is `// A very good start.`, and a rule flags `very`
- **THEN** hakkiri reads it as JavaScript and reports `very` on line 2

#### Scenario: A Text file under on_unknown warn

- **WHEN** `--on-unknown warn` is given and `*` selects `notes.txt` and `x.cfg`
- **THEN** hakkiri lints `notes.txt` as plain text and warns only for `x.cfg`

## REMOVED Requirements

### Requirement: A file has its own extension and its reader's

**Reason**: A file has languages, not extensions.
**Migration**: Read "A file has its own language, and its reader's when formats chose it".
