//! The Treebank word tokenizer, which cuts a title before it is tagged.
//!
//! A port of `tokenize/treebank.go` in [prose] at `b33e60d`, itself a port of
//! a sed script of the Penn Treebank project. The sentence tokenizer of this
//! crate is not this one: [`super::tokens`] ports `tokenize.New()`, which tags
//! the words of a sentence, and the title converter of prose cuts its input
//! with this one.
//!
//! Go reads each map of replacements in an order that changes between runs.
//! The replacements of a map touch different text, so this reads them in the
//! order the file lists them.
//!
//! [prose]: https://github.com/jdkato/prose/blob/b33e60dbff0492289b227f3681120ac057d8131b/tokenize/treebank.go

use std::sync::LazyLock;

use regex::Regex;

/// A pattern and the text that replaces each match of it.
struct Step {
    pattern: Regex,
    with: &'static str,
}

/// The steps in the order they run, apart from the two that wrap the text in
/// spaces between them.
struct Steps {
    /// From the starting quotes up to the brackets.
    before: Vec<Step>,
    /// From the ending quotes up to the contractions.
    after: Vec<Step>,
    newlines: Regex,
    spaces: Regex,
}

#[expect(
    clippy::expect_used,
    reason = "The patterns are constants that compile."
)]
fn steps(list: &[(&str, &'static str)]) -> Vec<Step> {
    list.iter()
        .map(|(pattern, with)| Step {
            pattern: Regex::new(pattern).expect("a Treebank pattern compiles"),
            with,
        })
        .collect()
}

#[expect(
    clippy::expect_used,
    reason = "The patterns are constants that compile."
)]
static STEPS: LazyLock<Steps> = LazyLock::new(|| Steps {
    before: steps(&[
        (r#"'([ (\[{<])""#, "${1} `` "),
        (r#"^(")"#, "``"),
        (r#"( ")"#, " ``"),
        ("(``)", " ${1} "),
        (r"([:,])([^0-9])", " ${1} ${2}"),
        (r"\.\.\.", " ... "),
        (r#"([^.])(\.)([\]\)}>"']*)[\t\n\f\r ]*$"#, "${1} ${2}${3} "),
        ("([^'])' ", "${1} ' "),
        (r"([:,])$", " ${1} "),
        (r"([;@#$%&?!])", " ${1} "),
        (r"([\]\[(){}<>])", " ${1} "),
        ("--", " -- "),
    ]),
    after: steps(&[
        (r#"""#, " '' "),
        (r"'([^\t\n\f\r ])('')'", "${1} ${2} "),
        (r"([^' ])('[sS]|'[mM]|'[dD]|') ", "${1} ${2} "),
        (r"([^' ])('ll|'LL|'re|'RE|'ve|'VE|n't|N'T) ", "${1} ${2} "),
        (r"(?i)(?-u:\b)(can)(not)(?-u:\b)", " ${1} ${2} "),
        (r"(?i)(?-u:\b)(d)('ye)(?-u:\b)", " ${1} ${2} "),
        (r"(?i)(?-u:\b)(gim)(me)(?-u:\b)", " ${1} ${2} "),
        (r"(?i)(?-u:\b)(gon)(na)(?-u:\b)", " ${1} ${2} "),
        (r"(?i)(?-u:\b)(got)(ta)(?-u:\b)", " ${1} ${2} "),
        (r"(?i)(?-u:\b)(lem)(me)(?-u:\b)", " ${1} ${2} "),
        (r"(?i)(?-u:\b)(mor)('n)(?-u:\b)", " ${1} ${2} "),
        (r"(?i)(?-u:\b)(wan)(na) ", " ${1} ${2} "),
        (r"(?i) ('t)(is)(?-u:\b)", " ${1} ${2} "),
        (r"(?i) ('t)(was)(?-u:\b)", " ${1} ${2} "),
    ]),
    newlines: Regex::new(r"[\n\r]").expect("the newline pattern compiles"),
    spaces: Regex::new(" {2,}").expect("the space pattern compiles"),
});

/// The words of `sentence`, which the tokenizer expects to be one sentence.
pub(super) fn tokenize(sentence: &str) -> Vec<String> {
    let steps = &*STEPS;
    let mut text = sentence.to_owned();
    for step in &steps.before {
        text = step.pattern.replace_all(&text, step.with).into_owned();
    }
    text = format!(" {text} ");
    for step in &steps.after {
        text = step.pattern.replace_all(&text, step.with).into_owned();
    }
    let text = steps.newlines.replace_all(&text, " ");
    let text = steps.spaces.replace_all(&text, " ");
    text.trim().split(' ').map(str::to_owned).collect()
}
