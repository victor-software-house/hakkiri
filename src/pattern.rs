//! Patterns: the `regex` crate, and `fancy-regex` for a pattern that needs
//! lookaround or a backreference.

use std::ops::Range;
use std::{error, fmt};

/// How many steps `fancy-regex` takes on one search before it gives up.
pub const BACKTRACK_LIMIT: usize = 1_000_000;

/// The largest compiled pattern, in bytes. It sits above the default of the
/// `regex` crate, since a rule with hundreds of swaps compiles to a big one.
const SIZE_LIMIT: usize = 64 << 20;

/// One match and the groups in it that hold text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Captured {
    /// The bytes of the whole match.
    pub(crate) whole: Range<usize>,
    /// The bytes of each capture group that matched and holds text, in group
    /// order.
    pub(crate) groups: Vec<Range<usize>>,
}

/// A compiled pattern.
#[derive(Debug)]
pub struct Pattern {
    source: String,
    engine: Engine,
}

#[derive(Debug)]
enum Engine {
    Regex(regex::Regex),
    Fancy(fancy_regex::Regex),
}

/// Why a pattern did not compile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PatternError {
    /// The pattern as written.
    pub pattern: String,
    /// What the engine says is wrong with it.
    pub message: String,
}

impl fmt::Display for PatternError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "the pattern `{}` does not compile: {}",
            self.pattern, self.message
        )
    }
}

impl error::Error for PatternError {}

/// Why a search of a text did not finish.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SearchError {
    /// `fancy-regex` took [`BACKTRACK_LIMIT`] steps on one search.
    BacktrackLimit {
        /// The pattern as compiled.
        pattern: String,
    },
    /// `fancy-regex` ran out of stack on the text.
    Stack {
        /// The pattern as compiled.
        pattern: String,
    },
}

impl fmt::Display for SearchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BacktrackLimit { pattern } => write!(
                f,
                "the pattern `{pattern}` reached the backtrack limit of {BACKTRACK_LIMIT} steps"
            ),
            Self::Stack { pattern } => {
                write!(f, "the pattern `{pattern}` ran out of stack on the text")
            }
        }
    }
}

impl error::Error for SearchError {}

impl SearchError {
    fn new(pattern: &str, error: &fancy_regex::Error) -> Self {
        let pattern = pattern.to_owned();
        if matches!(
            error,
            fancy_regex::Error::RuntimeError(fancy_regex::RuntimeError::StackOverflow)
        ) {
            Self::Stack { pattern }
        } else {
            Self::BacktrackLimit { pattern }
        }
    }
}

/// The whole match and the groups with text, from the groups of one match,
/// the whole match first.
fn captured<M: Into<Range<usize>>>(
    mut groups: impl Iterator<Item = Option<M>>,
) -> Option<Captured> {
    let whole = groups.next()??.into();
    Some(Captured {
        whole,
        groups: groups
            .flatten()
            .map(Into::into)
            .filter(|range: &Range<usize>| !range.is_empty())
            .collect(),
    })
}

impl Pattern {
    /// Compiles `source` with `regex`, and with `fancy-regex` when `regex`
    /// refuses it for a feature only a backtracking engine has.
    ///
    /// # Errors
    ///
    /// Returns a [`PatternError`] when neither engine compiles `source`.
    pub fn new(source: &str) -> Result<Self, PatternError> {
        let engine = match regex::RegexBuilder::new(source)
            .size_limit(SIZE_LIMIT)
            .build()
        {
            Ok(regex) => Engine::Regex(regex),
            Err(refused) => {
                let message = refused.to_string();
                let backtracking =
                    message.contains("look-around") || message.contains("backreference");
                match fancy_regex::RegexBuilder::new(source)
                    .backtrack_limit(BACKTRACK_LIMIT)
                    .build()
                {
                    Ok(regex) => Engine::Fancy(regex),
                    Err(error) => {
                        return Err(PatternError {
                            pattern: source.to_owned(),
                            message: reason(&if backtracking {
                                error.to_string()
                            } else {
                                message
                            }),
                        });
                    }
                }
            }
        };
        Ok(Self {
            source: source.to_owned(),
            engine,
        })
    }

    /// Whether the pattern matches anywhere in `text`.
    ///
    /// # Errors
    ///
    /// Returns a [`SearchError`] when `fancy-regex` gives up on `text`.
    pub fn is_match(&self, text: &str) -> Result<bool, SearchError> {
        match &self.engine {
            Engine::Regex(regex) => Ok(regex.is_match(text)),
            Engine::Fancy(regex) => regex
                .is_match(text)
                .map_err(|error| SearchError::new(&self.source, &error)),
        }
    }

