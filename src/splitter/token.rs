//! Words and end marks: the tokens Punkt decides over.
//!
//! A token is one run of non-whitespace characters. Quotes, brackets and
//! Markdown emphasis marks around the run belong to no word, and the end mark
//! at the end of the word is read off as a [`Mark`]. A sentence can end only at
//! the end of a token, so a run with no space inside never holds a boundary.

use std::collections::HashSet;
use std::ops::Range;

/// The type of a token that is a number, whatever its digits.
pub(super) const NUMBER: &str = "##number##";

/// What a word ends with.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Mark {
    /// No end mark.
    None,
    /// One period, which stays part of the word: `dr.`.
    Period,
    /// Two or more periods, or `…`. The word has none of them.
    Ellipsis,
    /// `?` or `!`. The word has none of them.
    Strong,
}

/// One word of a text, with the mark it ends with.
#[derive(Clone, Debug)]
pub(super) struct Token<'a> {
    /// The byte range of the whole run, quotes and brackets included.
    pub(super) span: Range<usize>,
    /// The word: no quotes, brackets or emphasis marks around it, no
    /// `?`, `!` or ellipsis at the end, but a single final period.
    pub(super) word: &'a str,
    /// What the word ends with.
    pub(super) mark: Mark,
    /// A comma, semicolon or colon follows the mark, so the sentence goes on.
    pub(super) internal_after: bool,
    /// A colon follows the word, so a list may follow.
    pub(super) colon_after: bool,
    /// What comes before the token.
    pub(super) gap: Gap,
}

/// What separates a token from the one before it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Gap {
    /// A space, or nothing.
    Space,
    /// One line break.
    Line,
    /// A blank line, or the start of the text.
    Paragraph,
}

/// What the first look at a token finds, before the next word is known.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct Annotation {
    /// The token may end a sentence.
    pub(super) sentence_break: bool,
    /// The token is an abbreviation.
    pub(super) abbreviation: bool,
    /// The token is an ellipsis.
    pub(super) ellipsis: bool,
}

/// Quotes, brackets and Markdown marks that open or close a word.
const fn is_edge(c: char) -> bool {
    matches!(
        c,
        '"' | '\''
            | '“'
            | '”'
            | '‘'
            | '’'
            | '«'
            | '»'
            | '('
            | ')'
            | '['
            | ']'
            | '{'
            | '}'
            | '<'
            | '>'
            | '*'
            | '_'
            | '`'
            | '~'
    )
}

/// Punctuation inside a sentence, after a word.
const fn is_internal(c: char) -> bool {
    matches!(c, ',' | ';' | ':')
}

/// A closing quote or bracket, which stays with the sentence before it.
const fn is_closing(c: char) -> bool {
    matches!(c, '"' | '\'' | '”' | '’' | '»' | ')' | ']' | '}')
}

/// The tokens of `text`, in order.
pub(super) fn tokens(text: &str) -> Vec<Token<'_>> {
    let mut tokens = Vec::new();
    let mut previous_end = 0;
    let mut run_start = None;
    for (index, c) in text.char_indices() {
        match (c.is_whitespace(), run_start) {
            (false, None) => run_start = Some(index),
            (true, Some(start)) => {
                tokens.push(token(text, start..index, previous_end));
                previous_end = index;
                run_start = None;
            }
            (true, None) | (false, Some(_)) => {}
        }
    }
    if let Some(start) = run_start {
        tokens.push(token(text, start..text.len(), previous_end));
    }
    mark_spaced_ellipses(&mut tokens);
    tokens
}

