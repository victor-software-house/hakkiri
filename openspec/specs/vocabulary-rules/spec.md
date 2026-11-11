# vocabulary-rules Specification

## Purpose

`Vale.Terms` and `Vale.Avoid` are the rules a vocabulary builds, in a built-in style `Vale`.  
`Terms` reports an accepted term written in another case, and `Avoid` reports a rejected term.  
A vocabulary that a glob names builds the same rules for that glob's files.

## Requirements

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

#### Scenario: A pattern shown as the spelling it matched

- **WHEN** `colou?r` is accepted and the text is `The COLOR and Colour and color and colour.`
- **THEN** the alerts are ``Use 'color' instead of 'COLOR'.`` and ``Use 'colour' instead of 'Colour'.``

#### Scenario: Two terms with one lowercase form

- **WHEN** `iOS` is accepted before `IOS`, and the text is `We use ios and iOS and IOS.`
- **THEN** the alerts are ``Use 'IOS' instead of 'ios'.`` and ``Use 'IOS' instead of 'iOS'.``

#### Scenario: An escape in a term

- **WHEN** `data\Wset` is accepted and the text is `A data set, data-set, dataxset and DATA SET.`
- **THEN** the only alert is ``Use 'data\Wset' instead of 'DATA SET'.``, because hakkiri lowercases the literal text of the term and leaves `\W` a non-word character

#### Scenario: A group name, a backreference and a flag in a term

- **WHEN** `(?<Dup>Ab)\k<Dup>`, `(?U)Ab+` or `A\pLB` is accepted
- **THEN** the term builds as `(?<Dup>ab)\k<Dup>`, `(?U)ab+` or `a\pLb`, since only the literal text is lowercased

#### Scenario: A phrase that wraps

- **WHEN** `mea culpa` is accepted and the text wraps `mea` and `Culpa`, or `mea` and `culpa`, over two lines
- **THEN** the first draws ``Use 'mea culpa' instead of 'mea Culpa'.`` and the second draws nothing

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

#### Scenario: One rule turned off

- **WHEN** a glob lists `styles: [Team, Vale]` and sets `Vale.Terms: off`
- **THEN** `Vale.Terms` draws nothing and `Vale.Avoid` still draws its alerts

#### Scenario: A rule set by name

- **WHEN** a glob lists `styles: [Team]` and sets `Vale.Terms: warning` and `Vale.Avoid: suggestion`
- **THEN** `github` draws a `warning` from `Vale.Terms` and `utilize` draws a `suggestion` from `Vale.Avoid`

### Requirement: A glob's vocabularies build rules of their own

hakkiri SHALL build `Vale.<Name>.Terms` and `Vale.<Name>.Avoid` from each vocabulary a glob of `files` names, as it builds `Vale.Terms` and `Vale.Avoid`, and SHALL run them on the files that glob matches with no style or setting.

#### Scenario: A glob names Team

- **WHEN** a glob names `Team`, which accepts `harness(?:es)?` and `kubectl`, and the text is `We harness it. HARNESS it. Run KUBECTL.`
- **THEN** the alerts are ``error Vale.Team.Terms: Use 'harness' instead of 'HARNESS'.`` and ``error Vale.Team.Terms: Use 'kubectl' instead of 'KUBECTL'.``

#### Scenario: A glob's own setting

- **WHEN** a glob names `Team` and sets `Vale.Team.Terms: off`
- **THEN** `Vale.Team.Terms` still runs on its files, because the vocabulary turns it on after the glob's settings

#### Scenario: A file no glob names

- **WHEN** `Team` is named at the top level only, and the text is `Run KUBECTL.`
- **THEN** `Vale.Team.Terms` draws no alert, and `Vale.Terms` draws one only where the style `Vale` is listed
