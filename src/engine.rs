//! The engine: one file's rules run over its tree, and the alerts they give.

use crate::alert::{Alert, Locator, settle};
use crate::check::{run, run_summary};
use crate::metric::{Variables, variables};
use crate::rule::{Check, Rule};
use crate::select::{SelectError, select};
use crate::summary::{Summary, summarize};
use crate::tree::Tree;

/// The alerts of `rules` in one file, sorted by line and then by column.
///
/// `tree` is the file's tree, `source` its text and `extension` its extension
/// without the dot. Alerts at one place keep the order of their rules.
/// A rule whose pattern gives up on a block reports one `error` alert at the
/// block, and the other rules still run.
/// A `metric` or `readability` rule with no `scope`, or `scope: text`, measures
/// the prose of the whole file, the comments of a code file included, and
/// reports at its start; any other scope measures each block it selects.
/// A `readability` rule reports at column 1 of the first line of each block.
/// The counts and scores of that prose are worked out once, and each rule adds
/// only its `occurrences`.
///
/// # Errors
///
/// Returns a [`SelectError`] when `tree` is not the tree of `source`.
pub fn lint(
    rules: &[Rule],
    tree: &Tree,
    source: &str,
    extension: &str,
) -> Result<Vec<Alert>, SelectError> {
    let mut locator = Locator::new(source);
    let mut alerts = Vec::new();
    let mut measured: Option<(Summary, Option<Variables>)> = None;
    for rule in rules {
        if matches!(rule.check, Check::Metric(_) | Check::Readability(_))
            && rule.scope.measures_file()
        {
            if measured.is_none() {
                // Vale takes the whole file as the summary of a plain-text file
                // (`lint.go` L539-544 at 66bbb85). That applies once hakkiri
                // reads a plain-text language, which it does not yet.
                let summary = summarize(tree, source)?;
                let base = variables(&summary.content, summary.elements);
                measured = Some((summary, base));
            }
            let hit = measured.as_ref().map_or(Ok(None), |(summary, base)| {
                base.as_ref()
                    .map_or(Ok(None), |base| run_summary(&rule.check, summary, base))
            });
            match hit {
                Ok(hit) => {
                    alerts.extend(hit.map(|hit| locator.alert_at(rule, 0, hit.messages(rule))));
                }
                Err(error) => alerts.push(locator.failure_at(rule, 0, error.to_string())),
            }
            continue;
        }
        let mut found = Vec::new();
        let mut failure = None;
        'blocks: for block in select(&rule.scope, tree, source, extension)? {
            match run(&rule.check, &block.text) {
                Ok(hits) => {
                    for hit in hits {
                        let messages = hit.messages(rule);
                        found.extend(if matches!(rule.check, Check::Readability(_)) {
                            Some(locator.alert_on_line(rule, &block.text, messages))
                        } else {
                            locator.alert(rule, &block.text, hit.range, messages)
                        });
                    }
                }
                Err(error) => {
                    failure = Some(locator.failure(rule, &block.text, error.to_string()));
                    break 'blocks;
                }
            }
        }
        alerts.extend(settle(found, rule.limit));
        alerts.extend(failure);
    }
    alerts.sort_by_key(|alert| (alert.line, alert.first_column));
    Ok(alerts)
}
