# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.

## 1. Spec and test

- [ ] 1.1 Correct the `metric` alert scenario of `vale-differences` and add the nested list item scenario.  
  Verify `openspec validate --all --strict` passes.
- [ ] 1.2 Add `a_metric_alert_points_at_the_first_byte_of_its_node` to `tests/metric.rs`.  
  Verify the test passes in CI.

## 2. Integration

- [ ] 2.1 Verify the pull request's CI passes: lint, prose lint, OpenSpec and the tests.
