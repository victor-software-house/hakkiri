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

/// An inline element whose text a scope reads as spaces.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Inline {
    Link,
    Strong,
    Emphasis,
    Code,
}

impl Inline {
    /// The inline element a `~` part of this name blanks instead of leaving
    /// out.
    fn of(name: &str) -> Option<Self> {
        match name {
            "link" => Some(Self::Link),
            "strong" => Some(Self::Strong),
            "emphasis" => Some(Self::Emphasis),
            "code" => Some(Self::Code),
            _ => None,
        }
    }

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
    // Vale's `table` is the cells, and has no scope for a row.
    if part == "table" && name == "table.row" {
        return false;
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
                // Vale reads `sentence.list` as `sentence & list`.
                let named_parts: Vec<&str> = match name.strip_prefix("sentence.") {
                    Some(rest) if !negated => vec!["sentence", rest],
                    _ => vec![name],
                };
                for (index, name) in named_parts.iter().enumerate() {
                    // A key of the front matter completes `frontmatter` and
                    // `text.frontmatter`, so any name below them is one.
                    let front = |prefix: &str| {
                        name.strip_prefix(prefix)
                            .is_some_and(|rest| rest.starts_with('.'))
                    };
                    let known = matches!(*name, "text" | "sentence" | "text.frontmatter")
                        || front("frontmatter")
                        || front("text.frontmatter")
                        || NodeScope::ALL.iter().any(|scope| named(name, scope.name()));
                    if !known {
                        return Err(refuse("is not a scope hakkiri defines"));
                    }
                    parts.push(Part {
                        name: (*name).to_owned(),
                        negated,
                        extension: (index + 1 == named_parts.len())
                            .then(|| extension.clone())
                            .flatten(),
                    });
                }
            }
            alternatives.push(Alternative { parts });
        }
        Ok(Self {
            alternatives,
            declared,
        })
    }

    /// Whether the rule wrote a `scope`.
    #[must_use]
    pub const fn is_declared(&self) -> bool {
        self.declared
    }

    /// Whether the scope is none, or `text` alone: a `metric` rule then
    /// measures the prose of the whole file instead of the blocks it selects.
    pub(crate) fn measures_file(&self) -> bool {
        match self.alternatives.as_slice() {
            [item] => matches!(
                item.parts.as_slice(),
                [part] if part.name == "text" && !part.negated && part.extension.is_none()
            ),
            _ => false,
        }
    }

    /// Whether any item selects sentences.
    pub(crate) fn wants_sentences(&self) -> bool {
        self.alternatives.iter().any(Alternative::wants_sentences)
    }

    /// What a block reads, or `None` when no item selects it.
    ///
    /// `names` are the scope names the block holds, and `extension` the file's
    /// extension.
    /// `around` are the names of the element around the block that a part may
    /// name beside the block's own, as `link` does in `title & link`: an item
    /// still needs a part that names the block itself.
    /// The block reads as spaces the inline elements that the selecting items
    /// leave out, such as `~link`; an item that does not select the block
    /// leaves out nothing.
    pub(crate) fn reads(
        &self,
        names: &[&str],
        around: &[&str],
        extension: &str,
        sentence: bool,
    ) -> Option<Vec<Inline>> {
        let mut blank = Vec::new();
        let mut selected = false;
        for item in &self.alternatives {
            if item.selects(names, around, extension, sentence) {
                selected = true;
                for inline in item.blank(extension) {
                    if !blank.contains(&inline) {
                        blank.push(inline);
                    }
                }
            }
        }
        selected.then_some(blank)
    }
}

impl Alternative {
    /// The inline elements the item leaves out in a file with `extension`.
    fn blank(&self, extension: &str) -> impl Iterator<Item = Inline> {
        self.parts
            .iter()
            .filter(move |part| part.negated && part.applies_to(extension))
            .filter_map(|part| Inline::of(&part.name))
    }

    fn wants_sentences(&self) -> bool {
        self.parts
            .iter()
            .any(|part| !part.negated && part.name == "sentence")
    }

    fn selects(&self, names: &[&str], around: &[&str], extension: &str, sentence: bool) -> bool {
        // A sentence is read for an item that names sentences, and only that
        // one: an item such as `heading` reads the whole heading.
        if sentence && !self.wants_sentences() {
            return false;
        }
        // An item whose every part names another language selects nothing,
        // whether its parts are negated or not.
        if self.parts.iter().all(|part| !part.applies_to(extension)) {
            return false;
        }
        let mut has_positive = false;
        let mut names_the_block = false;
        for part in &self.parts {
            if part.negated {
                continue;
            }
            has_positive = true;
            if part.name == "sentence" {
                if !sentence {
                    return false;
                }
                names_the_block = true;
            } else if names.iter().any(|name| named(&part.name, name)) {
                names_the_block = true;
            } else if !around.iter().any(|name| named(&part.name, name)) {
                return false;
            }
            if !part.applies_to(extension) {
                return false;
            }
        }
        if !has_positive && !names.contains(&"text") {
            return false;
        }
        if has_positive && !names_the_block {
            return false;
        }
        // A negated part that names the element around the block leaves the
        // block out, so `title & ~link` selects no title of a link.
        // The names of inline elements are read as blank in a block that
        // holds them, and only the element around a block excludes it here.
        let excluded_by_around = self.parts.iter().any(|part| {
            part.negated
                && part.applies_to(extension)
                && around.iter().any(|name| named(&part.name, name))
        });
        !excluded_by_around
            && self
                .parts
                .iter()
                .filter(|part| {
                    part.negated && Inline::of(&part.name).is_none() && part.applies_to(extension)
                })
                .all(|part| !names.iter().any(|name| named(&part.name, name)))
    }
}
