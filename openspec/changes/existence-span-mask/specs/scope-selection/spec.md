## MODIFIED Requirements

### Requirement: A match on a code span is not reported

A match that overlaps a code span SHALL give no alert from `substitution`.
An `existence` rule SHALL read each code span as one `*`, a byte that no word and no space matches, and SHALL give no alert for a match that starts or ends on a code span.
A match with text on both sides of a code span SHALL give an alert, and its matched text SHALL read the span as `x`.

#### Scenario: A pattern that reaches into a code span

- **WHEN** a rule has `tokens: ['run \w+']`, and the input is ``Then run `cargo test` now.``
- **THEN** hakkiri reports no alert

#### Scenario: A match across a code span

- **WHEN** an `existence` rule has `tokens: ['Run [^.]*now']`, and the input is ``Run `cargo test` now.``
- **THEN** hakkiri reports one alert, with the matched text `Run x now`

#### Scenario: A match that starts or ends on a code span

- **WHEN** an `existence` rule has `tokens: ['[^ ]+ now']` or `tokens: ['Run [^ ]+']`, and the input is ``Run `cargo` now.``
- **THEN** hakkiri reports no alert
