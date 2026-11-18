## ADDED Requirements

### Requirement: code_span_size sets how many words a code span counts as

`code_span_size` SHALL be a key of the project's config, of a package's settings file, and of a glob under `files`.  
Its value SHALL be the number of words a code span counts as in a word count.  
It SHALL be a whole number from 0 to 255, and 1 when nothing sets it.

A value that is no whole number from 0 to 255 SHALL be an error that gives its line.  
So SHALL a glob that gives the key a word or a list.

#### Scenario: The default

- **WHEN** no file sets `code_span_size` and a sentence contains four code spans and three other words
- **THEN** a `words` metric of the sentence reads 7.00 ([code-span-size 3][code-span-size-3])

#### Scenario: A value outside 0 to 255

- **WHEN** the config contains `code_span_size: 300`, or a glob contains `code_span_size: 300` or `code_span_size: many`
- **THEN** hakkiri reports the file and the line of the key, and exits 2

### Requirement: code_span_size resolves by the settings cascade

`code_span_size` SHALL resolve by the settings cascade.  
It starts with the top levels of a package's file and of the project, then each glob that matches the file, in rank order.  
The last one SHALL win.  
The flag and the variable of the `check` command SHALL win over all of them.

#### Scenario: The project sets zero

- **WHEN** the project config contains `code_span_size: 0`
- **THEN** the same metric reads 3.00

#### Scenario: A package sets it and the project overrides it

- **WHEN** a package's settings file contains `code_span_size: 0`, and the project contains none, then contains `code_span_size: 2`
- **THEN** the metric reads 3.00, then 11.00

#### Scenario: A glob beats the top level

- **WHEN** the project contains `code_span_size: 2` and the glob `a.md` contains `code_span_size: 0`
- **THEN** the metric of `a.md` reads 3.00 and that of `b.md` reads 11.00

#### Scenario: A package glob beats the project's top level

- **WHEN** a package's settings file contains the glob `'*.md': {code_span_size: 0}`, and the project contains `code_span_size: 2`, then `code_span_size: !override 2`
- **THEN** the metric of `a.md` reads 3.00, then 11.00

## MODIFIED Requirements

### Requirement: A package's settings file contains the keys of a project config

The settings file of a package SHALL contain `files`, `level`, `vocab`, `code_span_size` and `packages`.  
A key in it that the project config does not define SHALL be an error that gives the key, its line and the package.  
Like in the project's config, a rule setting for no loaded rule SHALL be an error.

#### Scenario: A key in Vale's spelling

- **WHEN** a package's `.ctl/hakkiri.yaml` contains `MinAlertLevel: warning` on line 3
- **THEN** hakkiri reports an error that shows `MinAlertLevel`, line 3 and the package, and exits 2

#### Scenario: A rule the package does not contain

- **WHEN** a package's file sets `Team.Nope: warning` and no loaded style contains `Team.Nope`
- **THEN** hakkiri reports an error that shows the key and the line

[code-span-size-3]: ../../design.md#3-the-key-is-code_span_size-a-number-of-words
