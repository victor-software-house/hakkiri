## MODIFIED Requirements

### Requirement: formats maps a file to a reader

`formats` SHALL map an extension, a file name or a glob to a reader.  
The entry SHALL give the reader by any langdex name, alias or extension of the language it reads.

The readers are Markdown, MDX, Rust, TypeScript, TSX (TypeScript with JavaScript XML), JavaScript and Text.  
So `md`, `markdown` and `Markdown` give one reader, and `text` gives plain text.

An entry keyed by the file's extension SHALL come first.  
Next SHALL come an entry keyed by the file's name, or by a glob that matches its name or its path.  
These entries SHALL go in the order of the keys as text.

A key that contains `*`, `?`, `[`, `{` or `/` SHALL be a glob.  
An entry SHALL beat the language of the file.

#### Scenario: An extension

- **WHEN** `formats` has `txt: md`
- **THEN** `a.txt` is read as Markdown

#### Scenario: Another name of the reader

- **WHEN** `formats` has `txt: markdown`
- **THEN** `a.txt` is read as Markdown

#### Scenario: Over the table

- **WHEN** `formats` has `md: text`
- **THEN** `a.md` is read as plain text

#### Scenario: A name and a glob

- **WHEN** `formats` has `NOTES: md` and `conf/*.cfg: md`
- **THEN** `NOTES` and `conf/x.cfg` are read as Markdown, and `other.cfg` as plain text

#### Scenario: The extension first

- **WHEN** `formats` has `txt: text` and `'*': md`
- **THEN** `a.txt` is read as plain text and `b.cfg` as Markdown

### Requirement: formats refuses a bad entry

A reader that is not a language hakkiri reads SHALL be an error that lists the seven.  
A key that contains a glob mark and is not a glob SHALL be an error.

#### Scenario: An unknown reader

- **WHEN** `formats` has `toml: loud`
- **THEN** hakkiri reports `` `loud` is not a language hakkiri reads: use Markdown, MDX, Rust, TypeScript, TSX, JavaScript or Text `` and exits 2

#### Scenario: A language with no reader

- **WHEN** `formats` has `cfg: python`
- **THEN** hakkiri reports `` `python` is not a language hakkiri reads: use Markdown, MDX, Rust, TypeScript, TSX, JavaScript or Text `` and exits 2

#### Scenario: A bad glob

- **WHEN** `formats` has the key `a[`
- **THEN** hakkiri reports that it is not a glob and exits 2
