//! Metrics: the metric-check scenarios of the rule-engine change, and the
//! `metric` scenarios of its rule-files spec.
//!
//! Each test loads one `metric` rule, lints one Markdown input, and checks the
//! alerts.

#![expect(
    clippy::panic_in_result_fn,
    reason = "A test returns Result for `?` and fails through an assertion."
)]

use std::error::Error;

use ctl_core::indoc;
use hakkiri::engine::{lint, lint_accepting};
use hakkiri::rule::Rule;
use hakkiri::{Language, parse};

type TestResult = Result<(), Box<dyn Error>>;

/// The messages of the alerts that `rule` gives for the Markdown `source`.
fn messages(rule: &str, source: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let rule = Rule::from_text("Team.Rule", "Team/Rule.yml", rule)?;
    let tree = parse(Language::Markdown, source)?;
    Ok(lint(&[rule], &tree, source, "md")?
        .into_iter()
        .map(|alert| alert.message)
        .collect())
}

/// A rule that prints the formula's result and holds for any text.
fn printing(formula: &str, source: &str) -> Result<Vec<String>, Box<dyn Error>> {
    messages(
        &format!("extends: metric\nmessage: \"%s\"\nformula: {formula}\ncondition: '> -1000000'\n"),
        source,
    )
}

/// A rule that reports the words of each sentence when there are more than two.
const WORDS_OVER_TWO: &str =
    "extends: metric\nmessage: \"%s\"\nscope: sentence\nformula: words\ncondition: '> 2'\n";

#[test]
fn a_code_span_counts_one_word_in_a_metric() -> TestResult {
    assert_eq!(
        messages(
            WORDS_OVER_TWO,
            "Run `a`, `b`, `c` and `one two three` today.\n"
        )?,
        ["7.00"]
    );
    assert_eq!(
        messages(WORDS_OVER_TWO, "Two words `four word span` more.\n")?,
        ["4.00"]
    );
    Ok(())
}

/// The messages of the alerts that `rule` gives for `source` when a code span
/// counts as `size` words.
fn sized(rule: &str, source: &str, size: u8) -> Result<Vec<String>, Box<dyn Error>> {
    let rule = Rule::from_text("Team.Rule", "Team/Rule.yml", rule)?;
    let tree = parse(Language::Markdown, source)?;
    Ok(lint_accepting(&[rule], &tree, source, "md", &[], size)?
        .into_iter()
        .map(|alert| alert.message)
        .collect())
}

#[test]
fn a_code_span_counts_as_many_words_as_its_size() -> TestResult {
    let first = "Run `a`, `b`, `c` and `one two three` today.\n";
    let second = "Two words `four word span` more.\n";
    // Vale counts no word for a code span: 3.00 and 3.00 at 66bbb85.
    for (size, wanted) in [
        (0, ["3.00", "3.00"]),
        (1, ["7.00", "4.00"]),
        (2, ["11.00", "5.00"]),
    ] {
        assert_eq!(
            [
                sized(WORDS_OVER_TWO, first, size)?,
                sized(WORDS_OVER_TWO, second, size)?
            ]
            .concat(),
            wanted,
            "size {size}"
        );
    }
    Ok(())
}

#[test]
fn a_link_whose_text_is_its_address_counts_no_word() -> TestResult {
    // Vale 66bbb85 counts 2 and 4 for the first two, and 2 for the third.
    assert_eq!(printing("words", "See [a.md](a.md) now.\n")?, ["2.00"]);
    assert_eq!(printing("words", "See [a.md](b.md) now.\n")?, ["4.00"]);
    assert_eq!(
        printing("words", "See [a.md][a] now.\n\n[a]: a.md\n")?,
        ["2.00"]
    );
    Ok(())
}

/// A rule that reports the words of every block of one scope.
fn words_of(scope: &str) -> String {
    format!("extends: metric\nmessage: \"%s\"\nscope: {scope}\nformula: words\ncondition: '> -1'\n")
}

