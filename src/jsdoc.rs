//! `JSDoc` and `TSDoc` tags in the text of a `/** */` comment.

use std::ops::Range;

/// The ranges of `text` that are prose: the text before the first tag, and the
/// description of each tag.
///
/// A line that starts with `@` and a letter begins a tag, unless it sits in a
/// fenced block.
/// The tag name is not prose, and neither is a type in braces after it.
/// After `@param` and the tags that name something, the name is not prose
/// either.
/// The body of `@example` is code, not prose.
/// A description runs to the next tag.
pub(crate) fn prose(text: &str) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let mut start = 0;
    let mut code = false;
    let mut fenced = false;
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        let marker = line.trim_start();
        if marker.starts_with("```") || marker.starts_with("~~~") {
            fenced = !fenced;
        }
        if !fenced && let Some(tag) = Tag::of(line) {
            if !code {
                ranges.push(start..offset);
            }
            start = offset + tag.description;
            code = tag.code;
        }
        offset += line.len();
    }
    if !code {
        ranges.push(start..text.len());
    }
    ranges.retain(|range| {
        text.get(range.clone())
            .is_some_and(|prose| !prose.trim().is_empty())
    });
    ranges
}

/// A tag that a line begins.
struct Tag {
    /// Where the description starts in the line.
    description: usize,
    /// Whether the description is code.
    code: bool,
}

impl Tag {
    /// The tag that `line` begins, or `None` when `line` begins no tag.
    fn of(line: &str) -> Option<Self> {
        let body = line.trim_start();
        let indent = line.len() - body.len();
        let tag = body.strip_prefix('@')?;
        if !tag.starts_with(|c: char| c.is_ascii_alphabetic()) {
            return None;
        }
        let name = tag
            .split(|c: char| !c.is_ascii_alphanumeric())
            .next()
            .unwrap_or_default();
        let mut at = skip_blanks(line, indent + "@".len() + name.len());
        at = skip_type(line, at);
        if matches!(
            name,
            "param"
                | "arg"
                | "argument"
                | "prop"
                | "property"
                | "typedef"
                | "callback"
                | "event"
                | "member"
                | "var"
                | "name"
                | "alias"
                | "memberof"
                | "augments"
                | "extends"
                | "mixes"
                | "borrows"
                | "function"
                | "func"
                | "method"
                | "class"
                | "constructor"
                | "namespace"
                | "module"
                | "enum"
                | "external"
                | "host"
                | "kind"
                | "listens"
                | "fires"
                | "emits"
                | "this"
                | "requires"
                | "template"
                | "implements"
                | "satisfies"
                | "see"
        ) {
            at = skip_name(line, at);
        }
        Some(Self {
            description: at,
            code: name == "example",
        })
    }
}

/// The offset in `line` after the type in braces at `at`, or `at` when no type
/// starts there.
fn skip_type(line: &str, at: usize) -> usize {
    let rest = line.get(at..).unwrap_or_default();
    match closing(rest, '{', '}') {
        // `{@link X}` is an inline tag and no type.
        Some(closed) if rest.starts_with('{') && !rest.starts_with("{@") => {
            skip_blanks(line, at + closed + 1)
        }
        _ => at,
    }
}

/// The offset in `line` after the name at `at` and the hyphen that follows it.
fn skip_name(line: &str, at: usize) -> usize {
    let rest = line.get(at..).unwrap_or_default();
    // `@see {@link X} for more` names nothing: the link is an inline tag.
    if rest.starts_with("{@") {
        return at;
    }
    let name = if rest.starts_with('[') {
        closing(rest, '[', ']').map_or(0, |closed| closed + 1)
    } else {
        rest.find(char::is_whitespace).unwrap_or(rest.len())
    };
    let at = skip_blanks(line, at + name);
    // TSDoc puts a hyphen between the name and the description.
    let rest = line.get(at..).unwrap_or_default();
    if rest
        .strip_prefix('-')
        .is_some_and(|after| after.is_empty() || after.starts_with(char::is_whitespace))
    {
        skip_blanks(line, at + 1)
    } else {
        at
    }
}

fn skip_blanks(line: &str, at: usize) -> usize {
    let rest = line.get(at..).unwrap_or_default();
    at + rest.len() - rest.trim_start_matches([' ', '\t']).len()
}

/// The byte index of the `close` that matches the `open` that starts `text`, on
/// its first line.
fn closing(text: &str, open: char, close: char) -> Option<usize> {
    let mut depth = 0_usize;
    for (index, c) in text.char_indices() {
        if c == '\n' {
            return None;
        }
        if c == open {
            depth += 1;
        } else if c == close {
            depth = depth.checked_sub(1)?;
            if depth == 0 {
                return Some(index);
            }
        }
    }
    None
}
