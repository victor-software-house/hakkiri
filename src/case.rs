//! Case conversion one character to one, as Go's `unicode` package does it.
//!
//! Rust's own conversions can give several characters for one, such as `SS`
//! for `ß`, and can read the neighbours of a character, as the final form of
//! `Σ`. Go does neither, and the ported readers match what Go gives.

/// The lower case of `c`, as one character.
///
/// `İ` is mapped to `i` as Go does, and a character with no single-character
/// lower case stays as it is.
pub(crate) fn lower(c: char) -> char {
    if c == '\u{130}' {
        return 'i';
    }
    single(c.to_lowercase(), c)
}

/// The upper case of `c`, as one character, or `c` when it has none.
pub(crate) fn upper(c: char) -> char {
    single(c.to_uppercase(), c)
}

/// The only character of `mapped`, or `fallback` when it holds more or none.
fn single(mut mapped: impl Iterator<Item = char>, fallback: char) -> char {
    match (mapped.next(), mapped.next()) {
        (Some(only), None) => only,
        _ => fallback,
    }
}

/// `text` in lower case, a character at a time.
pub(crate) fn lower_str(text: &str) -> String {
    text.chars().map(lower).collect()
}

/// `text` in upper case, a character at a time.
pub(crate) fn upper_str(text: &str) -> String {
    text.chars().map(upper).collect()
}
