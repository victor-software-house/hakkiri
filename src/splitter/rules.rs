//! Override rules that run after Punkt.
//!
//! They cover the failures the pySBD paper reports for Punkt. The rule that a
//! token with no space inside holds no boundary needs no code: a boundary falls
//! only at the end of a token, and a token ends at whitespace. The rule that a
//! closing quote or bracket stays with its sentence is applied where the
//! sentences are cut, in [`super::Model::split`].

use super::token::{Mark, Token};

/// The brackets of a text that open and close in pairs.
#[derive(Debug)]
pub(super) struct Brackets {
    /// The byte positions of the opening brackets that close later, sorted.
    opens: Vec<usize>,
    /// The byte positions of the closing brackets that match one, sorted.
    closes: Vec<usize>,
}

impl Brackets {
    /// Pairs the brackets in `text`: `(` with `)`, `[` with `]`, `{` with `}`.
    ///
    /// Each closing bracket pairs with the last unpaired opening bracket of
    /// its own kind. A bracket with no partner pairs with nothing.
    pub(super) fn new(text: &str) -> Self {
        let mut round = Vec::new();
        let mut square = Vec::new();
        let mut curly = Vec::new();
        let mut opens = Vec::new();
        let mut closes = Vec::new();
        for (index, c) in text.char_indices() {
            let partner = match c {
                '(' => {
                    round.push(index);
                    None
                }
                '[' => {
                    square.push(index);
                    None
                }
                '{' => {
                    curly.push(index);
                    None
                }
                ')' => round.pop(),
                ']' => square.pop(),
                '}' => curly.pop(),
                _ => None,
            };
            if let Some(open) = partner {
                opens.push(open);
                closes.push(index);
            }
        }
        opens.sort_unstable();
        Self { opens, closes }
    }

    /// Whether a bracket opens before `end` and closes at or after it.
    ///
    /// `end` is the byte position after a token. A boundary there falls inside
    /// the brackets.
    pub(super) fn encloses(&self, end: usize) -> bool {
        let opened = self.opens.partition_point(|&open| open < end);
        let closed = self.closes.partition_point(|&close| close < end);
        opened > closed
    }
}

/// Whether `current`, an ellipsis, is followed by a word in lower case, so the
/// sentence goes on.
pub(super) fn continues_after_ellipsis(current: &Token<'_>, next: &Token<'_>) -> bool {
    current.mark == Mark::Ellipsis && next.first_lower()
}

#[cfg(test)]
mod tests {
    use super::Brackets;

    #[test]
    fn a_boundary_inside_brackets_is_enclosed() {
        let text = "a (b. c) d. e";
        let brackets = Brackets::new(text);
        assert!(brackets.encloses(6));
        assert!(!brackets.encloses(11));
    }

    #[test]
    fn a_bracket_with_no_partner_encloses_nothing() {
        let brackets = Brackets::new("a (b. c d. e");
        assert!(!brackets.encloses(5));
        let brackets = Brackets::new("1) a. 2) b.");
        assert!(!brackets.encloses(6));
    }

    #[test]
    fn nested_and_mixed_brackets_pair_by_kind() {
        let text = "x [a (b. c) d. e] f. g";
        let brackets = Brackets::new(text);
        assert!(brackets.encloses(9));
        assert!(brackets.encloses(15));
        assert!(!brackets.encloses(20));
    }
}
