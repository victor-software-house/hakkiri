## REMOVED Requirements

### Requirement: The last column is where the match ends in the file

**Reason**: The end of a match that goes on to the next line was the end of its first line.  
That end points to no character of the match, and no field contained the line the match ends on.

**Migration**: Read the requirement "A match ends at its last character" under the ADDED section.

## ADDED Requirements

### Requirement: A match ends at its last character

An alert SHALL give the line and the column of the match's last character in the file.  
Markup inside a match SHALL count toward it.  
A match on one line SHALL end on the line it starts on.  
A match that goes on to the next lines SHALL end on the line of its last character.

#### Scenario: A match across markup

- **WHEN** a rule has `tokens: ['very easy']` and the input is `**very** easy`
- **THEN** the alert starts on line 1, column 3 and ends on line 1, column 13

#### Scenario: A match across a line break

- **WHEN** a rule has `tokens: ['very good']` and the input is `It is very` and `good.` on two lines
- **THEN** the alert starts on line 1, column 7 and ends on line 2, column 4

#### Scenario: A match across a line break and markup

- **WHEN** a rule has `tokens: ['very good']` and the input is `It is **very` and `good** here.` on two lines
- **THEN** the alert starts on line 1, column 9 and ends on line 2, column 4
