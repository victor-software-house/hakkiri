# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.

## 1. Spec and test

- [x] 1.1 Correct the `metric` alert scenario of `vale-differences` and add the nested list item scenario.  
  Verify `openspec validate --all --strict` passes.  
  Evidence, 2026-11-14 (UTC): the spec delta of this change corrects the `metric` scenario and adds the nested list item scenario, and `openspec validate --all --strict` passes.
- [x] 1.2 Add `a_metric_alert_points_at_the_first_byte_of_its_node` to `tests/metric.rs`.  
  Verify the test passes in CI.  
  Evidence, 2026-11-14 (UTC): the CI of [pull request 167](https://github.com/victor-software-house/hakkiri/pull/167) passed at `8444b61`, `a_metric_alert_points_at_the_first_byte_of_its_node` among its tests.

## 2. Integration

- [x] 2.1 Verify the pull request's CI passes: lint, prose lint, OpenSpec and the tests.  
  Evidence, 2026-11-14 (UTC): the CI of [pull request 167](https://github.com/victor-software-house/hakkiri/pull/167) passed at `8444b61`.