/// Reads each run of two or more lone periods, as in `. . .`, as ellipses.
fn mark_spaced_ellipses(tokens: &mut [Token<'_>]) {
    let lone: Vec<bool> = tokens
        .iter()
        .map(|token| token.word == "." && token.mark == Mark::Period)
        .collect();
    for (index, token) in tokens.iter_mut().enumerate() {
        let before = index
            .checked_sub(1)
            .and_then(|previous| lone.get(previous))
            .copied()
            .unwrap_or(false);
        let after = lone.get(index + 1).copied().unwrap_or(false);
        if lone.get(index).copied().unwrap_or(false) && (before || after) {
            token.word = "";
            token.mark = Mark::Ellipsis;
        }
    }
}

/// The token for the run `span`, after the whitespace that starts at
/// `previous_end`.
fn token(text: &str, span: Range<usize>, previous_end: usize) -> Token<'_> {
    let gap = text.get(previous_end..span.start).unwrap_or_default();
    let breaks = gap.matches('\n').count();
    let first = previous_end == 0;
    let run = text.get(span.clone()).unwrap_or_default();
    let (core, internal_after) = trim(run);
    let (word, mark) = split_mark(core);
    Token {
        span,
        word,
        mark,
        internal_after,
        colon_after: run.trim_end_matches(is_edge).ends_with(':'),
        gap: if first || breaks > 1 {
            Gap::Paragraph
        } else if breaks == 1 {
            Gap::Line
        } else {
            Gap::Space
        },
    }
}

/// `run` without the quotes, brackets and marks around it, and whether a
/// comma, semicolon or colon was among the trailing ones.
fn trim(run: &str) -> (&str, bool) {
    let mut core = run.trim_start_matches(is_edge);
    let mut internal = false;
    loop {
        let without_edge = core.trim_end_matches(is_edge);
        let shorter = without_edge.trim_end_matches(is_internal);
        internal |= shorter.len() != without_edge.len();
        if shorter.len() == core.len() {
            return (core, internal);
        }
        core = shorter;
    }
}

/// The word and the end mark of `core`.
fn split_mark(core: &str) -> (&str, Mark) {
    if let Some(word) = core.strip_suffix('…') {
        return (word.trim_end_matches('…'), Mark::Ellipsis);
    }
    let without_periods = core.trim_end_matches('.');
    match core.len() - without_periods.len() {
        0 => {}
        1 => return (core, Mark::Period),
        _ => return (without_periods, Mark::Ellipsis),
    }
    let without_strong = core.trim_end_matches(['?', '!']);
    if without_strong.len() == core.len() {
        (core, Mark::None)
    } else {
        (without_strong, Mark::Strong)
    }
}

impl Token<'_> {
    /// Whether the token is only closing quotes and brackets.
    pub(super) fn is_closing_only(&self, text: &str) -> bool {
        text.get(self.span.clone())
            .is_some_and(|run| !run.is_empty() && run.chars().all(is_closing))
    }

    /// Whether the word starts with an upper-case letter.
    pub(super) fn first_upper(&self) -> bool {
        self.word.chars().next().is_some_and(char::is_uppercase)
    }

    /// Whether the word starts with a lower-case letter.
    pub(super) fn first_lower(&self) -> bool {
        self.word.chars().next().is_some_and(char::is_lowercase)
    }

    /// Whether the word is one letter and a period, such as an initial.
    pub(super) fn is_initial(&self) -> bool {
        let mut chars = self.word.chars();
        matches!(
            (chars.next(), chars.next(), chars.next()),
            (Some(letter), Some('.'), None) if letter.is_alphabetic()
        )
    }

    /// Whether the word is letters and nothing else.
    pub(super) fn is_alphabetic(&self) -> bool {
        !self.word.is_empty() && self.word.chars().all(char::is_alphabetic)
    }

    /// The word in lower case, with every number as [`NUMBER`], and its
    /// period kept.
    pub(super) fn kind(&self) -> String {
        let lower = self.word.to_lowercase();
        if is_number(&lower) {
            NUMBER.to_owned()
        } else {
            lower
        }
    }

    /// [`Token::kind`] without its final period.
    pub(super) fn kind_without_period(&self) -> String {
        let mut kind = self.kind();
        if kind.len() > 1 && kind.ends_with('.') {
            kind.pop();
        }
        kind
    }

    /// [`Token::kind_without_period`] when the token ends a sentence, and
    /// [`Token::kind`] when it does not.
    pub(super) fn kind_without_sentence_period(&self, sentence_break: bool) -> String {
        if sentence_break {
            self.kind_without_period()
        } else {
            self.kind()
        }
    }

    /// Whether the token numbers a list item: a number with a period, or one
    /// letter with a period.
    pub(super) fn is_list_marker(&self) -> bool {
        self.mark == Mark::Period && (self.is_initial() || self.kind() == NUMBER)
    }

    /// Whether the token is symbols only, such as a bullet.
    pub(super) fn is_symbol(&self) -> bool {
        self.mark == Mark::None
            && !self.internal_after
            && !self.word.is_empty()
            && self.word.chars().all(|c| !c.is_alphanumeric())
    }

    /// Whether the token is a word or a number instead of punctuation.
    ///
    /// A number counts: its type, [`NUMBER`], is made of letters.
    pub(super) fn is_non_punctuation(&self) -> bool {
        self.kind().chars().any(char::is_alphabetic)
    }

    /// What the token is before the next word is known: a sentence end, an
    /// abbreviation, an ellipsis, or none of them.
    pub(super) fn first_look(&self, abbreviations: &HashSet<String>) -> Annotation {
        let mut annotation = Annotation::default();
        match self.mark {
            Mark::None => {}
            Mark::Strong => annotation.sentence_break = true,
            Mark::Ellipsis => annotation.ellipsis = true,
            Mark::Period => {
                if self.word != "." && is_abbreviation(self.word, abbreviations) {
                    annotation.abbreviation = true;
                } else {
                    annotation.sentence_break = true;
                }
            }
        }
        annotation
    }
}

