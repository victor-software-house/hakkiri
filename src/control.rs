//! Comments that turn rules off and on.
//!
//! A port of `UpdateCommentsAt`, `setComment`, `QueryComments` and
//! `RegionDisabled` in `file.go` of [Vale] at `66bbb85`, with the forms that
//! hakkiri adds.
//!
//! An HTML comment of a Markdown file is read as a directive:
//!
//! | Vale | hakkiri | Effect |
//! |:--|:--|:--|
//! | `vale off`, `vale on` | `hakkiri off`, `hakkiri on` | every rule |
//! | `vale Style.Rule = NO` or `YES` | `hakkiri Style.Rule: off` or `on` | one rule |
//! | `vale Style = NO` or `YES` | `hakkiri Style: off` or `on` | one style |
//! | `vale Style.Rule["a","b"] = NO` or `YES` | none | the matches `a` and `b` of a rule |
//! | `vale styles = A, B` | none | only the styles `A` and `B` of the file |
//! | none | `hakkiri Style.Rule: suggestion`, `warning` or `error` | the level of a rule |
//!
//! Vale also reads `on` and `off` for `YES` and `NO` in the second and third
//! rows.
//!
//! Vale reads each comment at two moments.
//! The text of a Markdown file after `off` is left out of its blocks, which
//! `cut` does.
//! A rule is gated per block, by the toggles that a comment inside the element
//! of the block sets before the element ends (`Controls::gated`).
//! An alert is hidden when its place lies between the comment that turned its
//! rule off and the one that turned it on (`Controls::hides`).
//!
//! [Vale]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/file.go

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::ops::Range;

use crate::alert::{Alert, Locator};
use crate::rule::Level;
use crate::tree::{Comment, Node, Tree, remap_nodes};

/// What a comment sets.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Setting {
    /// Every rule, off or on.
    All { disabled: bool },
    /// A rule or a style, by its key, off or on.
    Key { key: String, disabled: bool },
    /// The matches of a rule, off or on.
    Matches {
        rule: String,
        matches: Vec<String>,
        disabled: bool,
    },
    /// Only these styles of the file run.
    Styles(Vec<String>),
    /// The alerts of a rule take a level.
    Level { rule: String, level: Level },
}

/// Whether `key` names a rule, `Style.Rule`, or a style with no dot: Vale's
/// `.+\..+|[^.]+`.
fn is_key(key: &str) -> bool {
    match key.find('.') {
        None => !key.is_empty(),
        Some(_) => key
            .char_indices()
            .any(|(at, c)| c == '.' && at > 0 && at + 1 < key.len()),
    }
}

/// Whether `key` names a rule: a dot between two characters.
fn is_rule(key: &str) -> bool {
    key.contains('.') && is_key(key)
}

/// The strings of a JSON array of strings with no escape but `\"`, `\\` and
/// `\/`, or `None`.
fn json_strings(text: &str) -> Option<Vec<String>> {
    let inner = text.strip_prefix('[')?.strip_suffix(']')?;
    let mut found = Vec::new();
    let mut chars = inner.chars().peekable();
    loop {
        while chars.next_if(|c| c.is_whitespace()).is_some() {}
        if chars.next()? != '"' {
            return None;
        }
        let mut one = String::new();
        loop {
            match chars.next()? {
                '"' => break,
                '\\' => one.push(match chars.next()? {
                    c @ ('"' | '\\' | '/') => c,
                    _ => return None,
                }),
                c => one.push(c),
            }
        }
        found.push(one);
        while chars.next_if(|c| c.is_whitespace()).is_some() {}
        match chars.next() {
            None => return Some(found),
            Some(',') => {}
            Some(_) => return None,
        }
    }
}

/// What the comment `text` sets, or `None` when it sets nothing.
pub(crate) fn setting(text: &str) -> Option<Setting> {
    if let Some(rest) = text.strip_prefix("vale ") {
        return vale_setting(text, rest);
    }
    let rest = text.strip_prefix("hakkiri ")?;
    match rest {
        "off" => Some(Setting::All { disabled: true }),
        "on" => Some(Setting::All { disabled: false }),
        _ => {
            let (key, word) = rest.split_once(": ")?;
            if !is_key(key) || key.contains(' ') {
                return None;
            }
            let level = match word {
                "suggestion" => Level::Suggestion,
                "warning" => Level::Warning,
                "error" => Level::Error,
                "off" | "on" => {
                    return Some(Setting::Key {
                        key: key.to_owned(),
                        disabled: word == "off",
                    });
                }
                _ => return None,
            };
            is_rule(key).then(|| Setting::Level {
                rule: key.to_owned(),
                level,
            })
        }
    }
}

