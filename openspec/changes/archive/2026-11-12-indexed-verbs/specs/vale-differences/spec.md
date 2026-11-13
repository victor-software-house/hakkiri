## ADDED Requirements

### Requirement: A message verb with no value differs from Vale

hakkiri SHALL read a `%s` or `%[n]s` of a message that has no value to take as nothing, where Vale 3.26.0 prints Go's error text for it.

#### Scenario: A verb past the values

- **WHEN** a rule has `message: "[%[3]s][%[4]s][%[0]s][%s]"` and two values
- **THEN** hakkiri prints `[][][][]`, and Vale prints `[][%!s(BADINDEX)][%!s(BADINDEX)][%!s(MISSING)]`, since its message gets one empty value more than the check gives ([indexed-verbs 2][indexed-verbs-2])

[indexed-verbs-2]: ../../design.md#2-a-verb-with-no-value-is-nothing
