## ADDED Requirements

### Requirement: A case's format picks its reader

A case that runs through the config SHALL read its input by the reader a `formats` entry gives `input.<format>`, and otherwise by the extension `<format>`.  
An isolated case SHALL read its input by the extension `<format>`.  
A case whose format no reader reads SHALL read its input as plain text.

#### Scenario: A format no extension names

- **WHEN** a case has `format: txt` and `input: This is very good.`, and a rule flags `very`
- **THEN** the case lints the input as plain text and reports `1:9`, both in the rule's `tests` and in a test file

#### Scenario: A formats entry

- **WHEN** the config maps `cfg: md`, a glob selects `*.cfg`, and a case has `format: cfg` and a fenced block that holds `very`
- **THEN** the case reads the input as Markdown and the rule reports nothing for the fence
