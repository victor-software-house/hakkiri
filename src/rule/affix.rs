//! The `affix` field of an `existence` or `substitution` rule: each
//! `word/FLAGS` part of a token or a key becomes the alternation of the forms
//! its flags name.
//!
//! A port of `affix.go` in [Vale]'s `check` package at `66bbb85`.
//!
//! [Vale]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/affix.go

use super::LoadError;
use super::build::Source;
use super::package::{Forms, forms};

/// The characters a regular expression gives a meaning, as Go's
/// `regexp.QuoteMeta` reads them.
const META: &str = r"\.+*?()|[]{}^$";

/// `text` with each character of [`META`] escaped.
fn quote(text: &str) -> String {
    let mut quoted = String::with_capacity(text.len());
    for c in text.chars() {
        if META.contains(c) {
            quoted.push('\\');
        }
        quoted.push(c);
    }
    quoted
}

/// The byte of the last `/` of `part` that no backslash comes before.
fn unescaped_slash(part: &str) -> Option<usize> {
    let bytes = part.as_bytes();
    bytes.iter().enumerate().rev().find_map(|(at, byte)| {
        let escaped = at
            .checked_sub(1)
            .and_then(|before| bytes.get(before))
            .is_some_and(|before| *before == b'\\');
        (*byte == b'/' && !escaped).then_some(at)
    })
}

/// `token` with each space-separated `word/FLAGS` part replaced by the
/// alternation of its forms. Another part, and a token with no unescaped
/// slash, stay as written.
fn token(forms: &Forms, token: &str) -> Result<String, String> {
    let mut parts: Vec<String> = Vec::new();
    for written in token.split(' ') {
        if unescaped_slash(written).is_none() {
            parts.push(written.to_owned());
            continue;
        }
        // A word boundary on either side stays outside the alternation.
        let (before, rest) = written
            .strip_prefix(r"\b")
            .map_or(("", written), |rest| (r"\b", rest));
        let (after, part) = rest
            .strip_suffix(r"\b")
            .map_or(("", rest), |part| (r"\b", part));
        let Some(cut) = unescaped_slash(part) else {
            parts.push(written.to_owned());
            continue;
        };
        let (word, flags) = part.split_at(cut);
        let word = word.replace(r"\/", "/");
        if quote(&word) != word {
            return Err(format!(
                "'{part}' has affix flags, so its word must be plain text; write a literal slash as '\\/'"
            ));
        }
        let flags = flags.strip_prefix('/').unwrap_or(flags);
        let names = forms
            .of(&word, flags)
            .map_err(|error| format!("'{part}': {error}; write a literal slash as '\\/'"))?;
        let alternation: Vec<String> = names.iter().map(|form| quote(form)).collect();
        parts.push(format!("{before}(?:{}){after}", alternation.join("|")));
    }
    Ok(parts.join(" "))
}

/// `tokens`, the values of `field`, with the `word/FLAGS` parts expanded under
/// the affix file `affix`. A rule with no `affix` keeps its tokens as written.
pub(super) fn expand(
    source: &Source<'_>,
    field: &str,
    affix: &str,
    tokens: &[String],
) -> Result<Vec<String>, LoadError> {
    if affix.is_empty() {
        return Ok(tokens.to_vec());
    }
    let affixes = forms(source, affix)?;
    tokens
        .iter()
        .map(|written| {
            token(&affixes, written).map_err(|message| {
                source.problem(
                    field,
                    format!("rule `{}`: `{written}`: {message}", source.name),
                )
            })
        })
        .collect()
}
