//! Loading: the rule-loading scenarios of the rule-engine change.
//!
//! Each test loads one rule file and checks the rule it gives, or the error
//! that refuses it.

#![expect(
    clippy::panic_in_result_fn,
    reason = "A test returns Result for `?` and fails through an assertion."
)]

use std::error::Error;
use std::fs;
use std::path::PathBuf;

use ctl_core::indoc;
use hakkiri::rule::{Check, Level, LoadError, Rule, load_style};

type TestResult = Result<(), Box<dyn Error>>;

/// Loads `text` as the file `Team/Rule.yml`.
fn load(text: &str) -> Result<Rule, LoadError> {
    Rule::from_text("Team.Rule", "Team/Rule.yml", text)
}

/// The one problem of a refused rule file.
fn refused(text: &str) -> Result<(String, Option<usize>, String), Box<dyn Error>> {
    let error = load(text).err().ok_or("the rule loaded")?;
    let [problem] = error.problems.as_slice() else {
        return Err(format!("expected one problem: {error}").into());
    };
    assert_eq!(problem.file, "Team/Rule.yml");
    Ok((problem.field.clone(), problem.line, error.to_string()))
}

/// A style folder in the temporary directory, removed when it drops.
struct Folder(PathBuf);

impl Folder {
    fn new(name: &str) -> Result<Self, Box<dyn Error>> {
        let path = std::env::temp_dir().join(format!("hakkiri-{}-{name}", std::process::id()));
        fs::create_dir_all(path.join("Team"))?;
        Ok(Self(path.join("Team")))
    }
}

impl Drop for Folder {
    fn drop(&mut self) {
        if let Some(parent) = self.0.parent() {
            fs::remove_dir_all(parent).ok();
        }
    }
}

#[test]
fn a_folder_with_a_rule_and_a_readme() -> TestResult {
    let folder = Folder::new("readme")?;
    fs::write(
        folder.0.join("Filler.yml"),
        "extends: existence\nmessage: \"'%s'\"\ntokens: [very]\n",
    )?;
    fs::write(folder.0.join("README.md"), "Notes on the style.\n")?;
    fs::write(folder.0.join("Other.yaml"), "extends: grammar\n")?;
    let rules = load_style(&folder.0)?;
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].name, "Team.Filler");
    Ok(())
}

#[test]
fn every_problem_of_a_folder_is_reported() -> TestResult {
    let folder = Folder::new("problems")?;
    fs::write(folder.0.join("A.yml"), "extends: existence\ntokens: [a]\n")?;
    fs::write(folder.0.join("B.yml"), "extends: grammar\nmessage: m\n")?;
    let error = load_style(&folder.0).err().ok_or("the folder loaded")?;
    let files: Vec<_> = error.problems.iter().map(|p| p.file.as_str()).collect();
    assert_eq!(files.len(), 2);
    assert!(files[0].ends_with("A.yml"));
    assert!(files[1].ends_with("B.yml"));
    Ok(())
}

