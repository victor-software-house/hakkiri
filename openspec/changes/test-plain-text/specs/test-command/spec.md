## ADDED Requirements

### Requirement: A case's format picks its reader

A case that runs through the config SHALL read its input by the reader a `formats` entry gives `input.<format>`, and otherwise by the extension `<format>`.  
An isolated case SHALL read its input by the extension `<format>`.  
A case whose format no reader reads SHALL read its input as plain text, except that a case through the config follows `on_unknown`.

#### Scenario: A format no extension names

- **WHEN** a case has `format: txt` and `input: This is very good.`, and a rule flags `very`
- **THEN** the case lints the input as plain text and reports `1:9`, both in the rule's `tests` and in a test file

#### Scenario: A formats entry

- **WHEN** the config maps `cfg: md`, a glob selects `*.cfg`, and a case has `format: cfg` and a fenced block that holds `very`
- **THEN** the case reads the input as Markdown and the rule reports nothing for the fence

### Requirement: A case through the config follows on_unknown

A case through the config whose format no reader reads, and whose file the config selects, SHALL follow `on_unknown` as `check` does.  
`text` reads it as plain text, `warn` and `ignore` give it no alerts, and `error` makes the case fail to run with a message that names the format.

#### Scenario: An error

- **WHEN** the config holds `on_unknown: error` and `'*.txt': {styles: [Team]}`, and a case has `format: txt`
- **THEN** the case fails to run with ``no reader reads the format `txt` ``, and the run exits 2

#### Scenario: A warning or an ignore

- **WHEN** the config holds `on_unknown: warn`, or `ignore`, and a case has `format: txt` with `want: ''` and an input that a rule flags in plain text
- **THEN** the case passes, because the config does not lint such a file