/// The setting of a comment `text` that starts with `vale `, whose rest is
/// `rest`.
fn vale_setting(text: &str, rest: &str) -> Option<Setting> {
    match text {
        "vale off" => return Some(Setting::All { disabled: true }),
        "vale on" => return Some(Setting::All { disabled: false }),
        _ => {}
    }
    if let Some((body, answer)) = rest
        .strip_suffix(" = YES")
        .map(|body| (body, "YES"))
        .or_else(|| rest.strip_suffix(" = NO").map(|body| (body, "NO")))
        && let Some(open) = body.find('[')
        && body.ends_with(']')
        && body.len() > open + 2
        && is_rule(body.get(..open)?)
    {
        let rule = body.get(..open)?.to_owned();
        return Some(Setting::Matches {
            matches: json_strings(body.get(open..)?)?,
            rule,
            disabled: answer == "NO",
        });
    }
    for answer in ["YES", "NO", "on", "off"] {
        if let Some(key) = rest
            .strip_suffix(answer)
            .and_then(|key| key.strip_suffix(" = "))
            && is_key(key)
        {
            return Some(Setting::Key {
                key: key.to_owned(),
                disabled: answer == "NO" || answer == "off",
            });
        }
    }
    let names = rest
        .strip_prefix("styles = ")
        .or_else(|| rest.strip_prefix("style = "))?;
    Some(Setting::Styles(
        names.split(", ").map(str::to_owned).collect(),
    ))
}

/// The style of a rule named `Style.Rule`.
fn style_of(rule: &str) -> &str {
    rule.split_once('.').map_or(rule, |(style, _)| style)
}

/// A line and a column, which order a place in the file.
type Place = (usize, usize);

/// The stretch between the comment that turned a key off and the one that
/// turned it back on.
#[derive(Clone, Copy, Debug)]
struct Region {
    begin: Place,
    /// Where the region ends, or `None` when it runs to the end of the file.
    end: Option<Place>,
}

/// The comments of a file, read.
#[derive(Debug, Default)]
pub(crate) struct Controls {
    /// The toggles after each comment, with the byte the comment starts at.
    states: Vec<(usize, HashMap<String, bool>)>,
    regions: HashMap<String, Vec<Region>>,
    /// The level a rule has from each comment on, or `None` for its own.
    levels: HashMap<String, Vec<(Place, Option<Level>)>>,
}

impl Controls {
    /// The directives of `comments`, a file of `source` whose rules are of
    /// `styles`.
    pub(crate) fn new(comments: &[Comment], source: &str, styles: &[String]) -> Self {
        let mut controls = Self::default();
        let mut locator = Locator::new(source);
        let mut toggles: HashMap<String, bool> = HashMap::new();
        for comment in comments {
            let Some(setting) = setting(&comment.text) else {
                continue;
            };
            let place = locator.place(comment.at);
            match setting {
                Setting::All { disabled } => {
                    controls.toggle(&mut toggles, "off", disabled, place);
                }
                Setting::Key { key, disabled } => {
                    controls.toggle(&mut toggles, &key, disabled, place);
                    if is_rule(&key) {
                        controls.levels.entry(key).or_default().push((place, None));
                    }
                }
                Setting::Matches {
                    rule,
                    matches,
                    disabled,
                } => {
                    for found in matches {
                        let key = format!("{rule}[{found}]");
                        controls.toggle(&mut toggles, &key, disabled, place);
                    }
                }
                Setting::Styles(listed) => {
                    for style in styles {
                        toggles.insert(style.clone(), true);
                    }
                    for style in listed {
                        toggles.insert(style, false);
                    }
                }
                Setting::Level { rule, level } => {
                    controls
                        .levels
                        .entry(rule)
                        .or_default()
                        .push((place, Some(level)));
                }
            }
            controls.states.push((comment.at, toggles.clone()));
        }
        controls
    }

    /// Sets the toggle `key` and opens or closes its region at `place`.
    fn toggle(
        &mut self,
        toggles: &mut HashMap<String, bool>,
        key: &str,
        disabled: bool,
        place: Place,
    ) {
        toggles.insert(key.to_owned(), disabled);
        let regions = self.regions.entry(key.to_owned()).or_default();
        let open = regions.last().is_some_and(|region| region.end.is_none());
        if disabled {
            if !open {
                regions.push(Region {
                    begin: place,
                    end: None,
                });
            }
        } else if open && let Some(region) = regions.last_mut() {
            region.end = Some(place);
        }
    }

    /// The toggles when the element that ends at the byte `closes` is read:
    /// those of the last comment before it.
    fn toggles_at(&self, closes: usize) -> Option<&HashMap<String, bool>> {
        let after = self.states.partition_point(|(at, _)| *at < closes);
        after
            .checked_sub(1)
            .and_then(|index| self.states.get(index))
            .map(|(_, toggles)| toggles)
    }

    /// Whether a comment turned every rule off before the byte `closes`.
    pub(crate) fn off_at(&self, closes: usize) -> bool {
        self.toggles_at(closes)
            .is_some_and(|toggles| toggles.get("off").copied().unwrap_or(false))
    }

