## MODIFIED Requirements

### Requirement: An unknown extends is refused

hakkiri SHALL refuse a rule whose `extends` names neither a check type nor a rule, and report an error that names the file and the value.

#### Scenario: A check type Vale does not define

- **WHEN** `Team/Grammar.yml` holds `extends: grammar`
- **THEN** hakkiri reports an error naming `Team/Grammar.yml` and `grammar`, and exits with a failure status

## ADDED Requirements

### Requirement: A rule may extend another rule

A rule whose `extends` holds a dot SHALL start from the rule that name gives and lay its own fields on top, as Vale 3.26.0 does.  
The first segment of the name SHALL be a style folder of any loaded package, and the rest SHALL be folders and a file name, in a nested folder or a `_` folder too.  
The parent's `tests` SHALL NOT carry over.

#### Scenario: A child takes the fields of its parent

- **WHEN** `Kid/Child.yml` holds `extends: Base.Parent`, `message: "Child: found %s"` and `level: error`, `Base/Parent.yml` flags `alpha`, `beta` and `gamma`, and the input is `alpha beta gamma delta swapme`
- **THEN** hakkiri reports three errors from `Kid.Child`, with the message `Child: found alpha` and the two after it

#### Scenario: A parent in a nested folder or a `_` folder

- **WHEN** a rule holds `extends: Base.Deep.Parent`, or `extends: Base._shared.Parent`
- **THEN** hakkiri reads the rule from `Base/Deep/Parent.yml`, or from `Base/_shared/Parent.yml`, which no style loads as a rule

#### Scenario: A parent in another package

- **WHEN** the config lists the packages `one` and `two`, `one/Kid/Child.yml` holds `extends: Base.Parent`, and `two/Base/Parent.yml` exists
- **THEN** hakkiri finds the parent in `two`

#### Scenario: A fetched package

- **WHEN** the config pins a Vale package by `url` and `sha256`, and its rule `Direct.Length` extends `Std.Readability.SentenceLength`, which the same zip holds
- **THEN** hakkiri reads the package from the cache and loads the rule

#### Scenario: A chain of three

- **WHEN** `Kid/Grand.yml` extends `Kid.Child`, which extends `Base.Parent`
- **THEN** the rule starts from `Base.Parent`, and `Kid.Child` and `Kid.Grand` lay their fields on top in turn

#### Scenario: Cases of the child only

- **WHEN** a parent and its child each hold one case
- **THEN** `hakkiri test` runs one case for each rule, and the child does not run the case of its parent

### Requirement: A key with a plus or a minus edits a field of the parent

`key+` SHALL add its entries to a list of the parent, or merge them into a mapping of the parent with the child's entries winning.  
`key-` SHALL remove entries from a list by their text, or keys from a mapping by name.  
A bare key SHALL replace the parent's value.

#### Scenario: An entry added to a list

- **WHEN** a child of a rule that flags `alpha`, `beta` and `gamma` holds `tokens+: [delta]`
- **THEN** the child flags all four words, with the message of the parent

#### Scenario: An entry removed from a list

- **WHEN** the child holds `tokens-: [beta]`
- **THEN** the child flags `alpha` and `gamma`

#### Scenario: A mapping edited

- **WHEN** a child of a rule with `swap: {swapme: swapped, delta: delta2}` holds `swap+: {alpha: omega}` and `swap-: [delta]`
- **THEN** the child swaps `alpha` for `omega` and `swapme` for `swapped`, and not `delta`

### Requirement: A chain Vale refuses is refused

hakkiri SHALL refuse a chain of rules that Vale 3.26.0 refuses, with an error that names the rule file and the field.  
A rule read through a parent SHALL name no line.

#### Scenario: A key written two ways

- **WHEN** a child holds `tokens` and `tokens+`
- **THEN** hakkiri reports an error naming the file and `tokens+`: it both replaces `tokens` and edits it, so pick one, and exits 2

#### Scenario: An entry the parent lacks

- **WHEN** a child holds `tokens-: [missing]`, and the parent's `tokens` has no `missing`
- **THEN** hakkiri reports that the parent has no entry `missing` to remove, and exits 2

#### Scenario: A cycle

- **WHEN** `Kid/Child.yml` holds `extends: Kid.Child`
- **THEN** hakkiri reports an inheritance cycle, and exits 2

#### Scenario: A chain over ten rules

- **WHEN** a rule extends a chain of twelve rules
- **THEN** hakkiri reports that the chain is over 10 rules deep, and exits 2

#### Scenario: A parent that does not exist

- **WHEN** a rule holds `extends: Base.Absent` and no package holds `Base/Absent.yml`
- **THEN** hakkiri reports that `Base.Absent` is not on the search path, and exits 2

#### Scenario: A built-in parent

- **WHEN** a rule holds `extends: Vale.Terms`
- **THEN** hakkiri reports that `Vale.Terms` is a built-in rule with no file to extend, and exits 2

### Requirement: A rule hakkiri cannot run loads no rule

A rule that is valid Vale and that hakkiri cannot run, a rule that extends `script` or whose `scope` is a `doc()` selector, SHALL give no rule.  
It SHALL NOT stop the other rules of its style from loading.  
The loader SHALL keep its name and its file, so a config can turn it `off`.

#### Scenario: A script rule beside a rule that runs

- **WHEN** `Kid/Cannot.yml` holds `extends: script` and `Kid/Parent.yml` flags `alpha`
- **THEN** the style `Kid` loads the rule `Kid.Parent` and holds `Kid.Cannot` as a rule it cannot run

#### Scenario: A doc selector

- **WHEN** a rule holds `scope: doc(h1)`
- **THEN** the style holds it as a rule it cannot run

#### Scenario: A child of a script rule

- **WHEN** a rule extends a rule that extends `script`
- **THEN** it is a rule hakkiri cannot run too