#[test]
fn each_alert_keeps_its_place_where_vale_drops_it() -> TestResult {
    // Each case lists what Vale 66bbb85 gives: the same alerts less the ones
    // that share a line and a column with an earlier alert of the rule.
    let table = "| Head | Other |\n|:--|:--|\n| one | two words |\n";
    // Vale: 1.00.
    assert_eq!(sized(&words_of("table.cell"), table, 0)?, ["1.00", "2.00"]);
    // Vale: 2.00 and 2.00.
    assert_eq!(
        sized(
            &words_of("sentence"),
            "First sentence. `Second` starts with code. Third one.\n",
            0
        )?,
        ["2.00", "3.00", "2.00"]
    );
    // Vale: 1.00, on the line of the definition.
    assert_eq!(
        sized(
            &words_of("paragraph"),
            "See [`a/b`][r].\n\nTwo words here.\n\nThree words go here.\n\n[r]: a/b\n",
            0
        )?,
        ["1.00", "3.00", "4.00"]
    );
    // Vale: 4.00.
    assert_eq!(
        sized(
            &words_of("paragraph"),
            "First a paragraph holds `&amp;lt; marker`\n\nSecond paragraph here reads `&lt; marker`\n",
            0
        )?,
        ["4.00", "4.00"]
    );
    Ok(())
}

#[test]
fn an_autolink_counts_the_words_that_vale_counts() -> TestResult {
    // Vale counts 8 for the first, since the address of an email autolink is
    // not its text, and 4 for the second.
    assert_eq!(
        printing(
            "words",
            "Alan Beale <a@b.example> deserves special thanks.\n"
        )?,
        ["8.00"]
    );
    assert_eq!(
        printing("words", "Visit <https://b.example/a> for special thanks.\n")?,
        ["4.00"]
    );
    Ok(())
}

#[test]
fn the_size_of_a_code_span_reaches_a_metric_of_the_whole_file() -> TestResult {
    let whole = "extends: metric\nmessage: \"%s\"\nformula: words\ncondition: '> 2'\n";
    let text = "Run `a`, `b`, `c` and `one two three` today.\n";
    assert_eq!(sized(whole, text, 0)?, ["3.00"]);
    assert_eq!(sized(whole, text, 1)?, ["7.00"]);
    assert_eq!(sized(whole, text, 2)?, ["11.00"]);
    Ok(())
}

#[test]
fn the_size_of_a_code_span_reaches_a_readability_score() -> TestResult {
    let rule = "extends: readability\nmessage: \"%s\"\ngrade: -100\nmetrics:\n  - Flesch-Kincaid\n";
    let text = "Run `a`, `b`, `c` and `one two three` today.\n";
    let none = sized(rule, text, 0)?;
    let one = sized(rule, text, 1)?;
    assert_ne!(none, one);
    Ok(())
}

