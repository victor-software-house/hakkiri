//! `JSDoc` and `TSDoc` tags in the text of a `/** */` comment.

use std::ops::Range;

/// The ranges of `text` that are prose: the text before the first tag, and the
/// description of each tag.
///
/// A line that starts with `@` and a letter begins a tag.
/// The tag name is not prose, and neither are the type and the name after
/// `@param` or `@typedef`.
/// The description runs to the next tag.
pub(crate) fn prose(text: &str) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let mut start = 0;
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        if let Some(description) = description_start(line) {
            ranges.push(start..offset);
            start = offset + description;
        }
        offset += line.len();
    }
    ranges.push(start..text.len());
    ranges.retain(|range| {
        text.get(range.clone())
            .is_some_and(|prose| !prose.trim().is_empty())
    });
    ranges
}

/// Where the description of the tag that `line` begins starts in `line`, or
/// `None` when `line` begins no tag.
fn description_start(line: &str) -> Option<usize> {
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
    let after_name = indent + "@".len() + name.len();
    if matches!(name, "param" | "typedef") {
        Some(after_parameter(line, after_name))
    } else {
        Some(skip_blanks(line, after_name))
    }
}

/// The offset in `line` after the type and the name that follow `@param` or
/// `@typedef` at `at`.
fn after_parameter(line: &str, at: usize) -> usize {
    let mut at = skip_blanks(line, at);
    if let Some(closed) = line.get(at..).and_then(|rest| closing(rest, '{', '}')) {
        at = skip_blanks(line, at + closed + 1);
    }
    let rest = line.get(at..).unwrap_or_default();
    let name = if rest.starts_with('[') {
        closing(rest, '[', ']').map_or(0, |closed| closed + 1)
    } else {
        rest.find(char::is_whitespace).unwrap_or(rest.len())
    };
    at = skip_blanks(line, at + name);
    // TSDoc puts a hyphen between the name and the description.
    let rest = line.get(at..).unwrap_or_default();
    if rest
        .strip_prefix('-')
        .is_some_and(|after| after.is_empty() || after.starts_with(char::is_whitespace))
    {
        at = skip_blanks(line, at + 1);
    }
    at
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
            depth -= 1;
            if depth == 0 {
                return Some(index);
            }
        }
    }
    None
}
