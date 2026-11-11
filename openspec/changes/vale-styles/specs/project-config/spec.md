## ADDED Requirements

### Requirement: A rule hakkiri cannot run fails the run until it is turned off

Each rule hakkiri cannot run that the config runs on a linted file SHALL make the run exit 2, with a `failed` line that names the rule, its file and the setting that leaves it out.  
A rule is run when its style is listed and no setting turns it `off`, or when a level or `on` turns it on.  
A rule in a style no glob lists SHALL be ignored.  
`off` SHALL leave it out, and a level or `on` SHALL NOT make it run.

#### Scenario: A script rule in a listed style

- **WHEN** `Kid/Cannot.yml` holds `extends: script`, `Kid/Parent.yml` flags `alpha`, `beta` and `gamma`, `files` holds `'*.md': {styles: [Kid]}`, and `a.md` holds `alpha beta gamma delta swapme`
- **THEN** hakkiri reports three alerts from `Kid.Parent`, prints one `failed` line for `Kid.Cannot`, and exits 2

#### Scenario: A doc selector

- **WHEN** a rule of a listed style has `scope: doc(h1)`
- **THEN** hakkiri prints the same `failed` line for it, and runs the rest of the style

#### Scenario: A rule turned off

- **WHEN** the glob also holds `Kid.Cannot: off`
- **THEN** hakkiri reports the three alerts, prints no `failed` line, and exits 0

#### Scenario: A style no glob lists

- **WHEN** the glob lists `Other` and not `Kid`
- **THEN** `Kid.Cannot` is ignored and the run exits 0

#### Scenario: A level or on

- **WHEN** a glob that lists `Other` holds `Kid.Cannot: warning`, or `Kid.Cannot: on`
- **THEN** hakkiri prints the `failed` line for `Kid.Cannot`, and exits 2

#### Scenario: A rule that extends a script rule

- **WHEN** a rule of a listed style extends a script rule
- **THEN** hakkiri prints a `failed` line for it, as for the script rule
