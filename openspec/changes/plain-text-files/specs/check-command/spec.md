## REMOVED Requirements

### Requirement: A language is read by extension

**Reason**: OpenSpec cannot drop a scenario of a modified requirement.
Both scenarios of this one no longer hold: a file that no language reads was skipped in a walk and refused when named.
So the requirement is removed and added again below under a new name.

**Migration**: Read the requirement named "A reader is chosen by formats, then by extension" under the ADDED section.

## ADDED Requirements

### Requirement: A reader is chosen by formats, then by extension

hakkiri SHALL read a file by the reader its `formats` entry gives it, and otherwise by its extension: `.md`, `.mdx`, `.rs`, `.ts`, `.tsx` or `.js` as Markdown, MDX, Rust, TypeScript, TSX or JavaScript.  
A file that a glob selects and that no reader reads SHALL be read as plain text, unless `on_unknown` says otherwise.

#### Scenario: A file no reader reads, found by a walk

- **WHEN** a walk finds `notes.txt`, `Cargo.toml` and `Makefile`, and the glob `*` selects them
- **THEN** hakkiri lints each as plain text

#### Scenario: A file no reader reads, named

- **WHEN** `hakkiri check notes.txt` runs and a glob selects `notes.txt`
- **THEN** hakkiri lints it as plain text

#### Scenario: The reader of an extension

- **WHEN** `formats` holds `txt: md` and `a.txt` holds a fenced block that holds `very`
- **THEN** hakkiri reads `a.txt` as Markdown and reports nothing for the fence

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

A file read as plain text that holds a NUL byte in its first 8192 bytes SHALL be skipped with no alert and no message, whether a walk finds it or the command line names it.

#### Scenario: A picture

- **WHEN** `logo.png` holds a NUL byte and the glob `*` selects it
- **THEN** hakkiri skips it, counts no file for it, and exits 0

### Requirement: A file a formats entry maps has the extension of its reader

For the scopes of the rules that run on it, a file whose reader a `formats` entry gives SHALL have the extension of that reader, and any other file its own extension.

#### Scenario: A mapped file

- **WHEN** `formats` maps `txt: md`, `a.txt` holds `# beta`, a blank line and `beta alpha.`, one rule has `scope: text.md` and `tokens: [alpha]`, and another has `scope: text & ~heading.md` and `tokens: [beta]`
- **THEN** hakkiri reports `beta` on line 3 and `alpha` on line 3, and none in the heading

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
