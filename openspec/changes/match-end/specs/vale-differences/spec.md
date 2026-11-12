## ADDED Requirements

### Requirement: The end of a match that wraps a line differs from Vale

hakkiri SHALL name the line and column of a match's last character, where Vale 3.26.0 prints the first column plus the number of characters of the match, line breaks included.

#### Scenario: A match across a line break

- **WHEN** a rule matches `mea Culpa` where `mea` ends line 1 and `Culpa` starts line 2, with `mea` at column 5
- **THEN** hakkiri ends the alert on line 2, column 5, and Vale ends it at column 13 of line 1, which is past the end of the line ([match-end 1][match-end-1])

[match-end-1]: ../../design.md#1-the-end-names-the-last-character
