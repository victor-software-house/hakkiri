//! The features of one word: what the perceptron scores.
//!
//! A port of the `runes` template of `internal/perceptron/features.go` in
//! [prose] at `b33e60d`, the template the English model was trained with.
//!
//! [prose]: https://github.com/jdkato/prose/blob/b33e60dbff0492289b227f3681120ac057d8131b/internal/perceptron/features.go

/// The context word before the first word of a sentence.
pub(super) const START: &str = "-START-";
/// The context word two before the first word of a sentence.
pub(super) const START2: &str = "-START2-";
/// The context word after the last word of a sentence.
pub(super) const END: &str = "-END-";
/// The context word two after the last word of a sentence.
pub(super) const END2: &str = "-END2-";

/// The lower case of `c`, as a single character.
///
/// Go maps a character to one character. Rust maps `İ` to two, so that one is
/// mapped to `i` as Go does.
fn lower(c: char) -> char {
    if c == '\u{130}' {
        return 'i';
    }
    let mut lowered = c.to_lowercase();
    match (lowered.next(), lowered.next()) {
        (Some(only), None) => only,
        _ => c,
    }
}

/// The form of `word` the perceptron reads around the word it tags.
///
/// A word with a hyphen inside is `!HYPHEN`, a number of four digits is
/// `!YEAR`, any other word that starts with a digit is `!DIGITS`, and every
/// other word is in lower case.
pub(super) fn normalize(word: &str) -> String {
    let Some(first) = word.chars().next() else {
        return String::new();
    };
    if first != '-' && word.contains('-') {
        return "!HYPHEN".to_owned();
    }
    if word.len() == 4 && is_decimal_integer(word) {
        return "!YEAR".to_owned();
    }
    if first.is_ascii_digit() {
        return "!DIGITS".to_owned();
    }
    word.chars().map(lower).collect()
}

/// Whether `word` is an optional sign and then digits.
fn is_decimal_integer(word: &str) -> bool {
    let digits = word.strip_prefix(['+', '-']).unwrap_or(word);
    !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit())
}

/// The shape of `word`: `d` for digits only, `p` for punctuation only, `o` for
/// anything else without letters, `XX`, `xx`, `Xx` and `xX` for letters.
///
/// Go reads the Unicode categories `Lu`, `Ll`, `Nd`, `P` and `S`. This reads
/// the properties Rust has, which differ for a few characters such as `²`.
fn shape(word: &str) -> &'static str {
    let (mut upper, mut lower_count, mut digit, mut punct) = (0_usize, 0_usize, 0_usize, 0_usize);
    for c in word.chars() {
        if c.is_uppercase() {
            upper += 1;
        } else if c.is_lowercase() {
            lower_count += 1;
        } else if c.is_numeric() {
            digit += 1;
        } else if !c.is_alphanumeric() && !c.is_whitespace() && !c.is_control() {
            punct += 1;
        }
    }
    let letters = upper + lower_count;
    if letters == 0 {
        return if digit > 0 {
            "d"
        } else if punct > 0 {
            "p"
        } else {
            "o"
        };
    }
    if lower_count == 0 {
        return "XX";
    }
    if upper == 0 {
        return "xx";
    }
    if word.chars().next().is_some_and(char::is_uppercase) && upper == 1 {
        "Xx"
    } else {
        "xX"
    }
}

/// The last `count` characters of `word`.
fn suffix(word: &str, count: usize) -> &str {
    let skip = word.chars().count().saturating_sub(count);
    word.char_indices()
        .nth(skip)
        .and_then(|(at, _)| word.get(at..))
        .unwrap_or("")
}

/// The first `count` characters of `word`.
fn prefix(word: &str, count: usize) -> &str {
    word.char_indices()
        .nth(count)
        .and_then(|(at, _)| word.get(..at))
        .unwrap_or(word)
}

/// The context word `offset` places from the one at `at`.
fn around(context: &[String], at: usize, offset: isize) -> &str {
    at.checked_add_signed(offset)
        .and_then(|index| context.get(index))
        .map_or("", String::as_str)
}

/// The features of the word at `at` in `context`, which holds every word of
/// the sentence in its normal form and two padding words at each end.
///
/// `word` is the word as written, and `previous` and `before` are the tags of
/// the two words before it.
pub(super) fn build(
    context: &[String],
    at: usize,
    word: &str,
    previous: &str,
    before: &str,
) -> Vec<String> {
    let around = |offset: isize| around(context, at, offset);
    let ending = suffix(word, 3);
    vec![
        "a".to_owned(),
        format!("b{ending}"),
        format!("c{}", prefix(word, 1)),
        format!("d{previous}"),
        format!("e{before}"),
        format!("f{previous} {before}"),
        format!("g{}", around(0)),
        format!("h{previous} {}", around(0)),
        format!("i{}", around(-1)),
        format!("j{}", suffix(around(-1), 3)),
        format!("k{}", around(-2)),
        format!("l{}", around(1)),
        format!("m{}", suffix(around(1), 3)),
        format!("n{}", around(2)),
        format!("o{}", suffix(word, 2)),
        format!("p{}", suffix(word, 4)),
        format!("q{}", prefix(word, 2)),
        format!("r{}", prefix(word, 3)),
        format!("s{}", shape(word)),
        format!("t{previous} {ending}"),
    ]
}
