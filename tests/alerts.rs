//! Alerts: the alerts scenarios of the rule-engine change, the `existence`
//! scenarios of its match checks, and the long-input scenarios.
//!
//! Each test loads rules, lints one input, and checks the alerts.

#![expect(
    clippy::panic_in_result_fn,
    reason = "A test returns Result for `?` and fails through an assertion."
)]

use std::error::Error;
use std::path::Path;
use std::time::Instant;

use ctl_core::indoc;
use hakkiri::alert::Alert;
use hakkiri::engine::lint;
use hakkiri::rule::{Level, Rule};
use hakkiri::{Language, parse};

type TestResult = Result<(), Box<dyn Error>>;

/// The alerts of the rules, each a name and a rule file, in the file `file`.
fn alerts(rules: &[(&str, &str)], file: &str, source: &str) -> Result<Vec<Alert>, Box<dyn Error>> {
    let rules = rules
        .iter()
        .map(|(name, text)| Rule::from_text(name, "Team/Rule.yml", text))
        .collect::<Result<Vec<_>, _>>()?;
    let path = Path::new(file);
    let language = Language::from_path(path).ok_or("a file with an unknown extension")?;
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .ok_or("a file with no extension")?;
    Ok(lint(&rules, &parse(language, source)?, source, extension)?)
}

/// The alerts of one rule in a Markdown file.
fn markdown(rule: &str, source: &str) -> Result<Vec<Alert>, Box<dyn Error>> {
    alerts(&[("Team.Rule", rule)], "a.md", source)
}

/// What an alert names about its place: line, first and last column.
fn place(alert: &Alert) -> (usize, usize, usize) {
    (alert.line, alert.first_column, alert.last_column)
}