#[test]
fn a_rule_with_no_message() -> TestResult {
    let (_, _, message) = refused(indoc! {"
        extends: existence
        tokens: [very]
    "})?;
    assert!(message.contains("Team/Rule.yml"), "{message}");
    assert!(message.contains("message"), "{message}");
    Ok(())
}

#[test]
fn a_rule_with_no_extends() -> TestResult {
    let (_, _, message) = refused("message: m\n")?;
    assert!(message.contains("extends"), "{message}");
    Ok(())
}

#[test]
fn a_check_type_vale_does_not_define() -> TestResult {
    let (field, line, message) = refused(indoc! {"
        extends: grammar
        message: m
    "})?;
    assert_eq!(field, "extends");
    assert_eq!(line, Some(1));
    assert!(message.contains("Team/Rule.yml"), "{message}");
    assert!(message.contains("grammar"), "{message}");
    Ok(())
}

#[test]
fn a_script_rule_is_refused() -> TestResult {
    let (field, _, message) = refused(indoc! {"
        extends: script
        message: m
        scope: text
        script: |
          text := import(\"text\")
    "})?;
    assert_eq!(field, "extends");
    assert!(message.contains("script"), "{message}");
    Ok(())
}

#[test]
fn a_check_type_for_a_later_change_is_refused() -> TestResult {
    let (_, _, message) = refused(indoc! {"
        extends: capitalization
        message: m
    "})?;
    assert!(message.contains("capitalization"), "{message}");
    Ok(())
}

#[test]
fn token_in_an_existence_rule() -> TestResult {
    let (_, _, message) = refused(indoc! {"
        extends: existence
        message: m
        token: very
    "})?;
    assert!(message.contains("Team/Rule.yml:3"), "{message}");
    assert!(message.contains("token"), "{message}");
    Ok(())
}

#[test]
fn a_count_that_is_not_a_number() -> TestResult {
    let (field, line, message) = refused(indoc! {"
        extends: occurrence
        message: m
        token: a
        max: many
    "})?;
    assert_eq!(field, "max");
    assert_eq!(line, Some(4));
    assert!(message.contains("many"), "{message}");
    Ok(())
}

#[test]
fn a_number_in_quotes() -> TestResult {
    let rule = load(indoc! {"
        extends: occurrence
        message: m
        token: a
        max: '3'
    "})?;
    let Check::Occurrence(occurrence) = rule.check else {
        return Err("not an occurrence rule".into());
    };
    assert_eq!(occurrence.max, 3);
    Ok(())
}

#[test]
fn a_flag_that_is_not_a_boolean() -> TestResult {
    let (field, line, _) = refused(indoc! {"
        extends: existence
        message: m
        tokens: [a]
        nonword: sometimes
    "})?;
    assert_eq!(field, "nonword");
    assert_eq!(line, Some(4));
    Ok(())
}

#[test]
fn a_rule_with_no_level() -> TestResult {
    let rule = load(indoc! {"
        extends: existence
        message: m
        tokens: [very]
    "})?;
    assert_eq!(rule.level, Level::Warning);
    Ok(())
}

#[test]
fn each_level_loads() -> TestResult {
    for (name, level) in [
        ("suggestion", Level::Suggestion),
        ("warning", Level::Warning),
        ("error", Level::Error),
    ] {
        let rule = load(&format!(
            "extends: existence\nmessage: m\nlevel: {name}\ntokens: [a]\n"
        ))?;
        assert_eq!(rule.level, level);
    }
    Ok(())
}

#[test]
fn a_level_vale_does_not_define() -> TestResult {
    let (field, line, message) = refused(indoc! {"
        extends: existence
        message: m
        level: info
        tokens: [a]
    "})?;
    assert_eq!(field, "level");
    assert_eq!(line, Some(3));
    assert!(message.contains("Team/Rule.yml"), "{message}");
    Ok(())
}

#[test]
fn an_unclosed_group() -> TestResult {
    let error = load(indoc! {"
        extends: existence
        message: m
        tokens: ['(unclosed']
    "})
    .err()
    .ok_or("the rule loaded")?;
    let message = error.to_string();
    assert!(message.contains("Team.Rule"), "{message}");
    assert!(message.contains("(unclosed"), "{message}");
    Ok(())
}

#[test]
fn a_pattern_that_needs_lookaround_loads() -> TestResult {
    let rule = load(indoc! {r"
        extends: existence
        message: m
        nonword: true
        tokens: ['(?<=a)b']
    "})?;
    let Check::Existence(existence) = rule.check else {
        return Err("not an existence rule".into());
    };
    assert!(existence.pattern.backtracks());
    Ok(())
}

#[test]
fn a_tab_in_the_indentation() -> TestResult {
    let error = load("extends: existence\nmessage: m\ntokens:\n  - a\n\t- b\n")
        .err()
        .ok_or("the rule loaded")?;
    let [problem] = error.problems.as_slice() else {
        return Err(format!("expected one problem: {error}").into());
    };
    assert_eq!(problem.file, "Team/Rule.yml");
    assert_eq!(problem.line, Some(5), "{error}");
    Ok(())
}

#[test]
fn yes_and_no_stay_strings() -> TestResult {
    let rule = load(indoc! {"
        extends: existence
        message: m
        ignorecase: true
        tokens: [no]
    "})?;
    let Check::Existence(existence) = rule.check else {
        return Err("not an existence rule".into());
    };
    assert!(existence.pattern.source().contains("no"));
    Ok(())
}

#[test]
fn an_action_loads() -> TestResult {
    let rule = load(indoc! {"
        extends: existence
        message: Remove '%s'.
        tokens: [very]
        action:
          name: remove
    "})?;
    let action = rule.action.ok_or("no action")?;
    assert_eq!(action.name, "remove");
    assert_eq!(action.params, Vec::<String>::new());
    Ok(())
}

#[test]
fn a_vocab_flag_loads() -> TestResult {
    load(indoc! {"
        extends: substitution
        message: m
        vocab: false
        swap: {a: b}
    "})?;
    Ok(())
}

#[test]
fn affix_is_refused() -> TestResult {
    let (field, line, message) = refused(indoc! {"
        extends: existence
        message: m
        affix: en
        tokens: [a]
    "})?;
    assert_eq!(field, "affix");
    assert_eq!(line, Some(3));
    assert!(message.contains("affix"), "{message}");
    Ok(())
}

#[test]
fn a_rule_with_no_tokens_or_raw_is_refused() -> TestResult {
    let (field, line, message) = refused(indoc! {"
        extends: existence
        message: m
    "})?;
    assert_eq!(field, "");
    assert_eq!(line, Some(1));
    assert!(message.contains("`tokens` and `raw`"), "{message}");
    Ok(())
}

#[test]
fn a_rule_with_only_blank_tokens_is_refused() -> TestResult {
    let (_, line, message) = refused(indoc! {"
        extends: existence
        message: m
        tokens: ['', '  ']
    "})?;
    assert_eq!(line, Some(1));
    assert!(message.contains("`tokens` and `raw`"), "{message}");
    Ok(())
}

#[test]
fn a_metric_with_only_blank_tokens_is_refused() -> TestResult {
    let (_, line, message) = refused(indoc! {"
        extends: metric
        message: m
        tokens: ['  ']
        formula: occurrences
        condition: '> 3'
    "})?;
    assert_eq!(line, Some(1));
    assert!(message.contains("`tokens` and `raw`"), "{message}");
    Ok(())
}

#[test]
fn a_metric_with_an_empty_raw_is_refused() -> TestResult {
    let (_, line, message) = refused(indoc! {"
        extends: metric
        message: m
        raw: ['']
        formula: occurrences
        condition: '> 3'
    "})?;
    assert_eq!(line, Some(1));
    assert!(message.contains("`tokens` and `raw`"), "{message}");
    Ok(())
}

#[test]
fn a_rule_with_only_raw_loads() -> TestResult {
    load(indoc! {r"
        extends: existence
        message: m
        raw: ['\bvery\b']
    "})?;
    Ok(())
}

#[test]
fn a_doc_scope_is_refused() -> TestResult {
    let (field, _, message) = refused(indoc! {"
        extends: existence
        message: m
        scope: doc(heading)
        tokens: [a]
    "})?;
    assert_eq!(field, "scope");
    assert!(message.contains("doc("), "{message}");
    Ok(())
}

#[test]
fn a_swap_loads_longest_key_first() -> TestResult {
    let rule = load(indoc! {"
        extends: substitution
        message: m
        swap:
          order: sequence
          in order to: to
    "})?;
    let Check::Substitution(substitution) = rule.check else {
        return Err("not a substitution rule".into());
    };
    let keys: Vec<_> = substitution
        .swaps
        .iter()
        .map(|swap| swap.key.as_str())
        .collect();
    assert_eq!(keys, ["in order to", "order"]);
    Ok(())
}

#[test]
fn a_metric_counts_tokens() -> TestResult {
    let rule = load(indoc! {"
        extends: metric
        message: m
        nonword: true
        tokens: [';']
        formula: occurrences / words
        condition: '> 3'
    "})?;
    let Check::Metric(metric) = rule.check else {
        return Err("not a metric rule".into());
    };
    assert!(metric.tokens.is_some());
    Ok(())
}

#[test]
fn a_scope_hakkiri_does_not_define() -> TestResult {
    let error = load(indoc! {"
        extends: existence
        message: m
        scope: chapter
        tokens: [a]
    "})
    .err()
    .ok_or("the rule loaded")?;
    let message = error.to_string();
    assert!(message.contains("Team.Rule"), "{message}");
    assert!(message.contains("chapter"), "{message}");
    Ok(())
}
