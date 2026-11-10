//! Alerts: what a rule reports, and where in the file it points.

use std::ops::Range;

use crate::rule::{Action, Level, Rule};
use crate::text::Text;

/// What a rule reports about one match.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Alert {
    /// The name of the rule, such as `Team.Filler`.
    pub rule: String,
    /// The level of the alert.
    pub level: Level,
    /// The rule's message with its `%s` filled.
    pub message: String,
    /// The rule's description with its `%s` filled, empty when the rule has
    /// none.
    pub description: String,
    /// The rule's `link`, empty when the rule has none.
    pub link: String,
    /// The 1-based line of the match.
    pub line: usize,
    /// The 1-based column, in characters, where the match starts.
    pub first_column: usize,
    /// The 1-based column, in characters, of the last character of the match
    /// on its first line.
    pub last_column: usize,
    /// The text that matched.
    pub matched: String,
    /// The rule's `action`. Nothing applies it.
    pub action: Option<Action>,
}

/// Fills the `%s` of `template` with `values`, in order.
///
/// `%%` is `%`. A `%s` with no value left is nothing, and a value left over is
/// dropped. A `%` before any other character stays as written. A line break
/// in a value is one space.
#[must_use]
pub fn fill(template: &str, values: &[&str]) -> String {
    let mut out = String::with_capacity(template.len());
    let mut values = values.iter();
    let mut chars = template.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '%' {
            out.push(c);
            continue;
        }
        match chars.peek() {
            Some('%') => {
                chars.next();
                out.push('%');
            }
            Some('s') => {
                chars.next();
                if let Some(value) = values.next() {
                    out.extend(value.chars().map(|v| if v == '\n' { ' ' } else { v }));
                }
            }
            _ => out.push('%'),
        }
    }
    out
}

/// Where the bytes of one file lie as lines and columns.
///
/// It keeps the byte each line starts at, and counts the characters of a line
/// from where the last count stopped, so the alerts of a long line cost the
/// length of the line once.
#[derive(Debug)]
pub struct Locator<'a> {
    source: &'a str,
    starts: Vec<usize>,
    /// The byte a count of characters reached, the line it began on, and the
    /// characters it counted.
    cursor: Cursor,
}

#[derive(Clone, Copy, Debug)]
struct Cursor {
    line_start: usize,
    at: usize,
    columns: usize,
}

impl<'a> Locator<'a> {
    /// A locator for `source`.
    #[must_use]
    pub fn new(source: &'a str) -> Self {
        let mut starts = vec![0];
        starts.extend(source.match_indices('\n').map(|(index, _)| index + 1));
        Self {
            source,
            starts,
            cursor: Cursor {
                line_start: 0,
                at: 0,
                columns: 0,
            },
        }
    }

    /// The 1-based line and column of the file byte `offset`.
    ///
    /// A column counts characters. A byte inside a character belongs to that
    /// character.
    pub fn place(&mut self, offset: usize) -> (usize, usize) {
        let line = self.starts.partition_point(|&start| start <= offset);
        let line_start = line
            .checked_sub(1)
            .and_then(|index| self.starts.get(index))
            .copied()
            .unwrap_or_default();
        if self.cursor.line_start != line_start || offset < self.cursor.at {
            self.cursor = Cursor {
                line_start,
                at: line_start,
                columns: 0,
            };
        }
        self.cursor.columns += self.characters(self.cursor.at..offset);
        self.cursor.at = offset;
        (line, self.cursor.columns + 1)
    }

    /// The characters that start inside the bytes `range` of the source.
    fn characters(&self, range: Range<usize>) -> usize {
        self.source.as_bytes().get(range).map_or(0, |bytes| {
            bytes.iter().filter(|b| **b & 0xc0 != 0x80).count()
        })
    }

    /// The alert of `rule` for the bytes `range` of `text`, whose message and
    /// description take `values`.
    ///
    /// Returns `None` when `range` is not in `text`.
    pub fn alert(
        &mut self,
        rule: &Rule,
        text: &Text,
        range: Range<usize>,
        values: &[&str],
    ) -> Option<Alert> {
        let matched = text.as_str().get(range.clone())?;
        let offset = text.file_range(range)?.start;
        let (line, first_column) = self.place(offset);
        Some(Alert {
            rule: rule.name.clone(),
            level: rule.level,
            message: fill(&rule.message, values),
            description: fill(&rule.description, values),
            link: rule.link.clone(),
            line,
            first_column,
            last_column: first_column + extent(matched),
            matched: matched.to_owned(),
            action: rule.action.clone(),
        })
    }

    /// An alert that points at the start of `text`, for a fault of the rule
    /// itself.
    pub fn failure(&mut self, rule: &Rule, text: &Text, message: String) -> Alert {
        let (line, first_column) = self.place(text.file_offset(0).unwrap_or_default());
        Alert {
            rule: rule.name.clone(),
            level: Level::Error,
            message,
            description: String::new(),
            link: String::new(),
            line,
            first_column,
            last_column: first_column,
            matched: String::new(),
            action: None,
        }
    }
}

/// How many columns past its first one a match reaches, counting only its
/// first line. A match with no text reaches none.
fn extent(matched: &str) -> usize {
    matched
        .split_inclusive('\n')
        .next()
        .map_or(0, |line| line.chars().count().saturating_sub(1))
}

/// The alerts of one rule in one file, in file order, without a second alert
/// at one line and column, and at most `limit` of them when `limit` is above
/// 0.
#[must_use]
pub fn settle(mut alerts: Vec<Alert>, limit: usize) -> Vec<Alert> {
    alerts.sort_by_key(|alert| (alert.line, alert.first_column));
    alerts.dedup_by_key(|alert| (alert.line, alert.first_column));
    if limit > 0 {
        alerts.truncate(limit);
    }
    alerts
}
