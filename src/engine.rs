//! The engine: one file's rules run over its tree, and the alerts they give.

use std::borrow::Borrow;

use crate::alert::{Alert, Locator, settle};
use crate::check::{Memory, run, run_summary};
use crate::control::{Controls, styles_of};
use crate::metric::{Variables, variables};
use crate::rule::{Check, Exceptions, Rule};
use crate::scope::Selector;
use crate::select::{Block, SelectError, select};
use crate::summary::{Summary, summarize};
use crate::tree::{Scope, Tree};

/// The alerts of `rules` in one file, sorted by line and then by column.
///
/// `rules` is a slice of rules or of references to them, so a caller that picks
/// a few of many rules for a file lends them without a copy.
/// `tree` is the file's tree, `source` its text and `extension` its extension
/// without the dot. Alerts at one place keep the order of their rules.
/// A rule whose pattern gives up on a block reports one `error` alert at the
/// block, and the other rules still run.
/// A comment of a Markdown file can turn rules off and on: see
/// [`crate::control`]. A rule that a comment turned off for the element of a
/// block reads no text in it, and an alert between the comment that turned its
/// rule off and the one that turned it on is left out.
/// A `raw` rule reads the whole file, and only `off` gates it.
/// A `metric` or `readability` rule with no `scope`, or `scope: text`, measures
/// the prose of the whole file, the comments of a code file included, and
/// reports at its start; any other scope measures each block it selects.
/// A `readability` rule reports at column 1 of the first line of each block.
/// The counts and scores of that prose are worked out once, and each rule adds
/// only its `occurrences`.
/// The blocks of a scope are selected once, and every rule that reads the scope
/// shares them.
///
/// # Errors
///
/// Returns a [`SelectError`] when `tree` is not the tree of `source`.
pub fn lint<R: Borrow<Rule>>(
    rules: &[R],
    tree: &Tree,
    source: &str,
    extension: &str,
) -> Result<Vec<Alert>, SelectError> {
    lint_accepting(rules, tree, source, extension, &[])
}

/// The alerts of `rules` in one file, as [`lint`] gives them, with the
/// vocabularies of the file's globs held back too.
///
/// `accepted` holds the [`Exceptions`] of each vocabulary a glob that matches
/// the file names.
/// An `existence`, `repetition`, `spelling` or `substitution` rule holds back
/// each match they hold back, besides its own exceptions, whatever it sets for
/// `vocab` and `nonword`.
/// A `capitalization` or `conditional` rule reads none of them.
///
/// # Errors
///
/// Returns a [`SelectError`] when `tree` is not the tree of `source`.
pub fn lint_accepting<R: Borrow<Rule>>(
    rules: &[R],
    tree: &Tree,
    source: &str,
    extension: &str,
    accepted: &[&Exceptions],
) -> Result<Vec<Alert>, SelectError> {
    let mut locator = Locator::new(source);
    let controls = Controls::new(
        &tree.comments,
        source,
        &styles_of(rules.iter().map(|rule| {
            let rule: &Rule = rule.borrow();
            rule.name.as_str()
        })),
    );
    let mut alerts = Vec::new();
    let mut measured: Option<(Summary, Option<Variables>)> = None;
    // The blocks of each scope, selected once for every rule that reads it:
    // selecting splits the sentences and builds the text of each block.
    let mut selected: Vec<(&Selector, Vec<Block>)> = Vec::new();
    for rule in rules {
        let rule: &Rule = rule.borrow();
        if matches!(rule.check, Check::Metric(_) | Check::Readability(_))
            && rule.scope.measures_file()
        {
            if measured.is_none() {
                // Vale takes the whole file as the summary of a plain-text file
                // (`lint.go` lines 539 to 544 at 66bbb85). That applies once
                // hakkiri reads a plain-text language, which it does not yet.
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
        let mut memory = Memory::default();
        let at = if let Some(at) = selected.iter().position(|(scope, _)| *scope == &rule.scope) {
            at
        } else {
            selected.push((&rule.scope, select(&rule.scope, tree, source, extension)?));
            selected.len() - 1
        };
        let blocks: &[Block] = selected.get(at).map_or(&[], |(_, blocks)| blocks);
        'blocks: for block in blocks {
            // Vale reads the whole file at the end, after it has forgotten
            // every comment but `off`.
            let gated = if block.scope == Scope::Raw {
                controls.off_at(block.closes)
            } else {
                controls.gated(&rule.name, block.closes)
            };
            if gated {
                continue;
            }
            match run(
                &rule.check,
                &block.text,
                &mut memory,
                tree.language.extension(),
                accepted,
            ) {
                Ok(hits) => {
                    for hit in hits {
                        let messages = hit.messages(rule);
                        let alert = if matches!(rule.check, Check::Readability(_)) {
                            Some(locator.alert_on_line(rule, &block.text, messages))
                        } else {
                            locator.alert(rule, &block.text, hit.range, messages)
                        };
                        found.extend(alert.filter(|alert| {
                            !controls.hides(
                                &alert.rule,
                                &alert.matched,
                                (alert.line, alert.first_column),
                                block.closes,
                            )
                        }));
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

#[cfg(test)]
mod tests;
