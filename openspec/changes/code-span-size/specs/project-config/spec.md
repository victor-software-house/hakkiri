## ADDED Requirements

### Requirement: code_span_size sets how many words a code span counts as

`code_span_size` SHALL be a key of the project's config, of a package's settings file, and of a glob under `files`.  
Its value SHALL be the number of words a code span counts as in a word count, a whole number from 0 to 255, and it SHALL be 1 when nothing sets it.  
A value that is no whole number from 0 to 255, or a glob that gives the key a word or a list, SHALL be an error that names its line.

#### Scenario: The default

- **WHEN** no file sets `code_span_size` and a sentence holds four code spans and three other words
- **THEN** a `words` metric of the sentence reads 7.00 ([code-span-size 3][code-span-size-3])

#### Scenario: A value outside 0 to 255

- **WHEN** the config holds `code_span_size: 300`, or a glob holds `code_span_size: 300` or `code_span_size: many`
- **THEN** hakkiri names the file and the line of the key, and exits 2

### Requirement: The last code_span_size wins

The last setting SHALL win, in the order of a package's file, the project's top level, then each glob that matches the file, in the order written.  
The flag and the variable of the `check` command SHALL win over all of them.

#### Scenario: The project sets zero

- **WHEN** the project config holds `code_span_size: 0`
- **THEN** the same metric reads 3.00

#### Scenario: A package sets it and the project overrides it

- **WHEN** a package's settings file holds `code_span_size: 0`, and the project holds none, then holds `code_span_size: 2`
- **THEN** the metric reads 3.00, then 11.00

#### Scenario: A glob beats the top level

- **WHEN** the project holds `code_span_size: 2` and the glob `a.md` holds `code_span_size: 0`
- **THEN** the metric of `a.md` reads 3.00 and that of `b.md` reads 11.00

## MODIFIED Requirements

### Requirement: A package's settings file holds the keys of a project config

A package's settings file SHALL hold `files`, `level`, `vocab`, `code_span_size` and `packages`.  
A key it holds that the project config does not define SHALL be an error that names the key, its line and the package.  
A rule setting that names no loaded rule SHALL be an error, as in the project's config.

#### Scenario: A key in Vale's spelling

- **WHEN** a package's `.ctl/hakkiri.yaml` holds `MinAlertLevel: warning` on line 3
- **THEN** hakkiri reports an error naming `MinAlertLevel`, line 3 and the package, and exits 2

#### Scenario: A rule the package does not hold

- **WHEN** a package's file sets `Team.Nope: warning` and no loaded style holds `Team.Nope`
- **THEN** hakkiri reports an error naming the key and the line

[code-span-size-3]: ../../design.md#3-the-key-is-code_span_size-a-number-of-words
