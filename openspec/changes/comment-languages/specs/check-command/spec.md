## MODIFIED Requirements

### Requirement: A reader is chosen by formats, then by extension

hakkiri SHALL read a file by the reader its `formats` entry gives it, and otherwise by the reader of its language: Markdown, MDX, Rust, TypeScript, TSX, JavaScript, Go, Swift, YAML, TOML, Shell or Text.  
A reader SHALL follow the langdex language of the file, so no table of extensions lists the files a reader takes.  
A file that a glob selects and whose language no reader reads, or that has no language, SHALL be read as plain text, unless `on_unknown` says otherwise.  
A Text file SHALL be read as plain text whatever `on_unknown` says.

#### Scenario: A file no reader reads, found by a walk

- **WHEN** a walk finds `notes.txt`, `Dockerfile` and `Makefile`, and the glob `*` selects them
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

#### Scenario: A TOML file

- **WHEN** a glob selects `Cargo.toml`, which holds `name = "very" # very`
- **THEN** hakkiri reports one alert, in the comment, and none for the string

#### Scenario: Files of the data and shell languages

- **WHEN** `a.yml` holds `# A very good start.`, `a.bash` holds `# A very good start.`, `bashrc` holds `# A very good start.`, `bin/tool` starts with `#!/bin/sh` and holds `# A very good start.`, and a rule flags `very`
- **THEN** hakkiri reads the first as YAML and the other three as Shell, and reports `very` once in each