    /// Whether a comment turned `rule` off, or every rule or its style, for a
    /// block whose element ends at the byte `closes`.
    pub(crate) fn gated(&self, rule: &str, closes: usize) -> bool {
        let Some(toggles) = self.toggles_at(closes) else {
            return false;
        };
        let on = |key: &str| toggles.get(key).copied().unwrap_or(false);
        on("off") || on(style_of(rule)) || on(rule)
    }

    /// Whether an alert of `rule` that matched `matched`, at the line and
    /// column `place`, is hidden.
    ///
    /// It is hidden when a comment turned off the matches of the rule for a
    /// block that ends at `closes`, or when the alert lies in a region of a
    /// comment.
    pub(crate) fn hides(&self, rule: &str, matched: &str, place: Place, closes: usize) -> bool {
        let match_key = format!("{rule}[{matched}]");
        if !matched.is_empty()
            && self
                .toggles_at(closes)
                .is_some_and(|toggles| toggles.get(&match_key).copied().unwrap_or(false))
        {
            return true;
        }
        let style = style_of(rule);
        let mut keys = vec!["off", rule];
        if style != rule {
            keys.push(style);
        }
        if !matched.is_empty() {
            keys.push(&match_key);
        }
        keys.iter().any(|key| {
            self.regions.get(*key).is_some_and(|regions| {
                regions
                    .iter()
                    .any(|region| region.begin <= place && region.end.is_none_or(|end| place < end))
            })
        })
    }

    /// The level a comment gives the alerts of `rule` at `place`.
    pub(crate) fn level(&self, rule: &str, place: Place) -> Option<Level> {
        let timeline = self.levels.get(rule)?;
        let after = timeline.partition_point(|(from, _)| *from <= place);
        after
            .checked_sub(1)
            .and_then(|index| timeline.get(index))
            .and_then(|(_, level)| *level)
    }
}

/// The styles of the rules named `rules`, each once.
pub(crate) fn styles_of<'a>(rules: impl Iterator<Item = &'a str>) -> Vec<String> {
    let mut styles: Vec<String> = Vec::new();
    for rule in rules {
        let style = style_of(rule);
        if !styles.iter().any(|known| known == style) {
            styles.push(style.to_owned());
        }
    }
    styles
}

/// The bytes between a comment that turns every rule off and the one that
/// turns them on, or the end of the file.
fn off_ranges(comments: &[Comment], len: usize) -> Vec<Range<usize>> {
    let mut ranges: Vec<Range<usize>> = Vec::new();
    let mut open: Option<usize> = None;
    for comment in comments {
        match setting(&comment.text) {
            Some(Setting::All { disabled: true }) => {
                open.get_or_insert(comment.at);
            }
            Some(Setting::All { disabled: false }) => {
                if let Some(start) = open.take() {
                    ranges.push(start..comment.at);
                }
            }
            _ => {}
        }
    }
    if let Some(start) = open {
        ranges.push(start..len);
    }
    ranges
}

/// The parts of `span` that lie outside `ranges`, which are in order and do
/// not overlap. A span of no bytes is kept unless a range holds it.
fn outside(span: &Range<usize>, ranges: &[Range<usize>]) -> Vec<Range<usize>> {
    if span.is_empty() {
        let held = ranges
            .iter()
            .any(|range| range.start < span.start && span.start < range.end);
        return if held { Vec::new() } else { vec![span.clone()] };
    }
    let first = ranges.partition_point(|range| range.end <= span.start);
    let mut kept = Vec::new();
    let mut at = span.start;
    for range in ranges
        .get(first..)
        .unwrap_or_default()
        .iter()
        .take_while(|range| range.start < span.end)
    {
        if range.start > at {
            kept.push(at..range.start);
        }
        at = at.max(range.end);
    }
    if at < span.end {
        kept.push(at..span.end);
    }
    kept
}

/// Leaves out of `nodes` the text between a comment that turns every rule off
/// and the one that turns them on, as Vale leaves it out of its blocks.
///
/// `len` is the length of the file.
pub(crate) fn cut(nodes: &mut Vec<Node>, comments: &[Comment], len: usize) {
    let ranges = off_ranges(comments, len);
    if ranges.is_empty() {
        return;
    }
    remap_nodes(nodes, &|span| outside(&span, &ranges), &Some);
}

/// Gives each alert the level that a comment of `tree` sets for its rule at
/// its place, and leaves the others as they are, and so a fault of a rule
/// itself (`Alert::fault`).
///
/// The comment is `hakkiri Style.Rule: suggestion`, `warning` or `error`, and
/// it holds until a later comment names the rule.
/// `source` is the text `tree` was parsed from.
pub fn apply_levels(tree: &Tree, source: &str, alerts: &mut [Alert]) {
    if tree.comments.is_empty() {
        return;
    }
    let controls = Controls::new(&tree.comments, source, &[]);
    for alert in alerts.iter_mut().filter(|alert| !alert.fault) {
        if let Some(level) = controls.level(&alert.rule, (alert.line, alert.first_column)) {
            alert.level = level;
        }
    }
}
