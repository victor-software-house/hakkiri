## ADDED Requirements

### Requirement: A walk and a named file differ from Vale's

hakkiri SHALL skip the files git ignores in a walk and SHALL warn about a named file that no glob selects, where Vale 3.26.0 does neither.

#### Scenario: An ignored build folder

- **WHEN** `.gitignore` holds `build/` and the folder holds a file a glob selects
- **THEN** hakkiri skips it, and Vale lints it, because Vale skips only its default ignored folders and reads no git file ([gitignore 6][gitignore-6])

#### Scenario: A named file with no glob

- **WHEN** `hakkiri check README.md` runs and no glob matches it
- **THEN** hakkiri warns that no glob selects it, and Vale lints it with no rule and reports nothing ([gitignore 6][gitignore-6])

[gitignore-6]: ../../design.md#6-what-stays-different-from-vale
