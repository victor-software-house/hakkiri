## ADDED Requirements

### Requirement: Terms reports an accepted term written in another case

hakkiri SHALL build the rule `Vale.Terms` from the accepted terms of the named vocabularies, as a `substitution` rule with the level `error` and the message `Use '%s' instead of '%s'.`, and SHALL report an accepted term that is not written as the term is.

#### Scenario: A term in the wrong case

- **WHEN** `GitHub` is accepted and the text is `We use github and Github and GitHub.`
- **THEN** the alerts are ``error Vale.Terms: Use 'GitHub' instead of 'github'.`` and ``error Vale.Terms: Use 'GitHub' instead of 'Github'.``

#### Scenario: A plain term with a period

- **WHEN** `Node.js` is accepted and the text is `We use node.js and nodexjs.`
- **THEN** the only alert is ``Use 'Node.js' instead of 'node.js'.``

#### Scenario: A pattern

- **WHEN** `harness(?:es)?` is accepted and the text is `We saw Harnesses and harness and HARNESS.`
- **THEN** the alerts are ``Use 'harnesses' instead of 'Harnesses'.`` and ``Use 'harness' instead of 'HARNESS'.``

#### Scenario: A phrase in the wrong case

- **WHEN** `mea culpa` is accepted and the text is `We said Mea Culpa and MEA CULPA.`
- **THEN** the alerts are ``Use 'mea culpa' instead of 'Mea Culpa'.`` and ``Use 'mea culpa' instead of 'MEA CULPA'.``

#### Scenario: A term padded with spaces

- **WHEN** a list holds the term `padded` with spaces around it and the text is `This is padded and Padded.`
- **THEN** the only alert is ``Use 'padded' instead of 'Padded'.``

#### Scenario: No accepted term

- **WHEN** the named vocabularies hold no accepted term
- **THEN** no rule `Vale.Terms` exists, and a config that sets its level is an error that names the rule

### Requirement: Avoid reports a rejected term

hakkiri SHALL build the rule `Vale.Avoid` from the rejected terms, as an `existence` rule that is case-sensitive, with the level `error` and the message `Avoid using '%s'.`, and SHALL read the accepted terms as its exceptions.

#### Scenario: A rejected word

- **WHEN** `utilize` and `leverage` are rejected and the text is `Please utilize and leverage it. Utilize it, utilizes it.`
- **THEN** the alerts are ``error Vale.Avoid: Avoid using 'utilize'.`` and ``error Vale.Avoid: Avoid using 'leverage'.``, and `Utilize` and `utilizes` draw none

#### Scenario: A rejected pattern

- **WHEN** `foo.*bar` is rejected and the text is `See foo123bar and fooXbar and foobar and xfoo1bar.`
- **THEN** one alert reports the whole span: ``Avoid using 'foo123bar and fooXbar and foobar and xfoo1bar'.``

#### Scenario: A rejected phrase

- **WHEN** `Bad Word` is rejected and the text is `A bad word and a Bad Word and Bad  Word.`
- **THEN** the alerts are ``Avoid using 'Bad Word'.`` and ``Avoid using 'Bad  Word'.``

#### Scenario: A term that is accepted and rejected

- **WHEN** `Alpha` is both accepted and rejected and the text is `alpha Alpha.`
- **THEN** `Vale.Terms` reports `alpha` and `Vale.Avoid` reports nothing

### Requirement: Terms and Avoid run where the Vale style or their name is listed

hakkiri SHALL run `Vale.Terms` and `Vale.Avoid` on a file when a glob lists the style `Vale` or sets the rule by name, and on no other file, and a level set by name SHALL be their level.

#### Scenario: The Vale style is listed

- **WHEN** a glob lists `styles: [Team, Vale]` and the vocabulary accepts `GitHub`
- **THEN** `github` in a matching file draws ``error Vale.Terms: Use 'GitHub' instead of 'github'.``

#### Scenario: Only the vocabulary is named

- **WHEN** a glob lists `styles: [Team]` and the vocabulary accepts `GitHub`
- **THEN** `github` draws no alert, and the accepted terms still hold back the `Team` rules' matches

#### Scenario: A rule set by name

- **WHEN** a glob lists `styles: [Team]` and sets `Vale.Terms: warning` and `Vale.Avoid: suggestion`
- **THEN** `github` draws a `warning` from `Vale.Terms` and `utilize` draws a `suggestion` from `Vale.Avoid`
