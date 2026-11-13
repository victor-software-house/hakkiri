## ADDED Requirements

### Requirement: --code-span-size sets the words of a code span for every file

`check` SHALL take `--code-span-size <WORDS>`, a whole number from 0 to 255, and read `HAKKIRI_CODE_SPAN_SIZE` when the flag is not given.  
The flag SHALL win over the variable, and the variable over every `code_span_size` of a config file, a glob's included.  
A value that is no whole number from 0 to 255 SHALL be a usage error.

#### Scenario: The variable beats the config

- **WHEN** the config holds `code_span_size: 2` and `HAKKIRI_CODE_SPAN_SIZE=0` is set
- **THEN** a `words` metric over a sentence of four code spans and three words reads 3.00

#### Scenario: The flag beats the variable

- **WHEN** `--code-span-size 1` is given with `HAKKIRI_CODE_SPAN_SIZE=0`
- **THEN** the same metric reads 7.00 ([code-span-size 5][code-span-size-5])

[code-span-size-5]: ../../design.md#5-package-project-glob-variable-flag
