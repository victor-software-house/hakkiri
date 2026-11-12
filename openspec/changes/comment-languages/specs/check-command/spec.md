## MODIFIED Requirements

### Requirement: A reader is chosen by formats, then by extension

hakkiri SHALL read a file by the reader its `formats` entry gives it, and otherwise by its extension: `.md`, `.mdx`, `.rs`, `.ts`, `.tsx`, `.js`, `.go`, `.swift`, `.yaml` or `.yml`, `.toml`, and `.sh` or `.bash` as Markdown, MDX, Rust, TypeScript, TSX, JavaScript, Go, Swift, YAML, TOML or shell.  
A file that a glob selects and that no reader reads SHALL be read as plain text, unless `on_unknown` says otherwise.

#### Scenario: A file no reader reads, found by a walk

- **WHEN** a walk finds `notes.txt`, `Dockerfile` and `Makefile`, and the glob `*` selects them
- **THEN** hakkiri lints each as plain text

#### Scenario: A file no reader reads, named

- **WHEN** `hakkiri check notes.txt` runs and a glob selects `notes.txt`
- **THEN** hakkiri lints it as plain text

#### Scenario: The reader of an extension

- **WHEN** `formats` holds `txt: md` and `a.txt` holds a fenced block that holds `very`
- **THEN** hakkiri reads `a.txt` as Markdown and reports nothing for the fence

#### Scenario: A TOML file

- **WHEN** a glob selects `Cargo.toml`, which holds `name = "very" # very`
- **THEN** hakkiri reports one alert, in the comment, and none for the string
