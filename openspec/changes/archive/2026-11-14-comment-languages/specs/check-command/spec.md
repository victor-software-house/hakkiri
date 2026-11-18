## MODIFIED Requirements

### Requirement: A reader is chosen by formats, then by extension

hakkiri SHALL read a file by the reader its `formats` entry gives it.  
Otherwise it SHALL use the reader of the file's language: Markdown, MDX, Rust, TypeScript, TSX, JavaScript, Go, Swift, YAML, TOML, Shell or Text.

The reader SHALL follow the langdex language of the file, so no table of extensions lists the files a reader takes.

A file that a glob selects SHALL be read as plain text when no reader reads its language, or it has no language.  
`on_unknown` can set this otherwise, but a Text file SHALL be read as plain text whatever `on_unknown` is set to.

#### Scenario: A file no reader reads, found by a walk

- **WHEN** a walk finds `notes.txt`, `Dockerfile` and `Makefile`, and the glob `*` selects them
- **THEN** hakkiri lints each as plain text

#### Scenario: A file no reader reads, named

- **WHEN** `hakkiri check notes.txt` runs and a glob selects `notes.txt`
- **THEN** hakkiri lints it as plain text

#### Scenario: The reader of an extension

- **WHEN** `formats` contains `txt: md` and `a.txt` contains a fenced block that contains `very`
- **THEN** hakkiri reads `a.txt` as Markdown and reports nothing for the fence

#### Scenario: More extensions of a language

- **WHEN** `a.mts` contains `// A very good start.`, `a.markdown` contains `A very good start.`, and a rule flags `very`
- **THEN** hakkiri reads `a.mts` as TypeScript and `a.markdown` as Markdown, and reports `very` once in each

#### Scenario: An interpreter picks the reader

- **WHEN** `bin/tool` has no extension, its first line is `#!/usr/bin/env node`, its second is `// A very good start.`, and a rule flags `very`
- **THEN** hakkiri reads it as JavaScript and reports `very` on line 2

#### Scenario: A Text file under on_unknown warn

- **WHEN** `--on-unknown warn` is given and `*` selects `notes.txt` and `x.cfg`
- **THEN** hakkiri lints `notes.txt` as plain text and warns only for `x.cfg`

#### Scenario: A TOML file

- **WHEN** a glob selects `Cargo.toml`, which contains `name = "very" # very`
- **THEN** hakkiri reports one alert, in the comment, and none for the string

#### Scenario: Files of the data and shell languages

- **WHEN** `a.yml` contains `# A very good start.`, `a.bash` contains `# A very good start.`, `bashrc` contains `# A very good start.`, `bin/tool` starts with `#!/bin/sh` and contains `# A very good start.`, and a rule flags `very`
- **THEN** hakkiri reads the first as YAML and the other three as Shell, and reports `very` once in each
