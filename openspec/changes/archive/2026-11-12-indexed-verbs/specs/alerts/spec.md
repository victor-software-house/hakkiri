## MODIFIED Requirements

### Requirement: A message fills %s in order

Each `%s` in a message SHALL take the next value the check gives, and `%%` SHALL read as `%`.  
A `%[n]s`, with `n` counting from 1, SHALL take the `n`th value, and a `%s` after it SHALL take the value that follows, as Go's `fmt` does.  
A `%s` or `%[n]s` with no value to take SHALL read as nothing.  
A line break inside a value SHALL read as one space.

#### Scenario: A percent sign

- **WHEN** a rule has `message: "'%s' is 100%% filler."`, `ignorecase: true` and `tokens: [basically]`, and the input is `Basically, yes.`
- **THEN** the alert message is `'Basically' is 100% filler.`

#### Scenario: A match across a line break

- **WHEN** a rule has `message: "'%s'"` and `tokens: ['very good']`, and the input is `It is very` and `good.` on two lines
- **THEN** the alert message is `'very good'`

#### Scenario: Values taken by number

- **WHEN** a rule has `extends: sequence`, `message: "'%[2]s' follows '%[1]s'."` and `tokens: [{pattern: your}, {pattern: self}]`, and the input is `Do it your self.`
- **THEN** the alert message is `'self' follows 'your'.`

#### Scenario: A plain verb after an indexed one

- **WHEN** a rule has `extends: sequence`, `message: "%[2]s %s %[1]s %s."` and `tokens: [{pattern: red}, {pattern: green}, {pattern: blue}]`, and the input is `Paint it red green blue today.`
- **THEN** the alert message is `green blue red green.`

#### Scenario: A value used twice

- **WHEN** a rule has `message: "'%[1]s' twice: '%[1]s'."` and `tokens: [very]`, and the input is `It is very good.`
- **THEN** the alert message is `'very' twice: 'very'.`

#### Scenario: A verb with no value

- **WHEN** a rule has `extends: sequence`, `message: "[%[3]s][%[0]s][%s]"` and `tokens: [{pattern: red}, {pattern: green}]`, and the input is `Paint it red green today.`
- **THEN** the alert message is `[][][]`
