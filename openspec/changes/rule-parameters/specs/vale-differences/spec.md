## ADDED Requirements

### Requirement: Rule parameters differ from Vale's in these ways

hakkiri SHALL read a rule's parameters as Vale 3.26.0 does, apart from the scenarios below.

#### Scenario: A parameter in one section

- **WHEN** a section for `*.md` sets `Vale.Repetition[max] = 1` and a `.rs` file runs the rule
- **THEN** hakkiri applies the parameter to the files of the glob that sets it, and Vale applies it to the `.rs` file too ([rule-parameters 2][rule-parameters-2])

#### Scenario: A parameter in two sections

- **WHEN** one section sets `max = 1` and another sets `max = 5` for the same rule
- **THEN** hakkiri gives each glob its own value, and Vale gives `max = 5` to the files of both ([rule-parameters 2][rule-parameters-2])

#### Scenario: An empty styles list

- **WHEN** an empty list of styles follows a glob that set a parameter
- **THEN** hakkiri drops the parameter, as it drops a level, and Vale keeps it for every file ([rule-parameters 3][rule-parameters-3])

#### Scenario: A parameter Vale takes without a word

- **WHEN** a parameter names a rule that does not exist, holds no value, sets a negative `max`, or sets a `scope` that is none
- **THEN** hakkiri refuses it with the config file and the line, and Vale takes it with no error ([rule-parameters 4][rule-parameters-4])

#### Scenario: A parameter Vale refuses without a place

- **WHEN** a parameter has a value of the wrong kind, a name the check does not define, or a list
- **THEN** hakkiri refuses it with the config file and the line, and Vale names `[internal]`, or its rule file at line 0 or 1, and for a wrong kind cuts the message off after `1 error(s) decoding:` ([rule-parameters 4][rule-parameters-4])

#### Scenario: A flag and a name Vale accepts

- **WHEN** a flag parameter is spelled `YES` or `1`, or a parameter name is written `IGNORECASE`
- **THEN** hakkiri refuses it, since a YAML config writes `true` and `false` and names a field as the rule file does, and Vale reads the ini spelling and ignores the case of a name ([rule-parameters 4][rule-parameters-4])

#### Scenario: A rule built from a vocabulary

- **WHEN** a parameter names `Vale.Terms` or `Vale.Avoid`
- **THEN** hakkiri refuses it, and Vale ignores it ([rule-parameters 5][rule-parameters-5])

### Requirement: A consistency alert in a multi-line paragraph differs from Vale

hakkiri SHALL place a `consistency` alert on the last match of a form, though an earlier line holds the same text.  

#### Scenario: A form inside a longer word on a later line

- **WHEN** a `consistency` rule with `nonword: true` pairs `advisor` with `adviser`, and a paragraph holds `The adviser spoke first.` and then `Two advisors met two advisers.`
- **THEN** hakkiri puts the alert on line 2, column 22, the last match, and Vale searches the paragraph for the text `adviser` and puts it on line 1, column 5 ([pattern 2][pattern-2])

[rule-parameters-2]: ../../design.md#2-parameters-apply-per-glob
[rule-parameters-3]: ../../design.md#3-an-empty-styles-list-drops-the-parameters
[rule-parameters-4]: ../../design.md#4-the-parameters-are-laid-over-the-rules-fields-and-the-rules-reader-refuses-them
[rule-parameters-5]: ../../design.md#5-a-rule-built-from-a-vocabulary-takes-no-parameter
[pattern-2]: ../../../archive/2026-11-10-pattern-and-readability-checks/design.md#2-consistency-matches-as-existence-does-one-pattern-per-pair
