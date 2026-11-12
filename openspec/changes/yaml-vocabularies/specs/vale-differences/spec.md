## REMOVED Requirements

### Requirement: A YAML vocabulary is a known gap

**Reason**: hakkiri reads `vocab.yml` and `vocab.yaml`, so the gap closes.

**Migration**: Read the requirement "A YAML vocabulary differs from Vale in five places" under the ADDED section for what still differs.

## ADDED Requirements

### Requirement: A YAML vocabulary differs from Vale in five places

hakkiri SHALL read a `vocab.yml` or `vocab.yaml` as each scenario below says, where Vale 3.26.0 does otherwise.

#### Scenario: Several wrong entries

- **WHEN** a file holds three wrong entries
- **THEN** hakkiri reports all three with their lines, and Vale stops at the first ([yaml-vocabularies 2][yaml-vocabularies-2])

#### Scenario: A key the file does not define

- **WHEN** a file holds `accpt:` beside or instead of `accept:`
- **THEN** hakkiri refuses the file on the line of the key, and Vale ignores the key and accepts nothing from it ([yaml-vocabularies 1][yaml-vocabularies-1])

#### Scenario: The affix file

- **WHEN** `affix: custom` names a file that only `$DICPATH` holds
- **THEN** hakkiri reports that `custom.aff` is in no package, and Vale finds it there; hakkiri also looks in every other loaded package, which Vale does not ([yaml-vocabularies 6][yaml-vocabularies-6])

#### Scenario: A name with a folder

- **WHEN** `affix` is `../outside`
- **THEN** hakkiri refuses the name, and Vale joins it to the folder ([yaml-vocabularies 6][yaml-vocabularies-6])

#### Scenario: A syntax error of the file

- **WHEN** the last list of the file is `[a` on line 3, with no closing bracket
- **THEN** hakkiri names line 3, where the bracket opens, and Vale names line 2, which holds a complete entry and no fault ([yaml-vocabularies 2][yaml-vocabularies-2])

[yaml-vocabularies-1]: ../../design.md#1-the-file-parses-through-ctl-cores-input
[yaml-vocabularies-2]: ../../design.md#2-an-entry-is-checked-on-the-node-and-every-problem-is-reported
[yaml-vocabularies-6]: ../../design.md#6-the-affix-file-comes-from-the-packages-never-from-dicpath
