## ADDED Requirements

### Requirement: formats maps a file to a reader

`formats` SHALL map an extension, a file name or a glob to a reader: `md`, `mdx`, `rs`, `ts`, `tsx`, `js` or `text`.  
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

### Requirement: formats refuses a bad entry

A reader that is not one of the seven SHALL be an error that lists them, and a key that holds a glob mark and is not a glob SHALL be an error.

#### Scenario: An unknown reader

- **WHEN** `formats` holds `toml: loud`
- **THEN** hakkiri reports `` `loud` is not a reader: use md, mdx, rs, ts, tsx, js, text `` and exits 2

#### Scenario: A bad glob

- **WHEN** `formats` holds the key `a[`
- **THEN** hakkiri reports that it is not a glob and exits 2

### Requirement: on_unknown is a config key

`on_unknown` SHALL be a key of the project's config with the values of `--on-unknown`.  
It SHALL be resolved as the flag, then `HAKKIRI_ON_UNKNOWN`, then the config, then `text`, and an unknown word SHALL be an error.

#### Scenario: The order

- **WHEN** the config holds `on_unknown: error`, then `HAKKIRI_ON_UNKNOWN=ignore` is set, then `--on-unknown warn` is given
- **THEN** a selected file with no reader fails the run, then draws nothing, then draws a warning

#### Scenario: An unknown word

- **WHEN** the config holds `on_unknown: loud`
- **THEN** hakkiri reports an error that names `loud` and exits 2
