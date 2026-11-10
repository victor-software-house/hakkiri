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
    /// in the file, or of the end of the first line when the match goes on to
    /// the next.
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

    /// The column of the last character of `line`, counting its line feed.
    fn line_end(&self, line: usize) -> usize {
        let start = line
            .checked_sub(1)
            .and_then(|index| self.starts.get(index))
            .copied()
            .unwrap_or_default();
        let end = self.starts.get(line).copied().unwrap_or(self.source.len());
        self.characters(start..end)
    }

    /// The characters that start inside the bytes `range` of the source.
    fn characters(&self, range: Range<usize>) -> usize {
        self.source.as_bytes().get(range).map_or(0, |bytes| {
            bytes.iter().filter(|b| **b & 0xc0 != 0x80).count()
        })
    }

    /// The alert of `rule` for the bytes `range` of `text`, with the filled
    /// `message` and `description`.
    ///
    /// Returns `None` when `range` is not in `text`.
    pub fn alert(
        &mut self,
        rule: &Rule,
        text: &Text,
        range: Range<usize>,
        (message, description): (String, String),
    ) -> Option<Alert> {
        let matched = text.as_str().get(range.clone())?;
        let offset = text.file_range(range.clone())?.start;
        let last = matched
            .chars()
            .next_back()
            .and_then(|c| text.file_offset(range.end - c.len_utf8()));
        Some(self.report(rule, (offset, last), matched, (message, description)))
    }

    /// The alert of `rule` for a text with no match to point at, at the file
    /// byte `offset`.
    pub fn alert_at(&mut self, rule: &Rule, offset: usize, messages: (String, String)) -> Alert {
        self.report(rule, (offset, None), "", messages)
    }

    /// The alert of `rule` for a text with no match to point at, at column 1
    /// of the line the text starts on.
    pub fn alert_on_line(&mut self, rule: &Rule, text: &Text, messages: (String, String)) -> Alert {
        let (line, _) = self.place(text.file_offset(0).unwrap_or_default());
        let start = line
            .checked_sub(1)
            .and_then(|index| self.starts.get(index))
            .copied()
            .unwrap_or_default();
        self.report(rule, (start, None), "", messages)
    }

    /// `(offset, last)` is the file byte where the match starts, and where its
    /// last character starts when it has one.
    fn report(
        &mut self,
        rule: &Rule,
        (offset, last): (usize, Option<usize>),
        matched: &str,
        (message, description): (String, String),
    ) -> Alert {
        let (line, first_column) = self.place(offset);
        let last_column = last.map_or(first_column, |last| {
            let (last_line, column) = self.place(last);
            if last_line > line {
                self.line_end(line)
            } else {
                column.max(first_column)
            }
        });
        Alert {
            rule: rule.name.clone(),
            level: rule.level,
            message,
            description,
            link: rule.link.clone(),
            line,
            first_column,
            last_column,
            matched: matched.to_owned(),
            action: rule.action.clone(),
        }
    }

    /// An alert that points at the start of `text`, for a fault of the rule
    /// itself.
    pub fn failure(&mut self, rule: &Rule, text: &Text, message: String) -> Alert {
        self.failure_at(rule, text.file_offset(0).unwrap_or_default(), message)
    }

    /// An alert that points at the file byte `offset`, for a fault of the
    /// rule itself.
    pub fn failure_at(&mut self, rule: &Rule, offset: usize, message: String) -> Alert {
        let (line, first_column) = self.place(offset);
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