#[test]
fn words_per_sentence() -> TestResult {
    let found = messages(
        indoc! {"
            extends: metric
            message: \"%s words per sentence\"
            formula: words / sentences
            condition: '>= 4'
        "},
        "One two three four five six. Seven eight.",
    )?;
    assert_eq!(found, ["4.00 words per sentence"]);
    Ok(())
}

#[test]
fn a_contraction_and_initials() -> TestResult {
    assert_eq!(printing("words", "Don't stop the U.S. team.")?, ["5.00"]);
    Ok(())
}

#[test]
fn characters_are_letters_and_digits() -> TestResult {
    assert_eq!(printing("characters", "Ab, 12 c.")?, ["5.00"]);
    Ok(())
}

#[test]
fn a_short_sentence() -> TestResult {
    let found = messages(
        "extends: metric\nmessage: \"%s\"\nformula: flesch_kincaid\ncondition: '< 0'\n",
        "The cat sat on the mat.",
    )?;
    assert_eq!(found, ["-1.45"]);
    Ok(())
}

#[test]
fn second_level_headings() -> TestResult {
    let source = "## One\n\nA.\n\n## Two\n\nB.\n\n## Three\n\nC.\n";
    let found = messages(
        "extends: metric\nmessage: \"%s\"\nformula: heading.h2\ncondition: '> 1'\n",
        source,
    )?;
    assert_eq!(found, ["3.00"]);
    Ok(())
}

#[test]
fn a_heading_count_in_a_condition() -> TestResult {
    let source = "## One\n\nA.\n\n## Two\n\nB.\n";
    let rule = |condition: &str| {
        format!("extends: metric\nmessage: \"%s\"\nformula: paragraphs\ncondition: '{condition}'\n")
    };
    assert_eq!(messages(&rule(">= heading.h2"), source)?, ["2.00"]);
    assert_eq!(
        messages(&rule("> heading.h2"), source)?,
        Vec::<String>::new()
    );
    Ok(())
}

#[test]
fn the_comments_of_a_code_file_are_measured() -> TestResult {
    let rule = Rule::from_text(
        "Team.Rule",
        "Team/Rule.yml",
        "extends: metric\nmessage: \"%s\"\nformula: words\ncondition: '> 1'\n",
    )?;
    let source = "// One two three.\n// Four five.\nfn main() {}\n";
    let tree = parse(Language::Rust, source)?;
    let found = lint(&[rule], &tree, source, "rs")?;
    assert_eq!(
        found
            .iter()
            .map(|a| (a.message.as_str(), a.line))
            .collect::<Vec<_>>(),
        [("5.00", 1)]
    );
    Ok(())
}

#[test]
fn an_element_count_is_zero_with_a_scope() -> TestResult {
    let found = messages(
        "extends: metric\nmessage: \"%s\"\nscope: paragraph\nformula: heading.h2 + paragraphs\ncondition: '>= 0'\n",
        "## One\n\nA b.\n",
    )?;
    assert_eq!(found, ["0.00"]);
    Ok(())
}

#[test]
fn a_heading_is_not_measured() -> TestResult {
    let rule = "extends: metric\nmessage: \"%s\"\nformula: words\ncondition: '>= 0'\n";
    assert_eq!(messages(rule, "# One two three\n\nFour five.\n")?, ["2.00"]);
    assert_eq!(messages(rule, "# One two three\n")?, Vec::<String>::new());
    Ok(())
}

#[test]
fn a_two_sided_condition() -> TestResult {
    let found = messages(
        indoc! {"
            extends: metric
            message: \"%s\"
            formula: round(words / 3, 1)
            condition: '> 0.5 && result < 1'
        "},
        "One two.",
    )?;
    assert_eq!(found, ["0.70"]);
    Ok(())
}

#[test]
fn a_scope_measures_each_block() -> TestResult {
    let found = messages(
        "extends: metric\nmessage: \"%s\"\nscope: paragraph\nformula: words\ncondition: '> 1'\n",
        "One two three.\n\nFour.\n\nFive six.\n",
    )?;
    assert_eq!(found, ["3.00", "2.00"]);
    Ok(())
}

#[test]
fn a_formula_cut_short() {
    let error = Rule::from_text(
        "Team.Rate",
        "Team/Rate.yml",
        "extends: metric\nmessage: m\nformula: words /\ncondition: '> 1'\n",
    )
    .err()
    .map(|error| error.to_string());
    assert!(error.is_some_and(|error| error.contains("Team.Rate") && error.contains("formula")),);
}

#[test]
fn an_unknown_variable_is_refused() {
    let error = Rule::from_text(
        "Team.Rate",
        "Team/Rate.yml",
        "extends: metric\nmessage: m\nformula: wordz\ncondition: '> 1'\n",
    )
    .err()
    .map(|error| error.to_string());
    assert!(error.is_some_and(|error| error.contains("wordz") && error.contains("formula")),);
}

#[test]
fn a_semicolon_rate() -> TestResult {
    let rule = indoc! {"
        extends: metric
        message: \"%s semicolons per 1,000 words, over 3: split the sentences.\"
        nonword: true
        tokens: [';']
        formula: occurrences / words * 1000
        condition: '> 3'
    "};
    let found = messages(rule, "One; two; three four five six seven eight nine ten.")?;
    assert_eq!(
        found,
        ["200.00 semicolons per 1,000 words, over 3: split the sentences."]
    );
    Ok(())
}

#[test]
fn a_semicolon_with_word_boundaries() -> TestResult {
    let rule = indoc! {"
        extends: metric
        message: m
        tokens: [';']
        formula: occurrences / words * 1000
        condition: '> 3'
    "};
    assert_eq!(
        messages(rule, "One; two; three four five six seven eight nine ten.")?,
        Vec::<String>::new()
    );
    Ok(())
}
