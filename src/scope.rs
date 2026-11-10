//! A rule's `scope`: which nodes of the tree a rule reads.
//!
//! A scope is a list of items, and a node is read when any item selects it.
//! An item is parts joined by `&`, and every part must hold.
//! A part is a dotted scope name, `~` before it to leave out an element, and a
//! file extension as its last piece to limit it to files of that kind.

use std::{error, fmt};

use crate::tree::Scope as NodeScope;

/// The file extensions a scope part may end in. A rule for a language
/// hakkiri does not read loads and selects nothing.
const EXTENSIONS: [&str; 33] = [
    "adoc", "c", "cc", "cjs", "cpp", "cs", "css", "go", "h", "hpp", "html", "java", "js", "jsx",
    "kt", "lua", "md", "mdx", "mjs", "org", "php", "pl", "py", "r", "rb", "rs", "rst", "sass",
    "scala", "scss", "swift", "ts", "tsx",
];

/// The inline elements a `~` part blanks instead of leaving out.
const INLINE: [&str; 4] = ["link", "strong", "emphasis", "code"];

/// An inline element whose text a scope reads as spaces.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Inline {
    Link,
    Strong,
    Emphasis,
    Code,
}

impl Inline {
    pub(crate) const fn scope(self) -> NodeScope {
        match self {
            Self::Link => NodeScope::Link,
            Self::Strong => NodeScope::Strong,
            Self::Emphasis => NodeScope::Emphasis,
            Self::Code => NodeScope::Code,
        }
    }
}

/// Why a scope item was refused.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScopeError {
    /// The item as written.
    pub scope: String,
    /// What is wrong with it.
    pub message: String,
}

impl fmt::Display for ScopeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "`{}` {}", self.scope, self.message)
    }
}

impl error::Error for ScopeError {}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Part {
    /// The dotted scope name, without its extension.
    name: String,
    negated: bool,
    extension: Option<String>,
}

impl Part {
    /// Whether the part holds in a file with `extension`: it names no
    /// extension, or the file's.
    fn applies_to(&self, extension: &str) -> bool {
        self.extension
            .as_deref()
            .is_none_or(|wanted| wanted == extension)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Alternative {
    parts: Vec<Part>,
}

/// A parsed `scope`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Selector {
    alternatives: Vec<Alternative>,
    blank: Vec<Inline>,
    declared: bool,
}

/// Whether the dotted scope `name` is `part`, or inside it.
///
/// `text` alone is the prose leaves: it does not reach the scopes below it,
/// which are the comments.
fn named(part: &str, name: &str) -> bool {
    if part == "text" {
        return name == "text";
    }
    name.strip_prefix(part)
        .is_some_and(|rest| rest.is_empty() || rest.starts_with('.'))
}

impl Selector {
    /// Parses the items of a rule's `scope`. No item reads as `text`.
    ///
    /// # Errors
    ///
    /// Returns a [`ScopeError`] for an item that names no scope.
    pub fn parse(items: &[String]) -> Result<Self, ScopeError> {
        let declared = !items.is_empty();
        let text = [String::from("text")];
        let items = if declared { items } else { &text };
        let mut alternatives = Vec::new();
        let mut blank = Vec::new();
        for item in items {
            let refuse = |message: &str| ScopeError {
                scope: item.clone(),
                message: message.to_owned(),
            };
            let mut parts = Vec::new();
            for written in item.split('&') {
                let written = written.trim();
                let (negated, written) = written
                    .strip_prefix('~')
                    .map_or((false, written), |rest| (true, rest.trim()));
                let (name, extension) = match written.rsplit_once('.') {
                    Some((name, extension))
                        if EXTENSIONS.contains(&extension) && !name.is_empty() =>
                    {
                        (name, Some(extension.to_owned()))
                    }
                    _ => (written, None),
                };
                let known = matches!(name, "text" | "sentence")
                    || NodeScope::ALL.iter().any(|scope| named(name, scope.name()));
                if !known {
                    return Err(refuse("is not a scope hakkiri defines"));
                }
                let inline = match name {
                    "link" => Some(Inline::Link),
                    "strong" => Some(Inline::Strong),
                    "emphasis" => Some(Inline::Emphasis),
                    "code" => Some(Inline::Code),
                    _ => None,
                };
                if negated && let Some(inline) = inline {
                    blank.push(inline);
                }
                parts.push(Part {
                    name: name.to_owned(),
                    negated,
                    extension,
                });
            }
            alternatives.push(Alternative { parts });
        }
        blank.dedup();
        Ok(Self {
            alternatives,
            blank,
            declared,
        })
    }

    /// Whether the rule wrote a `scope`.
    #[must_use]
    pub const fn is_declared(&self) -> bool {
        self.declared
    }

    /// Whether any item selects sentences.
    pub(crate) fn wants_sentences(&self) -> bool {
        self.alternatives.iter().any(Alternative::wants_sentences)
    }

    /// The inline elements whose text reads as spaces.
    pub(crate) fn blank(&self) -> &[Inline] {
        &self.blank
    }

    /// Whether a block selects.
    ///
    /// `names` are the scope names the block holds, `ancestors` the scopes
    /// above its node, and `extension` the file's extension.
    pub(crate) fn selects(
        &self,
        names: &[&str],
        ancestors: &[NodeScope],
        extension: &str,
        sentence: bool,
    ) -> bool {
        self.alternatives
            .iter()
            .any(|item| item.selects(names, ancestors, extension, sentence))
    }
}

impl Alternative {
    fn wants_sentences(&self) -> bool {
        self.parts
            .iter()
            .any(|part| !part.negated && part.name == "sentence")
    }

    fn selects(
        &self,
        names: &[&str],
        ancestors: &[NodeScope],
        extension: &str,
        sentence: bool,
    ) -> bool {
        // A sentence is read for an item that names sentences, and only that
        // one: an item such as `heading` reads the whole heading.
        if sentence && !self.wants_sentences() {
            return false;
        }
        let mut has_positive = false;
        for part in &self.parts {
            if part.negated {
                continue;
            }
            has_positive = true;
            if part.name == "sentence" {
                if !sentence {
                    return false;
                }
            } else if !names.iter().any(|name| named(&part.name, name)) {
                return false;
            }
            if !part.applies_to(extension) {
                return false;
            }
        }
        if !has_positive && !names.contains(&"text") {
            return false;
        }
        self.parts
            .iter()
            .filter(|part| {
                part.negated && !INLINE.contains(&part.name.as_str()) && part.applies_to(extension)
            })
            .all(|part| {
                !names.iter().any(|name| named(&part.name, name))
                    && !ancestors
                        .iter()
                        .any(|scope| named(&part.name, scope.name()))
            })
    }
}