#[test]
fn a_word_in_a_sentence() -> TestResult {
    let found = alerts(
        &[(
            "Team.Very",
            indoc! {"
                extends: existence
                message: \"Drop '%s'.\"
                level: error
                tokens: [very]
            "},
        )],
        "a.md",
        "It is very good.",
    )?;
    let [alert] = found.as_slice() else {
        return Err(format!("expected one alert: {found:?}").into());
    };
    assert_eq!(alert.rule, "Team.Very");
    assert_eq!(alert.level, Level::Error);
    assert_eq!(alert.message, "Drop 'very'.");
    assert_eq!(place(alert), (1, 7, 10));
    assert_eq!(alert.matched, "very");
    Ok(())
}

#[test]
fn a_rule_sets_a_link_and_a_description() -> TestResult {
    let found = markdown(
        indoc! {"
            extends: existence
            message: m
            description: \"Drop '%s' for a plainer word.\"
            link: https://example.test/very
            tokens: [very]
        "},
        "It is very good.",
    )?;
    let [alert] = found.as_slice() else {
        return Err(format!("expected one alert: {found:?}").into());
    };
    assert_eq!(alert.description, "Drop 'very' for a plainer word.");
    assert_eq!(alert.link, "https://example.test/very");
    Ok(())
}

#[test]
fn a_letter_with_an_accent_before_the_match() -> TestResult {
    let found = markdown(
        "extends: existence\nmessage: m\ntokens: [very]\n",
        "Café very good.",
    )?;
    assert_eq!(found.iter().map(place).collect::<Vec<_>>(), [(1, 6, 9)]);
    Ok(())
}

#[test]
fn an_indented_doc_comment() -> TestResult {
    let found = alerts(
        &[(
            "Team.Rule",
            "extends: existence\nmessage: m\nscope: text.comment\ntokens: [marker]\n",
        )],
        "a.rs",
        "fn a() {}\n    /// A marker.\n",
    )?;
    assert_eq!(found.iter().map(place).collect::<Vec<_>>(), [(2, 11, 16)]);
    Ok(())
}

#[test]
fn a_percent_sign() -> TestResult {
    let found = markdown(
        indoc! {"
            extends: existence
            message: \"'%s' is 100%% filler.\"
            ignorecase: true
            tokens: [basically]
        "},
        "Basically, yes.",
    )?;
    assert_eq!(
        found.iter().map(|a| a.message.as_str()).collect::<Vec<_>>(),
        ["'Basically' is 100% filler."]
    );
    Ok(())
}

#[test]
fn a_match_across_a_line_break() -> TestResult {
    let found = markdown(
        "extends: existence\nmessage: \"'%s'\"\ntokens: ['very good']\n",
        "It is very\ngood.",
    )?;
    let [alert] = found.as_slice() else {
        return Err(format!("expected one alert: {found:?}").into());
    };
    assert_eq!(alert.message, "'very good'");
    assert_eq!(alert.matched, "very\ngood");
    assert_eq!(place(alert), (1, 7, 11));
    Ok(())
}

#[test]
fn two_rules_on_one_line() -> TestResult {
    let flag = |token: &str| format!("extends: existence\nmessage: m\ntokens: [{token}]\n");
    let (b, a) = (flag("good"), flag("very"));
    let found = alerts(
        &[("Team.B", &b), ("Team.A", &a)],
        "a.md",
        "It is very good.",
    )?;
    assert_eq!(
        found
            .iter()
            .map(|alert| (alert.rule.as_str(), alert.first_column))
            .collect::<Vec<_>>(),
        [("Team.A", 7), ("Team.B", 12)]
    );
    Ok(())
}

#[test]
fn two_rules_at_one_place_keep_their_order() -> TestResult {
    let flag = |token: &str| format!("extends: existence\nmessage: m\ntokens: [{token}]\n");
    let (b, a) = (flag("very"), flag("very"));
    let found = alerts(
        &[("Team.B", &b), ("Team.A", &a)],
        "a.md",
        "It is very good.",
    )?;
    assert_eq!(
        found
            .iter()
            .map(|alert| alert.rule.as_str())
            .collect::<Vec<_>>(),
        ["Team.B", "Team.A"]
    );
    Ok(())
}

#[test]
fn a_list_item_inside_a_quote() -> TestResult {
    let found = markdown(
        "extends: existence\nmessage: m\nscope: [list, blockquote]\ntokens: [marker]\n",
        "> - A marker\n",
    )?;
    assert_eq!(found.len(), 1, "{found:?}");
    Ok(())
}

#[test]
fn a_limit_of_one() -> TestResult {
    let found = markdown(
        "extends: existence\nmessage: m\nignorecase: true\ntokens: [very]\nlimit: 1\n",
        "Very good, very fast.",
    )?;
    assert_eq!(found.iter().map(place).collect::<Vec<_>>(), [(1, 1, 4)]);
    Ok(())
}

#[test]
fn nested_repeats_with_a_lookahead() -> TestResult {
    let slow = indoc! {"
        extends: existence
        message: m
        nonword: true
        tokens: ['(a+)+(?=b)c']
    "};
    let other = "extends: existence\nmessage: other\ntokens: [marker]\n";
    let source = format!("{}\n\nA marker.\n", "a".repeat(50_000));
    let found = alerts(
        &[("Team.Slow", slow), ("Team.Other", other)],
        "a.md",
        &source,
    )?;
    let [slow, other] = found.as_slice() else {
        return Err(format!("expected two alerts: {found:?}").into());
    };
    assert_eq!(slow.rule, "Team.Slow");
    assert_eq!(slow.level, Level::Error);
    assert!(slow.message.contains("backtrack limit"), "{}", slow.message);
    assert_eq!(place(slow), (1, 1, 1));
    assert_eq!((other.rule.as_str(), other.line), ("Team.Other", 3));
    Ok(())
}

#[test]
fn a_word_and_a_longer_word() -> TestResult {
    let found = markdown(
        "extends: existence\nmessage: \"'%s'\"\ntokens: [cat]\n",
        "A cat and a category.",
    )?;
    assert_eq!(
        found.iter().map(|a| a.matched.as_str()).collect::<Vec<_>>(),
        ["cat"]
    );
    Ok(())
}

#[test]
fn punctuation_with_nonword() -> TestResult {
    let found = markdown(
        "extends: existence\nmessage: m\nnonword: true\ntokens: ['!']\n",
        "Wow! Great!",
    )?;
    assert_eq!(found.len(), 2, "{found:?}");
    Ok(())
}

#[test]
fn a_word_before_a_token() -> TestResult {
    let found = markdown(
        indoc! {r"
            extends: existence
            message: m
            raw: ['\bvery\s+']
            tokens: [good]
        "},
        "It is very good.",
    )?;
    assert_eq!(
        found.iter().map(|a| a.matched.as_str()).collect::<Vec<_>>(),
        ["very good"]
    );
    Ok(())
}

#[test]
fn an_adverb_list_with_an_exception() -> TestResult {
    let found = markdown(
        indoc! {r"
            extends: existence
            message: m
            tokens: ['\w+ly']
            exceptions: [only]
        "},
        "It only barely works.",
    )?;
    assert_eq!(
        found.iter().map(|a| a.matched.as_str()).collect::<Vec<_>>(),
        ["barely"]
    );
    Ok(())
}

#[test]
fn a_word_inside_an_accepted_phrase() -> TestResult {
    let found = markdown(
        "extends: existence\nmessage: m\ntokens: [culpa]\nexceptions: ['mea culpa']\n",
        "A mea culpa and a culpa.",
    )?;
    assert_eq!(found.iter().map(place).collect::<Vec<_>>(), [(1, 19, 23)]);
    Ok(())
}

#[test]
fn a_word_inside_overlapping_phrases() -> TestResult {
    let found = markdown(
        "extends: existence\nmessage: m\ntokens: [c]\nexceptions: ['a b', 'b c']\n",
        "a b c",
    )?;
    assert_eq!(found, Vec::<Alert>::new());
    Ok(())
}

#[test]
fn a_phrase_that_holds_a_shorter_key() -> TestResult {
    let found = markdown(
        indoc! {"
            extends: substitution
            message: \"Use '%s' instead of '%s'.\"
            ignorecase: true
            swap: {in order to: to, order: sequence}
        "},
        "In order to win.",
    )?;
    assert_eq!(
        found.iter().map(|a| a.message.as_str()).collect::<Vec<_>>(),
        ["Use 'to' instead of 'In order to'."]
    );
    Ok(())
}

#[test]
fn a_name_in_two_spellings() -> TestResult {
    let found = markdown(
        "extends: substitution\nmessage: m\nignorecase: true\nswap: {github: GitHub}\n",
        "Use GitHub and github.",
    )?;
    assert_eq!(
        found.iter().map(|a| a.matched.as_str()).collect::<Vec<_>>(),
        ["github"]
    );
    Ok(())
}

#[test]
fn a_sentence_that_starts_with_the_key() -> TestResult {
    let found = markdown(
        indoc! {"
            extends: substitution
            message: \"Use '%s' instead of '%s'.\"
            capitalize: true
            ignorecase: true
            swap: {utilize: use}
        "},
        "Utilize it.",
    )?;
    assert_eq!(
        found.iter().map(|a| a.message.as_str()).collect::<Vec<_>>(),
        ["Use 'Use' instead of 'Utilize'."]
    );
    Ok(())
}

#[test]
fn a_match_that_starts_with_a_digit_is_not_capitalized() -> TestResult {
    let found = markdown(
        indoc! {"
            extends: substitution
            message: \"Use '%s' instead of '%s'.\"
            capitalize: true
            nonword: true
            swap: {'\\d+st': first}
        "},
        "Take 1st.",
    )?;
    assert_eq!(
        found.iter().map(|a| a.message.as_str()).collect::<Vec<_>>(),
        ["Use 'first' instead of '1st'."]
    );
    Ok(())
}

#[test]
fn a_verb_kept_from_the_match() -> TestResult {
    let found = markdown(
        indoc! {r#"
            extends: substitution
            message: "Use '%s' instead of '%s'."
            swap: {'(\w+) out of': '$1 from'}
        "#},
        "Jump out of it.",
    )?;
    assert_eq!(
        found.iter().map(|a| a.message.as_str()).collect::<Vec<_>>(),
        ["Use 'Jump from' instead of 'Jump out of'."]
    );
    Ok(())
}

#[test]
fn matches_inside_words() -> TestResult {
    let found = markdown(
        "extends: occurrence\nmessage: \"%d endings\"\nmax: 1\ntoken: 'ing\\b'\n",
        "Going and coming.",
    )?;
    assert_eq!(
        found.iter().map(|a| a.message.as_str()).collect::<Vec<_>>(),
        ["2 endings"]
    );
    Ok(())
}

#[test]
fn too_few_matches() -> TestResult {
    let found = markdown(
        "extends: occurrence\nmessage: \"%d uses\"\nscope: paragraph\nmin: 2\ntoken: '\\bthe\\b'\n",
        "A cat sat.",
    )?;
    let [alert] = found.as_slice() else {
        return Err(format!("expected one alert: {found:?}").into());
    };
    assert_eq!(
        (alert.message.as_str(), alert.matched.as_str()),
        ("0 uses", "A")
    );
    assert_eq!(alert.first_column, 1);
    Ok(())
}

#[test]
fn a_code_span_is_no_match() -> TestResult {
    let found = markdown(
        "extends: existence\nmessage: m\nnonword: true\ntokens: ['x']\n",
        "Run `cargo` now.",
    )?;
    assert_eq!(found, Vec::<Alert>::new());
    Ok(())
}

/// The rules the long-input tests run together: one of each match check.
const MARKER_RULES: [(&str, &str); 3] = [
    (
        "Team.Existence",
        "extends: existence\nmessage: m\ntokens: [marker]\n",
    ),
    (
        "Team.Substitution",
        "extends: substitution\nmessage: \"Use '%s' instead of '%s'.\"\nswap: {marker: signpost}\n",
    ),
    (
        "Team.Occurrence",
        "extends: occurrence\nmessage: \"%d markers\"\nmax: 1\nmin: 2\ntoken: marker\n",
    ),
];

/// How many alerts of each rule the alerts hold.
fn per_rule(found: &[Alert]) -> Vec<(&str, usize)> {
    MARKER_RULES
        .iter()
        .map(|(name, _)| {
            (
                *name,
                found.iter().filter(|alert| alert.rule == *name).count(),
            )
        })
        .collect()
}

#[test]
fn a_match_at_the_end_of_a_long_line() -> TestResult {
    let source = format!("{}marker\n", "a ".repeat(24_997));
    let start = Instant::now();
    let found = alerts(&MARKER_RULES, "a.md", &source)?;
    eprintln!("a 50,000-character line: {:?}", start.elapsed());
    assert_eq!(
        per_rule(&found),
        [
            ("Team.Existence", 1),
            ("Team.Substitution", 1),
            ("Team.Occurrence", 1)
        ]
    );
    assert!(
        found
            .iter()
            .all(|alert| place(alert) == (1, 49_995, 50_000))
    );
    Ok(())
}

#[test]
fn sixty_thousand_lines() -> TestResult {
    let source = "A marker.\n".repeat(60_000);
    let start = Instant::now();
    let found = alerts(&MARKER_RULES, "a.md", &source)?;
    eprintln!("a 60,000-line file: {:?}", start.elapsed());
    assert_eq!(
        per_rule(&found),
        [
            ("Team.Existence", 60_000),
            ("Team.Substitution", 60_000),
            ("Team.Occurrence", 1)
        ]
    );
    assert_eq!(found.last().map(|alert| alert.line), Some(60_000));
    Ok(())
}
