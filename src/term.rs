//! A vocabulary term shown in the case of the text it replaced.
//!
//! A port of `recaseToTerm` and its helpers in `substitution.go` of [Vale] at
//! `66bbb85`. A term that is a pattern, such as `colou?r`, is listed as the
//! spellings it matches. The one that equals the observed text without regard
//! to case is shown.
//!
//! [Vale]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/substitution.go

#[cfg(test)]
mod tests;

/// The most spellings a term may list; past it a term is no longer one word
/// spelled several ways.
const MAX_EXPANSIONS: usize = 64;

/// `term` with the literal text of the pattern lowercased and the rest as
/// written: an escape such as `\W`, `\p{Lu}` or `\x{41}`, and the name of a
/// capture group.
///
/// Vale lowercases the whole pattern, so `\W` becomes `\w` and the term means
/// something else. hakkiri leaves the escapes alone.
pub(crate) fn lowercase_literals(term: &str) -> String {
    let mut out = String::with_capacity(term.len());
    let mut chars = term.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                out.push(c);
                let Some(escaped) = chars.next() else { break };
                out.push(escaped);
                if matches!(escaped, 'p' | 'P' | 'x' | 'u' | 'U') && chars.peek() == Some(&'{') {
                    for inner in chars.by_ref() {
                        out.push(inner);
                        if inner == '}' {
                            break;
                        }
                    }
                }
            }
            '(' => {
                out.push(c);
                // `(?P<Name>` and `(?<Name>` name a group.
                let rest: String = chars.clone().take(4).collect();
                let skip = if rest.starts_with("?P<") {
                    3
                } else if rest.starts_with("?<")
                    && !rest.starts_with("?<=")
                    && !rest.starts_with("?<!")
                {
                    2
                } else {
                    0
                };
                if skip > 0 {
                    for inner in chars.by_ref().take(skip) {
                        out.push(inner);
                    }
                    for inner in chars.by_ref() {
                        out.push(inner);
                        if inner == '>' {
                            break;
                        }
                    }
                }
            }
            _ => out.extend(c.to_lowercase()),
        }
    }
    out
}

/// `term` in the spelling that equals `observed` without regard to case, when
/// one of the spellings the term lists does, and `term` itself otherwise.
///
/// Runs of white space in `observed` count as one space.
pub(crate) fn recase_to_term(term: &str, observed: &str) -> String {
    let unwrapped = observed.split_whitespace().collect::<Vec<_>>().join(" ");
    let folded = unwrapped.to_lowercase();
    spellings(term)
        .into_iter()
        .find(|candidate| candidate.to_lowercase() == folded)
        .unwrap_or_else(|| term.to_owned())
}

/// The spellings of `term`: every one a finite pattern matches, or the literal
/// characters of any other pattern in order, or none.
fn spellings(term: &str) -> Vec<String> {
    if let Some(found) = enumerate(term) {
        return found;
    }
    let skeleton = skeleton(term);
    if skeleton.is_empty() {
        Vec::new()
    } else {
        vec![skeleton]
    }
}

/// Every spelling a pattern can match, such as `OAuth2?` giving `OAuth2` and
/// `OAuth`, or `None` when the pattern is not a finite set.
fn enumerate(pattern: &str) -> Option<Vec<String>> {
    let runes: Vec<char> = pattern.chars().collect();
    let (spellings, next) = alternation(&runes, 0, false)?;
    (next == runes.len() && !spellings.is_empty()).then_some(spellings)
}

/// The spellings of the alternation read from `at`, and where it ends. A
/// nested one ends at its closing bracket.
fn alternation(runes: &[char], mut at: usize, nested: bool) -> Option<(Vec<String>, usize)> {
    let mut branches: Vec<String> = Vec::new();
    let mut current = vec![String::new()];
    while let Some(&rune) = runes.get(at) {
        let mut atom: Vec<String>;
        match rune {
            ')' => {
                if !nested {
                    return None;
                }
                branches.extend(current);
                return Some((branches, at + 1));
            }
            '|' => {
                branches.extend(current);
                current = vec![String::new()];
                at += 1;
                continue;
            }
            '(' => {
                let mut start = at + 1;
                if runes.get(start) == Some(&'?') && runes.get(start + 1) == Some(&':') {
                    start += 2;
                } else if runes.get(start) == Some(&'?') {
                    return None;
                }
                let (inner, next) = alternation(runes, start, true)?;
                atom = inner;
                at = next;
            }
            '\\' => {
                let escaped = *runes.get(at + 1)?;
                if escaped.is_alphabetic() || escaped.is_numeric() {
                    return None;
                }
                atom = vec![escaped.to_string()];
                at += 2;
            }
            '[' | ']' | '{' | '}' | '.' | '*' | '+' | '^' | '$' => return None,
            other => {
                atom = vec![other.to_string()];
                at += 1;
            }
        }
        match runes.get(at) {
            Some('?') => {
                atom.push(String::new());
                at += 1;
            }
            Some('*' | '+' | '{') => return None,
            _ => {}
        }
        if current.len() * atom.len() > MAX_EXPANSIONS {
            return None;
        }
        current = current
            .iter()
            .flat_map(|prefix| atom.iter().map(move |suffix| format!("{prefix}{suffix}")))
            .collect();
    }
    if nested {
        return None;
    }
    branches.extend(current);
    Some((branches, at))
}

/// The literal characters of `pattern`, in order, or nothing when it uses a
/// construct that no single fixed text stands for: a character class, a group,
/// an alternation, a wildcard or a repeat. A `?` after a character keeps it.
fn skeleton(pattern: &str) -> String {
    let runes: Vec<char> = pattern.chars().collect();
    let mut out = String::new();
    let mut at = 0;
    while let Some(&rune) = runes.get(at) {
        match rune {
            '\\' => {
                let Some(&escaped) = runes.get(at + 1) else {
                    return String::new();
                };
                out.push(escaped);
                at += 1;
            }
            '?' => {}
            '[' | ']' | '(' | ')' | '{' | '}' | '|' | '.' | '*' | '+' | '^' | '$' => {
                return String::new();
            }
            other => out.push(other),
        }
        at += 1;
    }
    out
}