/// Whether `word`, a word that ends in a period, is a known abbreviation, or
/// ends in one after a hyphen.
fn is_abbreviation(word: &str, abbreviations: &HashSet<String>) -> bool {
    let stem = word.strip_suffix('.').unwrap_or(word).to_lowercase();
    abbreviations.contains(&stem)
        || stem
            .rsplit('-')
            .next()
            .is_some_and(|last| abbreviations.contains(last))
}

/// Whether `lower` is a number: an optional minus, an optional leading point
/// or comma, a digit, and then digits, points, commas and hyphens.
fn is_number(lower: &str) -> bool {
    let rest = lower.strip_prefix('-').unwrap_or(lower);
    let rest = rest.strip_prefix(['.', ',']).unwrap_or(rest);
    let mut chars = rest.chars();
    chars.next().is_some_and(|c| c.is_ascii_digit())
        && chars.all(|c| c.is_ascii_digit() || matches!(c, '.' | ',' | '-'))
}

#[cfg(test)]
mod tests {
    use super::{Gap, Mark, tokens};

    fn marks(text: &str) -> Vec<(&str, Mark)> {
        tokens(text)
            .into_iter()
            .map(|token| (token.word, token.mark))
            .collect()
    }

    #[test]
    fn reads_each_end_mark() {
        assert_eq!(
            marks("Stop. Why? Go! Wait... Hm… ok"),
            [
                ("Stop.", Mark::Period),
                ("Why", Mark::Strong),
                ("Go", Mark::Strong),
                ("Wait", Mark::Ellipsis),
                ("Hm", Mark::Ellipsis),
                ("ok", Mark::None),
            ]
        );
    }

    #[test]
    fn drops_quotes_brackets_and_emphasis() {
        assert_eq!(
            marks(r#"He said "Stop." (see **Note.**) done"#),
            [
                ("He", Mark::None),
                ("said", Mark::None),
                ("Stop.", Mark::Period),
                ("see", Mark::None),
                ("Note.", Mark::Period),
                ("done", Mark::None),
            ]
        );
    }

    #[test]
    fn a_comma_after_a_mark_keeps_the_sentence_going() {
        let found = tokens("etc., and more");
        assert_eq!(found[0].word, "etc.");
        assert!(found[0].internal_after);
        assert!(!found[1].internal_after);
    }

    #[test]
    fn counts_line_and_paragraph_starts() {
        let found = tokens("one\ntwo\n\nthree four");
        let gaps: Vec<_> = found.iter().map(|token| token.gap).collect();
        assert_eq!(
            gaps,
            [Gap::Paragraph, Gap::Line, Gap::Paragraph, Gap::Space]
        );
    }

    #[test]
    fn a_token_with_no_space_keeps_its_inner_marks() {
        assert_eq!(marks("v1.2.3 README.md 3.50"), {
            [
                ("v1.2.3", Mark::None),
                ("README.md", Mark::None),
                ("3.50", Mark::None),
            ]
        });
    }
}
