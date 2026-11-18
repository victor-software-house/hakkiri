## MODIFIED Requirements

### Requirement: formats maps a file to a reader

`formats` SHALL map an extension, a file name or a glob to a reader.
The value SHALL be any langdex name, alias or extension of the language the reader reads.
The readers are Markdown, MDX, Rust, TypeScript, TSX, JavaScript, Go, Swift, YAML, TOML, Shell and Text.

So `md`, `markdown` and `Markdown` select one reader, `sh` and `bash` select the Shell reader, and `text` selects plain text.

An entry for the file's extension SHALL come first.
Next SHALL come an entry whose key is the file's name, or a glob that matches its name or its path.
These SHALL come in the order of the keys as text.

A key that contains `*`, `?`, `[`, `{` or `/` SHALL be a glob.
An entry SHALL beat the language of the file.

#### Scenario: An extension

- **WHEN** `formats` contains `txt: md`
- **THEN** `a.txt` is read as Markdown

#### Scenario: Another name of the reader

- **WHEN** `formats` contains `txt: markdown`
- **THEN** `a.txt` is read as Markdown

#### Scenario: Over the table

- **WHEN** `formats` contains `md: text`
- **THEN** `a.md` is read as plain text

#### Scenario: A name and a glob

- **WHEN** `formats` contains `NOTES: md` and `conf/*.cfg: md`
- **THEN** `NOTES` and `conf/x.cfg` are read as Markdown, and `other.cfg` as plain text

#### Scenario: The extension first

- **WHEN** `formats` contains `txt: text` and `'*': md`
- **THEN** `a.txt` is read as plain text and `b.cfg` as Markdown

#### Scenario: A script with no extension

- **WHEN** `formats` contains `mise-tasks/*: sh`
- **THEN** a file `mise-tasks/lint` is read as Shell

### Requirement: formats rejects a bad entry

A reader value that matches no language hakkiri reads SHALL be an error that lists the twelve.
A key that contains a glob mark and is not a glob SHALL be an error.

#### Scenario: An unknown reader

- **WHEN** `formats` contains `toml: loud`
- **THEN** hakkiri reports `` `loud` is not a language hakkiri reads: use Markdown, MDX, Rust, TypeScript, TSX, JavaScript, Go, Swift, YAML, TOML, Shell or Text `` and exits 2

#### Scenario: A language with no reader

- **WHEN** `formats` contains `cfg: python`
- **THEN** hakkiri reports `` `python` is not a language hakkiri reads: use Markdown, MDX, Rust, TypeScript, TSX, JavaScript, Go, Swift, YAML, TOML, Shell or Text `` and exits 2

#### Scenario: A bad glob

- **WHEN** `formats` contains the key `a[`
- **THEN** hakkiri reports that it is not a glob and exits 2