    /// The byte range of each match in `text`, in order.
    ///
    /// # Errors
    ///
    /// Returns a [`SearchError`] when `fancy-regex` gives up on `text`.
    pub fn find_all(&self, text: &str) -> Result<Vec<Range<usize>>, SearchError> {
        match &self.engine {
            Engine::Regex(regex) => Ok(regex.find_iter(text).map(|found| found.range()).collect()),
            Engine::Fancy(regex) => regex
                .find_iter(text)
                .map(|found| found.map(|found| found.range()))
                .collect::<Result<_, _>>()
                .map_err(|error| SearchError::new(&self.source, &error)),
        }
    }

    /// The byte range of each match in `text`, including a match that starts
    /// inside an earlier one.
    ///
    /// After each match, the search starts again one character past the
    /// match's start.
    ///
    /// # Errors
    ///
    /// Returns a [`SearchError`] when `fancy-regex` gives up on `text`.
    pub fn find_overlapping(&self, text: &str) -> Result<Vec<Range<usize>>, SearchError> {
        let mut found = Vec::new();
        let mut at = 0;
        while at <= text.len() {
            let next = match &self.engine {
                Engine::Regex(regex) => regex.find_at(text, at).map(|found| found.range()),
                Engine::Fancy(regex) => regex
                    .find_from_pos(text, at)
                    .map_err(|error| SearchError::new(&self.source, &error))?
                    .map(|found| found.range()),
            };
            let Some(range) = next else { break };
            at = range.start
                + text
                    .get(range.start..)
                    .and_then(|rest| rest.chars().next())
                    .map_or(1, char::len_utf8);
            found.push(range);
        }
        Ok(found)
    }

    /// Each match as the number in the name of its first group that starts
    /// with `prefix` and sits in the match, and the byte range of that group.
    ///
    /// A group named `swap_3` with the prefix `swap_` has the number 3.
    ///
    /// # Errors
    ///
    /// Returns a [`SearchError`] when `fancy-regex` gives up on `text`.
    pub fn find_groups(
        &self,
        text: &str,
        prefix: &str,
    ) -> Result<Vec<(usize, Range<usize>)>, SearchError> {
        let numbered = |names: Vec<Option<&str>>| -> Vec<(usize, usize)> {
            names
                .into_iter()
                .enumerate()
                .filter_map(|(group, name)| {
                    let number = name?.strip_prefix(prefix)?.parse().ok()?;
                    Some((group, number))
                })
                .collect()
        };
        let mut found = Vec::new();
        match &self.engine {
            Engine::Regex(regex) => {
                let groups = numbered(regex.capture_names().collect());
                for captures in regex.captures_iter(text) {
                    found.extend(groups.iter().find_map(|&(group, number)| {
                        captures.get(group).map(|hit| (number, hit.range()))
                    }));
                }
            }
            Engine::Fancy(regex) => {
                let groups = numbered(regex.capture_names().collect());
                for captures in regex.captures_iter(text) {
                    let captures =
                        captures.map_err(|error| SearchError::new(&self.source, &error))?;
                    found.extend(groups.iter().find_map(|&(group, number)| {
                        captures.get(group).map(|hit| (number, hit.range()))
                    }));
                }
            }
        }
        Ok(found)
    }

    /// Each match with the groups in it that hold text.
    ///
    /// # Errors
    ///
    /// Returns a [`SearchError`] when `fancy-regex` gives up on `text`.
    pub(crate) fn find_captured(&self, text: &str) -> Result<Vec<Captured>, SearchError> {
        let mut found = Vec::new();
        match &self.engine {
            Engine::Regex(regex) => {
                for captures in regex.captures_iter(text) {
                    found.extend(captured(captures.iter()));
                }
            }
            Engine::Fancy(regex) => {
                for captures in regex.captures_iter(text) {
                    let captures =
                        captures.map_err(|error| SearchError::new(&self.source, &error))?;
                    found.extend(captured(captures.iter()));
                }
            }
        }
        Ok(found)
    }

    /// `text` with every match replaced by `replacement`, where `$1` reads as
    /// the text of the first group.
    ///
    /// # Errors
    ///
    /// Returns a [`SearchError`] when `fancy-regex` gives up on `text`.
    pub fn replace_all(&self, text: &str, replacement: &str) -> Result<String, SearchError> {
        match &self.engine {
            Engine::Regex(regex) => Ok(regex.replace_all(text, replacement).into_owned()),
            Engine::Fancy(regex) => regex
                .try_replacen(text, 0, replacement)
                .map(std::borrow::Cow::into_owned)
                .map_err(|error| SearchError::new(&self.source, &error)),
        }
    }

    /// The pattern as it was compiled.
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Whether `fancy-regex` runs the pattern, because `regex` refused it.
    #[must_use]
    pub const fn backtracks(&self) -> bool {
        matches!(self.engine, Engine::Fancy(_))
    }
}

/// The reason in an engine's message: the text after its last `error:` line,
/// or the whole message.
fn reason(message: &str) -> String {
    message
        .rsplit_once("error: ")
        .map_or(message, |(_, reason)| reason)
        .trim()
        .replace('\n', " ")
}
