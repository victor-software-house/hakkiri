//! The engine's unit tests.

#![expect(
    clippy::panic_in_result_fn,
    reason = "A test returns Result for `?` and fails through an assertion."
)]

use super::lint;
use crate::rule::Rule;
use crate::select::SELECTIONS;
use crate::{Language, parse};

/// A rule that flags the word `needle`, in `scope`.
fn rule(name: &str, scope: &str) -> Result<Rule, Box<dyn std::error::Error>> {
    Ok(Rule::from_text(
        &format!("Team.{name}"),
        &format!("Team/{name}.yml"),
        &format!("extends: existence\nmessage: \"%s\"\nscope: {scope}\ntokens: [{name}]\n"),
    )?)
}

#[test]
fn rules_that_read_one_scope_select_it_once() -> Result<(), Box<dyn std::error::Error>> {
    let source = "One needle here. A second sentence.\n\nA paragraph.\n";
    let tree = parse(Language::Markdown, source)?;
    let rules = [
        rule("one", "sentence")?,
        rule("two", "sentence")?,
        rule("three", "sentence")?,
        rule("four", "paragraph")?,
    ];
    let before = SELECTIONS.with(std::cell::Cell::get);
    lint(&rules, &tree, source, "md")?;
    let made = SELECTIONS.with(std::cell::Cell::get) - before;
    assert_eq!(
        made, 2,
        "one selection for each scope, not one for each rule"
    );
    Ok(())
}
