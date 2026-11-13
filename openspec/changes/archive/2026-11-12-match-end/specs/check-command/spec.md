## MODIFIED Requirements

### Requirement: The JSON output is one object

`--format json` SHALL print one object with `alerts`, `failures` and `files`.  
Each alert SHALL hold `path`, `line`, `column`, `end_line`, `end_column`, `level`, `rule`, `message`, `description`, `link` and `match`.  
The output SHALL hold no ESC byte, whatever the colour variables hold.

#### Scenario: JSON with colour forced

- **WHEN** `hakkiri check --format json` runs with `CLICOLOR_FORCE=1` and a file draws an alert
- **THEN** the output parses as JSON with one alert, and holds no ESC byte

#### Scenario: The end of a match

- **WHEN** a file draws an alert for a match on line 1, columns 9 to 12
- **THEN** the alert holds `line` 1, `column` 9, `end_line` 1 and `end_column` 12
