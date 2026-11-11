# Tasks

Every compile, lint and test step runs in a box or the pull request's CI, not on the authoring machine.

## 1. The end of a match

- [x] 1.1 Give `Alert` a `last_line`, name the last character's line and column in it, and print `end_line` in the JSON.  
  Verify the tests for a match across a line break, across a break and markup, across three lines, and on one line fail before the change and pass after it.  
  Evidence, 2026-11-11 (UTC): the alert tests do not compile against the old `Alert`, since they read `last_line`. Against the old source in a box, the JSON tests `json_holds_no_escape_byte_under_forced_colour` and `a_match_across_lines_ends_at_its_last_character` fail at run time, and all pass with the change.
- [x] 1.2 Make `tests/vocabulary_rules.rs` compare the end of a wrapped match through Vale's count, and pin the true end of the two wrapped matches in its fixture.  
  Verify the comparison passes for the nine configs, with the wrapped rows included.  
  Evidence, 2026-11-11 (UTC): `every_config_gives_the_alerts_vale_gives` passes in a box with the wrapped rows compared, and `a_match_across_lines_ends_at_its_last_character` pins `mea Culpa` to line 1, column 5 through line 2, column 5, and `Bad Word` to line 2, column 17 through line 3, column 4.  
  `tests/ignores.rs` carries the old comment and still leaves wrapped rows out of its comparison, which a pull request outside this row changes.

## 2. The surfaces

- [x] 2.1 State the fields in `src/instructions.md`, add the deltas of the `alerts`, `check-command` and `vale-differences` specs, and add the patch changeset.  
  Verify `operator_docs` and the OpenSpec validation pass.  
  Evidence, 2026-11-11 (UTC): the full suite, `lint`, `doc`, `specs`, `specs:budget`, `lint:md`, `lint:typos` and `lint:prose` pass in a box.

## 3. Integration

- [ ] 3.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.
