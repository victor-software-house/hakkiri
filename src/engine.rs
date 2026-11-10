//! The engine: one file's rules run over its tree, and the alerts they give.

use crate::alert::{Alert, Locator, settle};
use crate::check::run;
use crate::rule::Rule;
use crate::select::{SelectError, select};
use crate::tree::Tree;

/// The alerts of `rules` in one file, sorted by line and then by column.
///
/// `tree` is the file's tree, `source` its text and `extension` its extension
/// without the dot. Alerts at one place keep the order of their rules.
/// A rule whose pattern gives up on a block reports one `error` alert at the
/// block, and the other rules still run.
///
/// Only `existence` rules run; the other check types come in a later change
/// of the stack.
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
    for rule in rules {
        let mut found = Vec::new();
        let mut failure = None;
        'blocks: for block in select(&rule.scope, tree, source, extension)? {
            match run(&rule.check, &block.text) {
                Ok(hits) => {
                    for hit in hits {
                        let values: Vec<&str> = hit.values.iter().map(String::as_str).collect();
                        found.extend(locator.alert(rule, &block.text, hit.range, &values));
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
