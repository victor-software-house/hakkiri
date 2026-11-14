## ADDED Requirements

### Requirement: The style Vale holds Vale.Spelling

hakkiri SHALL give the style `Vale` a rule `Vale.Spelling` that reports each word the built-in dictionary lacks, with the message `Did you really mean '%s'?`, the level `error` and the scope `text`.  
A config SHALL set its level, turn it off, and set its parameters as it does any rule.

#### Scenario: A project lists Vale

- **WHEN** a glob lists `styles: [Vale]` and the input is `We blorft mail.`
- **THEN** hakkiri reports one alert, with the rule `Vale.Spelling`, the level `error`, the matched text `blorft` and the message `Did you really mean 'blorft'?`

#### Scenario: Turned off

- **WHEN** the same glob sets `Vale.Spelling: off`
- **THEN** hakkiri reports no alert

### Requirement: Vale.Spelling adds the dictionaries of each package

`Vale.Spelling` SHALL read every dictionary in the `config/dictionaries` folder of each package of the project, in the order of the packages, and accept a word that one holds beside the built-in dictionary.  
A parameter `dictionaries`, `aff` or `dic` SHALL replace them, and `append` SHALL keep the built-in dictionary.

#### Scenario: A dictionary in a package

- **WHEN** `config/dictionaries/team.dic` holds `blorft`, and the input is `We blorft mail.`
- **THEN** hakkiri reports no alert

#### Scenario: A dictionary named by a parameter

- **WHEN** the config sets `params: {Vale.Spelling: {dictionaries: team}}` and the input is `We blorft mail.`
- **THEN** hakkiri reports the words `We` and `mail`, and not `blorft`

### Requirement: Vale.Spelling accepts the words of the ignore folders

`Vale.Spelling` SHALL accept each word of each `.txt` file under the `config/ignore` folder of each package, folders below it included.  
A file with another extension SHALL be no list.

#### Scenario: A list in a folder below

- **WHEN** `config/ignore/sub/more.txt` holds `foxs` and the input is `The foxs ran.`
- **THEN** hakkiri reports no alert

#### Scenario: A file that is not a list

- **WHEN** `config/ignore/notes.md` holds `xyz` and the input is `The xyz plan.`
- **THEN** hakkiri reports `xyz`
