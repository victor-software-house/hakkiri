//! Patterns: the `regex` crate, and `fancy-regex` for a pattern that needs
//! lookaround or a backreference.

use std::{error, fmt};

/// How many steps `fancy-regex` takes on one search before it gives up.
pub const BACKTRACK_LIMIT: usize = 1_000_000;

/// The largest compiled pattern, in bytes. It sits above the default of the
/// `regex` crate, since a rule with hundreds of swaps compiles to a big one.
const SIZE_LIMIT: usize = 64 << 20;

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
    /// Returns a [`PatternError`] when `fancy-regex` gives up on `text` at its
    /// backtrack limit.
    pub fn is_match(&self, text: &str) -> Result<bool, PatternError> {
        match &self.engine {
            Engine::Regex(regex) => Ok(regex.is_match(text)),
            Engine::Fancy(regex) => regex.is_match(text).map_err(|error| PatternError {
                pattern: self.source.clone(),
                message: error.to_string(),
            }),
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
