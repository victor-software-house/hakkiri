## MODIFIED Requirements

### Requirement: formats maps a file to a reader

`formats` SHALL map an extension, a file name or a glob to a reader: `md`, `mdx`, `rs`, `ts`, `tsx`, `js`, `go`, `swift`, `yaml`, `toml`, `sh` or `text`.  
An entry named by the file's extension SHALL come first, then, in the order of the keys as text, an entry whose key is the file's name or a glob that matches its name or its path.  
A key that holds `*`, `?`, `[`, `{` or `/` SHALL be a glob.  
An entry SHALL beat the extension table.

#### Scenario: An extension

- **WHEN** `formats` holds `txt: md`
- **THEN** `a.txt` is read as Markdown

#### Scenario: Over the table

- **WHEN** `formats` holds `md: text`
- **THEN** `a.md` is read as plain text

#### Scenario: A name and a glob

- **WHEN** `formats` holds `NOTES: md` and `conf/*.cfg: md`
- **THEN** `NOTES` and `conf/x.cfg` are read as Markdown, and `other.cfg` as plain text

#### Scenario: The extension first

- **WHEN** `formats` holds `txt: text` and `'*': md`
- **THEN** `a.txt` is read as plain text and `b.cfg` as Markdown

#### Scenario: A script with no extension

- **WHEN** `formats` holds `mise-tasks/*: sh`
- **THEN** a file `mise-tasks/lint` is read as shell

### Requirement: formats refuses a bad entry

A reader that is not one of the twelve SHALL be an error that lists them, and a key that holds a glob mark and is not a glob SHALL be an error.

#### Scenario: An unknown reader

- **WHEN** `formats` holds `toml: loud`
- **THEN** hakkiri reports `` `loud` is not a reader: use md, mdx, rs, ts, tsx, js, go, swift, yaml, toml, sh, text `` and exits 2

#### Scenario: A bad glob

- **WHEN** `formats` holds the key `a[`
- **THEN** hakkiri reports that it is not a glob and exits 2
