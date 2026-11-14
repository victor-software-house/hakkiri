# Tasks

Every compile, lint and test step runs in a box or the pull request's CI, not on the authoring machine.

## 1. The probe

- [x] 1.1 Probe Vale at `66bbb85`, built with the grammars of its `grammars/build.sh`, and hakkiri over 23 files that each hold one kind of comment, and record Vale's alerts in `tests/fixtures/comment-languages`.  
  Verify `comments_read_as_vale_reads_them_but_where_decided` passes: every difference from Vale is a row of `departures.tsv`.  
  Evidence, 2026-11-12 (UTC), in a box: Go, YAML, TOML, TypeScript and JavaScript gave Vale's alerts on the fixtures apart from the departures.
  The departures are Swift, Shell, directives, indentation, columns, and 3 words that Vale's duplicate-position check drops.
- [x] 1.2 Probe the same tools over 640 files of the ctl repositories, 278 Go files and 122 Swift files, and give a cause to each difference that is left.  
  Verify the table of [design 9][design-9] holds the counts.  
  Evidence, 2026-11-12 (UTC): with Vale's check off, the differences fall to 92 rows for Go, 18 for Rust and 4 for TOML, each a Markdown reading, an indented example, a directive or a column.
- [x] 1.3 Measure the release binary before and after.  
  Verify the numbers are in [design 1][design-1] and in the ledger row.  
  Evidence, 2026-11-12 (UTC), release builds in a box: 15,264,456 bytes before, 21,171,528 after, and 76 and 82 seconds.

## 2. The readers

- [x] 2.1 Add the languages Go, Swift, YAML, TOML and Shell to `Language`, which reach their readers through langdex, and one dialect row each.  
  Verify `a_toml_file_gives_its_comments_only`, `formats_read_a_script_with_no_extension_as_shell`, `a_swift_doc_comment_has_the_doc_scope`, `a_hash_comment_is_a_line_comment` and the error text in `tests/check.rs` pass.
- [x] 2.2 Drop the comments that tools read, in every dialect, and the `#!` line.  
  Verify `tests/fixtures/comment-languages` rows for `ts-directives`, `js-directives`, `go` and `sh` show no departure, and those of `swift-directive`, `yaml-directive` and `toml-directive` show Vale-only rows.
- [x] 2.3 Mask the doc links and names at the alert.  
  Verify `a_go_link_with_a_target_is_prose` passes, and the rows for `go`, `rust` and `ts-doc` show no departure.
- [x] 2.4 Keep the indentation of a line comment of a Go, YAML, TOML or shell file.  
  Verify the rows for `go-indent`, `yaml-indent` and `toml-indent` list Vale-only rows and no hakkiri-only row.

## 3. Documents

- [x] 3.1 Update the instructions, the skill, the README, `AGENTS.md`, `openspec/config.yaml` and the specs.  
  Verify `the_command_prints_the_instructions_file` and `both_documents_name_every_flag_variable_and_exit_code` pass, and `openspec validate --all --strict` passes.

## 4. Integration

- [x] 4.1 Verify the pull request's CI passes: lint, workflow policy, prose lint, OpenSpec, the Template check and the dependency checks.  
  Evidence, 2026-11-14 (UTC): The CI of [pull request 163](https://github.com/victor-software-house/hakkiri/pull/163) passed, and it merged as `66f2cb3`.
- [x] 4.2 Take up the other languages of [HKR-017][ledger] in later changes, each with its fixture and its size.  
  Evidence, 2026-11-14 (UTC): The row stays queued for the languages this change leaves, with no plan until the next change.

[design-1]: design.md#1-the-first-tranche-is-the-languages-the-repositories-use
[design-9]: design.md#9-what-the-files-show
[ledger]: ../../../tasks.yaml
