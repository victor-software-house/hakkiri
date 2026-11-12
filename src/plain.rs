//! Plain text: the tree of a file that no language reads.

use crate::tree::{Node, Scope};

/// The roots of a plain-text file: one `raw` root over the whole file, whose
/// children are its paragraphs, the runs of lines that hold something between
/// blank lines.
///
/// A paragraph spans from its first character to its last, with no final line
/// break. The text is read as written: no marker, escape or entity changes it.
pub(crate) fn roots(source: &str) -> Vec<Node> {
    let mut paragraphs = Vec::new();
    let mut start = None;
    let mut end = 0;
    let mut offset = 0;
    for line in source.split_inclusive('\n') {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if let Some(first) = start.take() {
                paragraphs.push(Node::new(Scope::Paragraph, first..end, Vec::new()));
            }
        } else {
            let lead = line.len() - line.trim_start().len();
            start.get_or_insert(offset + lead);
            end = offset + lead + trimmed.len();
        }
        offset += line.len();
    }
    if let Some(first) = start {
        paragraphs.push(Node::new(Scope::Paragraph, first..end, Vec::new()));
    }
    vec![Node::new(Scope::Raw, 0..source.len(), paragraphs)]
}
