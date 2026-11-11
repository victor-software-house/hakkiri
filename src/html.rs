//! The text of an HTML block, read as Vale reads it.
//!
//! Vale renders a Markdown file to HTML and walks the tokens of
//! `golang.org/x/net/html` at v0.62.0.
//! It collects the text of a block, and reads the block under the tags opened
//! since the last reset.
//! It flushes the block at the next start tag that is not inline, at an end
//! tag that is not inline, and at the end of the input.
//! It resets the tags at an end tag that is not inline, and when it flushes
//! at a start tag.
//! A block element that opens inside an inline element reads as inline.
//! The text of `script`, `style`, `noscript`, `iframe` and `pre` is left out,
//! as is the text of an element whose class is `code`, `pre` or `problematic`.
//! The first text after a `tt`, `code`, `kbd` or `samp` start tag is masked,
//! and reads as one `*` for each character.
//! hakkiri reads the text of `tt` and `code` as a code span, which a rule reads
//! as one word.
//!
//! Vale joins the texts of a block with a space where an inline tag opens
//! before the text, and as the source has it where one closes before it.
//! [`Spacer`] follows that.
//!
//! This module follows [that tokenizer's rules][tokenizer] for where a tag
//! ends and what is text, so a `>` in a quoted attribute value, or inside a
//! bogus comment such as `<?php echo "<div>"; ?>`, is read as it reads it.
//! The content of a `script` element ends at the first `</script` that no
//! `<!--` and `<script` hide.
//!
//! [tokenizer]: https://github.com/golang/net/blob/v0.62.0/html/token.go

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::ops::Range;

use crate::entity;
use crate::tree::{Join, Reading};

/// The inline tags: an end tag of one resets nothing.
const INLINE_TAGS: [&str; 23] = [
    "b", "big", "i", "small", "abbr", "acronym", "cite", "dfn", "em", "kbd", "strong", "a", "br",
    "img", "span", "sub", "sup", "code", "tt", "del", "q", "samp", "var",
];

/// The tags that never hold content.
const VOID_TAGS: [&str; 13] = [
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source", "track",
    "wbr",
];

/// The tags whose content Vale leaves out of a block.
const SKIP_TAGS: [&str; 5] = ["script", "style", "pre", "noscript", "iframe"];

/// The classes whose element Vale leaves out of a block.
const SKIP_CLASSES: [&str; 3] = ["problematic", "pre", "code"];

/// The tags whose first text Vale masks.
const MASKED_TAGS: [&str; 4] = ["tt", "code", "kbd", "samp"];

/// The tags whose first text hakkiri reads as a code span, and not as `*`.
const CODE_TAGS: [&str; 2] = ["tt", "code"];

/// The tags whose content is text up to their end tag.
const RAW_TEXT_TAGS: [&str; 10] = [
    "iframe",
    "noembed",
    "noframes",
    "noscript",
    "plaintext",
    "script",
    "style",
    "textarea",
    "title",
    "xmp",
];

/// The whitespace the tokenizer skips: not a vertical tab, which only the
/// trimming of a text counts (`is_blank_char`).
const fn is_blank(c: u8) -> bool {
    matches!(c, b' ' | b'\n' | b'\r' | b'\t' | 0x0c)
}

/// An attribute of a tag.
#[derive(Debug)]
struct Attribute {
    /// The name, in lowercase.
    key: String,
    /// The bytes of the value, quotes aside.
    value: Range<usize>,
}

/// A start, end or self-closing tag.
#[derive(Debug)]
struct Tag {
    /// The name, in lowercase.
    name: String,
    attributes: Vec<Attribute>,
}

impl Tag {
    /// Whether the `class` of the tag names a class Vale leaves out.
    fn skipped_class(&self, html: &str) -> bool {
        self.attributes
            .iter()
            .find(|attribute| attribute.key == "class")
            .and_then(|attribute| html.get(attribute.value.clone()))
            .is_some_and(|class| class.split(' ').any(|name| SKIP_CLASSES.contains(&name)))
    }
}

#[derive(Debug)]
enum Token {
    Text(Range<usize>),
    Start(Tag),
    SelfClosing(Tag),
    End(Tag),
    /// A comment, a declaration, a processing instruction or `</>`, and its
    /// bytes.
    Comment(Range<usize>),
    /// The end of the input, or a tag that it ends.
    Eof,
}

/// A state of the script reader.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Script {
    Data,
    LessThan,
    EndTagOpen,
    EscapeStart,
    EscapeStartDash,
    Escaped,
    EscapedDash,
    EscapedDashDash,
    EscapedLessThan,
    EscapedEndTagOpen,
    DoubleEscapeStart,
    DoubleEscaped,
    DoubleEscapedDash,
    DoubleEscapedDashDash,
    DoubleEscapedLessThan,
    DoubleEscapeEnd,
}

/// The tokenizer of `golang.org/x/net/html` at v0.62.0, over the bytes of one
/// block.
struct Tokenizer<'a> {
    bytes: &'a [u8],
    at: usize,
    /// The raw text tag whose content comes next.
    raw_tag: Option<&'static str>,
    /// A read went past the end.
    eof: bool,
}

impl<'a> Tokenizer<'a> {
    const fn new(html: &'a str) -> Self {
        Self {
            bytes: html.as_bytes(),
            at: 0,
            raw_tag: None,
            eof: false,
        }
    }

    fn read(&mut self) -> Option<u8> {
        if let Some(&c) = self.bytes.get(self.at) {
            self.at += 1;
            Some(c)
        } else {
            self.eof = true;
            None
        }
    }

    /// Steps back over a byte that was read.
    const fn back(&mut self) {
        self.at -= 1;
    }

    fn skip_blanks(&mut self) {
        while let Some(c) = self.read() {
            if !is_blank(c) {
                self.back();
                return;
            }
        }
    }

    fn read_token(&mut self) -> Token {
        let start = self.at;
        if let Some(raw) = self.raw_tag.take() {
            if raw == "plaintext" {
                self.at = self.bytes.len();
                self.eof = true;
            } else if raw == "script" {
                self.read_script();
            } else {
                self.read_raw_text(raw);
            }
            if self.at > start {
                return Token::Text(start..self.at);
            }
        }
        while let Some(c) = self.read() {
            if c != b'<' {
                continue;
            }
            let Some(c) = self.read() else { break };
            let opens_tag = c.is_ascii_alphabetic();
            if !(opens_tag || matches!(c, b'/' | b'!' | b'?')) {
                self.back();
                continue;
            }
            let lt = self.at - 2;
            if start < lt {
                self.at = lt;
                return Token::Text(start..lt);
            }
            if opens_tag {
                return self.read_start_tag();
            }
            if c == b'/' {
                let Some(c) = self.read() else { break };
                if c == b'>' {
                    return Token::Comment(lt..self.at);
                }
                if c.is_ascii_alphabetic() {
                    let tag = self.read_tag(false);
                    return if self.eof {
                        Token::Eof
                    } else {
                        Token::End(tag)
                    };
                }
                self.back();
                self.read_until_close_angle();
                return Token::Comment(lt..self.at);
            }
            if c == b'!' {
                self.read_markup_declaration();
            } else {
                self.back();
                self.read_until_close_angle();
            }
            return Token::Comment(lt..self.at);
        }
        if start < self.at {
            return Token::Text(start..self.at);
        }
        Token::Eof
    }

    /// Reads up to the end tag of `raw`, which is not consumed.
    fn read_raw_text(&mut self, raw: &str) {
        loop {
            let Some(c) = self.read() else { return };
            if c != b'<' {
                continue;
            }
            let Some(c) = self.read() else { return };
            if c != b'/' {
                self.back();
                continue;
            }
            if self.read_raw_end_tag(raw) || self.eof {
                return;
            }
        }
    }

    /// Reads the content of a `script` element, up to its end tag, which is
    /// not consumed.
    ///
    /// A `<!--` hides the end tag until a `-->`, and a `<script` after it
    /// hides it again until the end tag of that script, as the HTML5 standard
    /// defines.
    fn read_script(&mut self) {
        let mut state = Script::Data;
        loop {
            let Some(next) = self.script_step(state) else {
                return;
            };
            state = next;
        }
    }

    /// Reads one byte of a script and gives the state it leaves, or `None`
    /// when the content ends.
    fn script_step(&mut self, state: Script) -> Option<Script> {
        use Script::{
            Data, DoubleEscapeEnd, DoubleEscapeStart, DoubleEscaped, DoubleEscapedDash,
            DoubleEscapedDashDash, DoubleEscapedLessThan, EndTagOpen, EscapeStart, EscapeStartDash,
            Escaped, EscapedDash, EscapedDashDash, EscapedEndTagOpen, EscapedLessThan, LessThan,
        };
        match state {
            EndTagOpen | EscapedEndTagOpen => {
                if self.read_raw_end_tag("script") || self.eof {
                    return None;
                }
                Some(if state == EndTagOpen { Data } else { Escaped })
            }
            DoubleEscapeStart => {
                self.back();
                for letter in "script".bytes() {
                    let c = self.read()?;
                    if !c.eq_ignore_ascii_case(&letter) {
                        self.back();
                        return Some(Escaped);
                    }
                }
                let c = self.read()?;
                if is_blank(c) || matches!(c, b'/' | b'>') {
                    return Some(DoubleEscaped);
                }
                self.back();
                Some(Escaped)
            }
            DoubleEscapeEnd => {
                if self.read_raw_end_tag("script") {
                    self.at += "</script>".len();
                    return Some(Escaped);
                }
                if self.eof {
                    return None;
                }
                Some(DoubleEscaped)
            }
            Data
            | LessThan
            | EscapeStart
            | EscapeStartDash
            | Escaped
            | EscapedDash
            | EscapedDashDash
            | EscapedLessThan
            | DoubleEscaped
            | DoubleEscapedDash
            | DoubleEscapedDashDash
            | DoubleEscapedLessThan => {
                let c = self.read()?;
                Some(match (state, c) {
                    (Data, b'<') => LessThan,
                    (LessThan, b'/') => EndTagOpen,
                    (LessThan, b'!') => EscapeStart,
                    (EscapeStart, b'-') => EscapeStartDash,
                    (EscapeStartDash | EscapedDash | EscapedDashDash, b'-') => EscapedDashDash,
                    (Escaped | EscapedDash | EscapedDashDash, b'<') => EscapedLessThan,
                    (Escaped, b'-') => EscapedDash,
                    (EscapedLessThan, b'/') => EscapedEndTagOpen,
                    (EscapedLessThan, letter) if letter.is_ascii_alphabetic() => DoubleEscapeStart,
                    (DoubleEscaped, b'-') => DoubleEscapedDash,
                    (DoubleEscapedDash | DoubleEscapedDashDash, b'-') => DoubleEscapedDashDash,
                    (DoubleEscaped | DoubleEscapedDash | DoubleEscapedDashDash, b'<') => {
                        DoubleEscapedLessThan
                    }
                    (DoubleEscapedLessThan, b'/') => DoubleEscapeEnd,
                    // The states above that read an end tag or a word give no
                    // byte here.
                    (EscapedDashDash | DoubleEscapedDashDash, b'>') | (Data | EndTagOpen, _) => {
                        Data
                    }
                    (LessThan | EscapeStart | EscapeStartDash | EscapedLessThan, _) => {
                        // A byte that starts nothing is read again as script.
                        self.back();
                        Data
                    }
                    (DoubleEscapedLessThan, _) => {
                        self.back();
                        DoubleEscaped
                    }
                    (
                        Escaped | EscapedDash | EscapedDashDash | EscapedEndTagOpen
                        | DoubleEscapeStart | DoubleEscapeEnd,
                        _,
                    ) => Escaped,
                    (DoubleEscaped | DoubleEscapedDash | DoubleEscapedDashDash, _) => DoubleEscaped,
                })
            }
        }
    }

    /// Reads `</raw` and the byte after it, and backs up to the `<` when they
    /// are an end tag. The `</` is read.
    fn read_raw_end_tag(&mut self, raw: &str) -> bool {
        for letter in raw.bytes() {
            let Some(c) = self.read() else { return false };
            if !c.eq_ignore_ascii_case(&letter) {
                self.back();
                return false;
            }
        }
        let Some(c) = self.read() else { return false };
        if is_blank(c) || c == b'/' || c == b'>' {
            self.at -= 3 + raw.len();
            return true;
        }
        self.back();
        false
    }

    /// Reads a start tag whose `<` and first letter are read.
    fn read_start_tag(&mut self) -> Token {
        let tag = self.read_tag(true);
        if self.eof {
            return Token::Eof;
        }
        if let Some(raw) = RAW_TEXT_TAGS.iter().copied().find(|raw| *raw == tag.name) {
            self.raw_tag = Some(raw);
        }
        let before_close = self.at - 2;
        let last_value_end = tag
            .attributes
            .last()
            .map(|attribute| attribute.value.end.wrapping_sub(1));
        let self_closing =
            self.bytes.get(before_close) == Some(&b'/') && last_value_end != Some(before_close);
        if self_closing {
            Token::SelfClosing(tag)
        } else {
            Token::Start(tag)
        }
    }

    /// Reads the name and attributes of a tag whose `<` and first letter are
    /// read, up to its `>`.
    fn read_tag(&mut self, save: bool) -> Tag {
        let name_start = self.at - 1;
        let name_end = loop {
            match self.read() {
                None => break self.at,
                Some(c) if is_blank(c) => break self.at - 1,
                Some(b'/' | b'>') => {
                    self.back();
                    break self.at;
                }
                Some(_) => {}
            }
        };
        let name =
            String::from_utf8_lossy(self.bytes.get(name_start..name_end).unwrap_or_default())
                .to_ascii_lowercase();
        let mut tag = Tag {
            name,
            attributes: Vec::new(),
        };
        if self.eof {
            return tag;
        }
        self.skip_blanks();
        if self.eof {
            return tag;
        }
        loop {
            match self.read() {
                None | Some(b'>') => break,
                Some(_) => self.back(),
            }
            let key = self.read_attribute_key();
            let value = self.read_attribute_value();
            let key = String::from_utf8_lossy(self.bytes.get(key).unwrap_or_default())
                .to_ascii_lowercase();
            let seen = tag.attributes.iter().any(|attribute| attribute.key == key);
            if save && !key.is_empty() && !seen {
                tag.attributes.push(Attribute { key, value });
            }
            self.skip_blanks();
            if self.eof {
                break;
            }
        }
        tag
    }

    fn read_attribute_key(&mut self) -> Range<usize> {
        let start = self.at;
        while let Some(c) = self.read() {
            if c == b'=' && start + 1 == self.at {
                continue;
            }
            if c == b'=' || is_blank(c) || c == b'/' || c == b'>' {
                self.back();
                return start..self.at;
            }
        }
        start..self.at
    }

    fn read_attribute_value(&mut self) -> Range<usize> {
        let mut value = self.at..self.at;
        self.skip_blanks();
        if self.eof {
            return value;
        }
        let Some(c) = self.read() else { return value };
        if c == b'/' {
            return value;
        }
        if c != b'=' {
            self.back();
            return value;
        }
        self.skip_blanks();
        if self.eof {
            return value;
        }
        let Some(quote) = self.read() else {
            return value;
        };
        match quote {
            b'>' => self.back(),
            b'\'' | b'"' => {
                value = self.at..self.at;
                loop {
                    let Some(c) = self.read() else {
                        value.end = self.at;
                        return value;
                    };
                    if c == quote {
                        value.end = self.at - 1;
                        return value;
                    }
                }
            }
            _ => {
                value = self.at - 1..self.at - 1;
                loop {
                    let Some(c) = self.read() else {
                        value.end = self.at;
                        return value;
                    };
                    if is_blank(c) {
                        value.end = self.at - 1;
                        return value;
                    }
                    if c == b'>' {
                        self.back();
                        value.end = self.at;
                        return value;
                    }
                }
            }
        }
        value
    }

    /// Reads past the next `>`, or to the end.
    fn read_until_close_angle(&mut self) {
        while let Some(c) = self.read() {
            if c == b'>' {
                return;
            }
        }
    }

    /// Reads a token whose `<!` is read: a comment, a declaration or a bogus
    /// comment.
    fn read_markup_declaration(&mut self) {
        let (Some(first), Some(second)) = (self.read(), self.read()) else {
            return;
        };
        if first == b'-' && second == b'-' {
            self.read_comment();
        } else {
            self.at -= 2;
            self.read_until_close_angle();
        }
    }

    /// Reads past the end of a comment whose `<!--` is read.
    ///
    /// A comment ends at `-->`, at `--!>`, and at `>` right after `<!--` or
    /// `<!---`.
    fn read_comment(&mut self) {
        let mut dashes = 0_usize;
        let mut beginning = true;
        while let Some(c) = self.read() {
            match c {
                b'-' => {
                    dashes += 1;
                    continue;
                }
                b'>' if dashes >= 2 || beginning => return,
                b'!' if dashes >= 2 => match self.read() {
                    None | Some(b'>') => return,
                    Some(b'-') => {
                        dashes = 1;
                        beginning = false;
                        continue;
                    }
                    Some(_) => {}
                },
                _ => {}
            }
            dashes = 0;
            beginning = false;
        }
    }
}

/// Where the text of an HTML block is read.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Place {
    /// A paragraph.
    Paragraph,
    /// A list item: an `li` is open.
    List,
    /// A block quote: a `blockquote` is open.
    Blockquote,
    /// A heading of this depth, 0 to 9.
    Heading(u8),
    /// A table header cell: a `th` is open.
    Header,
    /// A table cell: a `td` is open.
    Cell,
    /// The caption of a table: a `caption` is open.
    TableCaption,
    /// The caption of a figure: a `figcaption` is open.
    FigureCaption,
    /// A `data` element is open.
    Meta,
}

impl Place {
    /// The place of a block read under the tags `history`, the outermost
    /// first.
    fn of(history: &[String]) -> Self {
        if history.iter().any(|tag| tag == "data") {
            return Self::Meta;
        }
        for tag in history {
            let place = match tag.as_str() {
                "li" => Self::List,
                "blockquote" => Self::Blockquote,
                "th" => Self::Header,
                "td" => Self::Cell,
                "caption" => Self::TableCaption,
                "figcaption" => Self::FigureCaption,
                other => match other.as_bytes() {
                    [b'h', depth @ b'0'..=b'9'] => Self::Heading(depth - b'0'),
                    _ => continue,
                },
            };
            return place;
        }
        Self::Paragraph
    }
}

/// The inline elements that have a scope of their own.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Inline {
    /// An `a`.
    Link,
    /// A `strong` or a `b`.
    Strong,
    /// An `em` or an `i`.
    Emphasis,
    /// A `code` or a `tt`.
    Code,
}

impl Inline {
    fn of(tag: &str) -> Option<Self> {
        match tag {
            "a" => Some(Self::Link),
            "strong" | "b" => Some(Self::Strong),
            "em" | "i" => Some(Self::Emphasis),
            "code" | "tt" => Some(Self::Code),
            _ => None,
        }
    }
}

/// A run of text in a block: one text token, without the whitespace at its
/// ends.
#[derive(Debug)]
pub(crate) struct Piece {
    /// The bytes of the text in the HTML.
    pub(crate) range: Range<usize>,
    /// What reads before the text, and whether it reads as `*`.
    pub(crate) reading: Reading,
}

/// An inline element and the pieces it holds.
#[derive(Debug)]
pub(crate) struct Capture {
    pub(crate) kind: Inline,
    /// The indexes of its first piece and the one after its last.
    pub(crate) pieces: Range<usize>,
}

/// One block of text Vale lints: the text between two flushes.
#[derive(Debug)]
pub(crate) struct Block {
    pub(crate) place: Place,
    pub(crate) pieces: Vec<Piece>,
    pub(crate) captures: Vec<Capture>,
}

/// What Vale reads in an HTML block.
#[derive(Debug, Default)]
pub(crate) struct Walk {
    /// Whether the block ends the run of tags a block of text is read under:
    /// it has an end tag that is not inline, or text that is read.
    pub(crate) resets: bool,
    /// The blocks of text, in order.
    pub(crate) blocks: Vec<Block>,
    /// The alt text of each image, in order.
    pub(crate) alts: Vec<Range<usize>>,
    /// The content of each `meta` tag that describes the page, in order.
    pub(crate) descriptions: Vec<Range<usize>>,
}

/// The whitespace Vale trims from a token's text.
const fn is_blank_char(c: char) -> bool {
    matches!(c, ' ' | '\n' | '\r' | '\t' | '\x0c' | '\x0b')
}

/// The part of `raw` between its blank ends, where a reference that stands for
/// blank text, such as `&#32;`, is blank too: Vale reads the references of a
/// text before it trims it.
fn trim_blank(raw: &str) -> Range<usize> {
    let references = entity::references(raw, false);
    let blank = |text: &str| text.chars().all(is_blank_char);
    let mut start = 0;
    let mut end = raw.len();
    while start < end {
        let rest = raw.get(start..end).unwrap_or_default();
        let at = end - rest.trim_start_matches(is_blank_char).len();
        let next = references
            .iter()
            .find(|(range, text)| range.start == at && range.end <= end && blank(text));
        start = next.map_or(at, |(range, _)| range.end);
        if next.is_none() {
            break;
        }
    }
    while end > start {
        let rest = raw.get(start..end).unwrap_or_default();
        let at = start + rest.trim_end_matches(is_blank_char).len();
        let previous = references
            .iter()
            .find(|(range, text)| range.end == at && range.start >= start && blank(text));
        end = previous.map_or(at, |(range, _)| range.start);
        if previous.is_none() {
            break;
        }
    }
    start..end
}

/// The separator the source put at an end of a text: a line feed for a line
/// break, a space for any other whitespace, and none when there was none.
fn separator(c: Option<char>) -> Option<char> {
    match c? {
        '\n' | '\r' => Some('\n'),
        ' ' | '\t' => Some(' '),
        _ => None,
    }
}

/// Whether text that starts with `c` is not padded with a space before it:
/// it closes a sentence or a bracket.
const fn closes(c: char) -> bool {
    matches!(
        c,
        '.' | '?' | '!' | ',' | ':' | ';' | ')' | ']' | '}' | '—' | '–'
    )
}

/// Whether text that follows `c` is not padded with a space: `c` opens a
/// bracket or is a dash.
const fn binds_tightly(c: char) -> bool {
    matches!(c, '(' | '[' | '{' | '—' | '–')
}

/// What Vale keeps while it joins the texts of a block, one variable of its
/// walk each.
#[derive(Debug, Default)]
struct Spacer {
    /// The last start tag opened an inline element or a void one.
    inline: bool,
    /// An inline element or a comment closed since the last text.
    closed_inline: bool,
    /// The next text is masked.
    skip: bool,
    /// What the last text ended with.
    trailing: Option<char>,
    /// What the text before a comment ended with, when the comment follows
    /// text that no inline element holds.
    pad_comment: Option<char>,
}

impl Spacer {
    /// A start tag that Vale reads as the opening of an element.
    fn open(&mut self, inline_or_void: bool, masks: bool) {
        self.inline = inline_or_void;
        self.skip = masks;
        self.closed_inline = false;
        self.pad_comment = None;
    }

    /// The end tag of an inline element.
    fn close_inline(&mut self) {
        self.closed_inline = true;
        self.pad_comment = None;
    }

    /// A self-closing tag of an inline element.
    const fn self_closing_inline(&mut self) {
        self.inline = true;
        self.closed_inline = false;
    }

    /// A comment.
    const fn comment(&mut self) {
        if !self.inline {
            self.closed_inline = true;
            self.pad_comment = self.trailing;
        }
    }

    /// Whether the tag, comment or declaration `html` changes how the text
    /// around it joins: an inline or void tag, or a comment.
    fn joins(html: &str) -> bool {
        match Tokenizer::new(html).read_token() {
            Token::Start(tag) => {
                INLINE_TAGS.contains(&tag.name.as_str()) || VOID_TAGS.contains(&tag.name.as_str())
            }
            Token::End(tag) | Token::SelfClosing(tag) => INLINE_TAGS.contains(&tag.name.as_str()),
            Token::Comment(_) => true,
            Token::Text(_) | Token::Eof => false,
        }
    }

    /// A tag, comment or declaration, written as `html`.
    fn tag(&mut self, html: &str) {
        match Tokenizer::new(html).read_token() {
            Token::Start(tag) => {
                let name = tag.name.as_str();
                let inline = INLINE_TAGS.contains(&name) || VOID_TAGS.contains(&name);
                self.open(
                    inline,
                    MASKED_TAGS.contains(&name) || SKIP_TAGS.contains(&name),
                );
            }
            Token::End(tag) => {
                if INLINE_TAGS.contains(&tag.name.as_str()) {
                    self.close_inline();
                }
            }
            Token::SelfClosing(tag) => {
                if INLINE_TAGS.contains(&tag.name.as_str()) {
                    self.self_closing_inline();
                }
            }
            Token::Comment(_) => self.comment(),
            Token::Text(_) | Token::Eof => {}
        }
    }

    /// A text that continues the one before it.
    fn continue_text(&mut self, text: &str) {
        self.trailing = separator(text.chars().next_back());
    }

    /// A text token `decoded`, as the tokenizer gives it.
    ///
    /// `last` is the last character of the block so far, `class_skipped` is
    /// whether the element before the text has a class that Vale leaves out,
    /// and `is_href` whether the text is the address of the link before it.
    /// Returns how the text reads, or `None` when it is blank.
    fn text(
        &mut self,
        decoded: &str,
        last: Option<char>,
        class_skipped: bool,
        is_href: bool,
    ) -> Option<Reading> {
        let body = decoded.trim_matches(is_blank_char);
        if body.is_empty() {
            self.trailing = separator(decoded.chars().next_back());
            return None;
        }
        let mut sep = separator(decoded.chars().next());
        if self.closed_inline && sep.is_none() {
            sep = self.pad_comment;
        }
        self.pad_comment = None;
        let masking = self.skip || class_skipped;
        let starter = !masking && body.chars().next().is_some_and(closes);
        let mut lead = if self.closed_inline && sep.is_some() {
            sep
        } else if !self.closed_inline && self.inline && !starter {
            Some(' ')
        } else {
            None
        };
        self.closed_inline = false;
        self.skip = false;
        if lead == Some(' ') {
            let tight = self.trailing.is_none() && last.is_some_and(binds_tightly);
            let after_space = last.is_some_and(|c| !c.is_ascii() && c.is_whitespace());
            if tight || after_space {
                lead = None;
            }
        }
        self.trailing = separator(decoded.chars().next_back());
        Some(Reading {
            before: match lead {
                Some('\n') => Join::Newline,
                Some(_) => Join::Space,
                None => Join::Nothing,
            },
            masked: masking || is_href,
        })
    }
}

/// An inline element that is still open.
struct Open {
    kind: Inline,
    tag: String,
    /// The index of the first piece after its start tag.
    first: usize,
}

/// Which tag a token is.
#[derive(Clone, Copy, Eq, PartialEq)]
enum Kind {
    Start,
    SelfClosing,
    End,
}

/// What Vale keeps while it walks the tokens of one block.
///
/// The flags are Vale's own variables, each set and read where Vale does.
#[derive(Default)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "Each flag is one variable of Vale's walk, which the code follows."
)]
struct Walker {
    /// The tags open around the text, the outermost first.
    history: Vec<String>,
    pieces: Vec<Piece>,
    /// Whether the text so far is more than blanks.
    nonblank: bool,
    /// The last character of the text so far.
    last: Option<char>,
    open: Vec<Open>,
    captures: Vec<Capture>,
    spacer: Spacer,
    /// The next text is of a `code` or a `tt`, which reads as a code span.
    code_next: bool,
    /// A skipped element is open.
    in_block: bool,
    /// The element of a skipped class that is open, its name and the number
    /// of its start and end tags seen.
    class_block: Option<(String, usize, usize)>,
    /// The block elements that are open, and whether each has a skipped class.
    enclosing: Vec<(String, bool)>,
    /// How many elements of each name opened inside an inline element, which
    /// Vale reads as inline.
    promoted: HashMap<String, usize>,
    /// The class of the token before had a skipped class.
    previous_skipped: bool,
    /// The `href` of the token before.
    previous_href: String,
    walk: Walk,
}

/// Walks the tokens of `html`, an HTML block of a Markdown file.
pub(crate) fn walk(html: &str) -> Walk {
    let mut tokenizer = Tokenizer::new(html);
    let mut walker = Walker::default();
    loop {
        let token = tokenizer.read_token();
        let done = matches!(token, Token::Eof);
        walker.token(html, token);
        if done {
            break;
        }
    }
    walker.walk
}

/// The comments in `html`, as the byte range of each from `<!--` to the end
/// of its last `>`.
///
/// A declaration, a processing instruction and `</>` are not comments.
pub(crate) fn comments(html: &str) -> Vec<Range<usize>> {
    let mut tokenizer = Tokenizer::new(html);
    let mut found = Vec::new();
    loop {
        match tokenizer.read_token() {
            Token::Eof => break,
            Token::Comment(range)
                if html
                    .get(range.clone())
                    .is_some_and(|text| text.starts_with("<!--")) =>
            {
                found.push(range);
            }
            Token::Text(_)
            | Token::Start(_)
            | Token::SelfClosing(_)
            | Token::End(_)
            | Token::Comment(_) => {}
        }
    }
    found
}

/// Whether the HTML in `html` ends the run of tags a block of text is read
/// under.
pub(crate) fn resets_scope(html: &str) -> bool {
    walk(html).resets
}

impl Walker {
    fn token(&mut self, html: &str, token: Token) {
        match token {
            Token::Eof => self.flush(),
            Token::Text(range) => self.text(html, range),
            Token::Comment(_) => {
                self.spacer.comment();
                self.previous_skipped = false;
                self.previous_href.clear();
            }
            Token::Start(tag) => self.tag(html, &tag, Kind::Start),
            Token::SelfClosing(tag) => self.tag(html, &tag, Kind::SelfClosing),
            Token::End(tag) => self.tag(html, &tag, Kind::End),
        }
    }

    /// Whether an inline element is in the tags open around the text.
    fn inside_inline(&self) -> bool {
        self.history
            .iter()
            .any(|tag| INLINE_TAGS.contains(&tag.as_str()))
    }

    fn tag(&mut self, html: &str, tag: &Tag, kind: Kind) {
        let name = tag.name.as_str();
        let mut inline = INLINE_TAGS.contains(&name);
        let void = VOID_TAGS.contains(&name);
        if kind == Kind::Start && !inline && !void && self.inside_inline() {
            *self.promoted.entry(tag.name.clone()).or_default() += 1;
            inline = true;
        } else if kind == Kind::End
            && !inline
            && let Some(count) = self.promoted.get_mut(name)
            && *count > 0
        {
            *count -= 1;
            inline = true;
        }
        if let Some((class_tag, begin, end)) = &mut self.class_block
            && class_tag == name
        {
            if kind == Kind::Start {
                *begin += 1;
            } else {
                *end += 1;
            }
        }
        let closes_class = self
            .class_block
            .as_ref()
            .is_some_and(|(_, begin, end)| begin == end && *begin > 0);
        let skipped_class = tag.skipped_class(html);
        if kind == Kind::Start && !inline && !void {
            self.enclosing.push((tag.name.clone(), skipped_class));
        } else if kind == Kind::End
            && !inline
            && let Some(at) = self.enclosing.iter().rposition(|(open, _)| open == name)
        {
            self.enclosing.remove(at);
        }
        let block_skip = skipped_class && !inline;
        let skips = SKIP_TAGS.contains(&name);
        if kind == Kind::Start && !inline && (skips || block_skip) {
            if block_skip {
                self.class_block = Some((tag.name.clone(), 1, 0));
            }
            self.in_block = true;
        } else if self.in_block && (closes_class || kind == Kind::End && !inline && skips) {
            self.in_block = false;
            if closes_class {
                self.class_block = None;
            }
        } else if kind == Kind::Start {
            self.start(tag, inline, void);
        } else if kind == Kind::End && inline {
            self.end_inline(name);
        } else if kind == Kind::SelfClosing && inline {
            self.spacer.self_closing_inline();
        }
        if kind == Kind::End && !inline {
            self.walk.resets = true;
            self.flush();
            self.reset();
        }
        self.previous_skipped = skipped_class;
        self.previous_href = tag
            .attributes
            .iter()
            .find(|attribute| attribute.key == "href")
            .and_then(|attribute| html.get(attribute.value.clone()))
            .map(|href| entity::unescape(href, true).into_owned())
            .unwrap_or_default();
        if !skipped_class && !self.enclosing.iter().any(|(_, skipped)| *skipped) {
            self.attributes(html, tag);
        }
    }

    /// A start tag that no skip reads past.
    fn start(&mut self, tag: &Tag, inline: bool, void: bool) {
        if !inline && !void && self.nonblank {
            self.flush();
            self.reset();
        }
        // Vale reads an `em` or a `b` in a `code` as a `code`.
        let in_code = self.history.last().is_some_and(|last| last == "code");
        let name = if matches!(tag.name.as_str(), "em" | "b") && in_code {
            "code"
        } else {
            tag.name.as_str()
        };
        self.spacer.open(
            inline || void,
            MASKED_TAGS.contains(&name) || SKIP_TAGS.contains(&name),
        );
        self.code_next = CODE_TAGS.contains(&name);
        if let Some(kind) = Inline::of(name) {
            self.open.push(Open {
                kind,
                tag: name.to_owned(),
                first: self.pieces.len(),
            });
        }
        self.history.push(name.to_owned());
    }

    /// The end tag of an inline element, or of one read as inline.
    fn end_inline(&mut self, name: &str) {
        if self.history.last().is_some_and(|last| last == name) {
            self.history.pop();
        }
        self.spacer.close_inline();
        if self.open.last().is_some_and(|open| open.tag == name)
            && let Some(open) = self.open.pop()
        {
            self.captures.push(Capture {
                kind: open.kind,
                pieces: open.first..self.pieces.len(),
            });
        }
    }

    /// Records the alt text of an image and the description of a page.
    fn attributes(&mut self, html: &str, tag: &Tag) {
        let value = |key: &str| {
            tag.attributes
                .iter()
                .find(|attribute| attribute.key == key)
                .map(|attribute| (attribute.value.clone(), html.get(attribute.value.clone())))
        };
        match tag.name.as_str() {
            "img" => {
                if let Some((range, Some(alt))) = value("alt")
                    && !alt.is_empty()
                {
                    self.walk.alts.push(range);
                }
            }
            "meta" => {
                let describes = value("name").is_some_and(|(_, name)| name == Some("description"))
                    || value("property").is_some_and(|(_, name)| name == Some("og:description"));
                if describes
                    && let Some((range, Some(content))) = value("content")
                    && !content.is_empty()
                {
                    self.walk.descriptions.push(range);
                }
            }
            _ => {}
        }
    }

    fn text(&mut self, html: &str, range: Range<usize>) {
        let class_skipped = self.previous_skipped;
        let href = std::mem::take(&mut self.previous_href);
        self.previous_skipped = false;
        if self.in_block {
            return;
        }
        let raw = html.get(range.clone()).unwrap_or_default();
        let decoded = entity::unescape(raw, false);
        let body = decoded.trim_matches(is_blank_char);
        let Some(reading) = self.spacer.text(
            &decoded,
            self.last,
            class_skipped,
            !href.is_empty() && href == body,
        ) else {
            return;
        };
        let code = std::mem::take(&mut self.code_next);
        self.walk.resets = true;
        self.nonblank |= reading.masked || body.chars().any(|c| !c.is_whitespace());
        self.last = if reading.masked {
            Some('*')
        } else {
            body.chars().next_back()
        };
        let kept = trim_blank(raw);
        self.pieces.push(Piece {
            range: range.start + kept.start..range.start + kept.end,
            // The text of a code span reads as `x`, and not as `*`.
            reading: Reading {
                masked: reading.masked && !code,
                ..reading
            },
        });
    }

    /// Gives the text so far to the walk, under the tags that are open.
    fn flush(&mut self) {
        if !self.nonblank {
            return;
        }
        self.walk.blocks.push(Block {
            place: Place::of(&self.history),
            pieces: std::mem::take(&mut self.pieces),
            captures: std::mem::take(&mut self.captures),
        });
    }

    /// Forgets the text and the tags open around it.
    fn reset(&mut self) {
        self.history.clear();
        self.pieces.clear();
        self.captures.clear();
        self.open.clear();
        self.nonblank = false;
        self.last = None;
    }
}

/// How the segments of a Markdown paragraph read, where HTML tags lie between
/// them.
///
/// `start` is where the node starts, `segments` its text, and `tags` the spans
/// of its inline HTML, each one tag or comment, all in `source`.
/// Vale joins the text after a tag as it joins the text of an HTML block, and
/// masks the text of a `kbd` or a `samp`.
/// It trims each text between two tags, so a segment that touches a tag loses
/// its whitespace there, and one that holds only whitespace is dropped.
/// Returns the segments, and a reading for each, or no reading when no tag lies
/// between two segments.
pub(crate) fn inline_reads(
    source: &str,
    start: usize,
    segments: Vec<Range<usize>>,
    tags: &[Range<usize>],
) -> (Vec<Range<usize>>, Vec<Reading>) {
    // A tag that opens a block ends the text in Vale, and hakkiri reads the
    // text around it as it did before: one text, with the tag a gap.
    let tags: Vec<Range<usize>> = tags
        .iter()
        .filter(|tag| source.get((*tag).clone()).is_some_and(Spacer::joins))
        .cloned()
        .collect();
    let mut spacer = Spacer::default();
    let mut kept = Vec::with_capacity(segments.len());
    let mut reads = Vec::with_capacity(segments.len());
    let mut last: Option<char> = None;
    let mut from = start;
    let mut tagged = false;
    let mut masked_run = false;
    for (index, segment) in segments.iter().enumerate() {
        let text = source.get(segment.clone()).unwrap_or_default();
        let before: Vec<&Range<usize>> = tags
            .iter()
            .filter(|tag| from <= tag.start && tag.end <= segment.start)
            .collect();
        let until = segments
            .get(index + 1)
            .map_or(usize::MAX, |next| next.start);
        let tag_after = tags
            .iter()
            .any(|tag| segment.end <= tag.start && tag.end <= until);
        let mut range = segment.clone();
        let reading = if before.is_empty() {
            spacer.continue_text(text);
            let one_text = source
                .get(from..segment.start)
                .is_some_and(|gap| gap.chars().all(|c| c.is_whitespace() || c == '>'));
            Reading {
                before: Join::Gap,
                masked: masked_run && one_text,
            }
        } else {
            tagged = true;
            for tag in before {
                if let Some(html) = source.get(tag.clone()) {
                    spacer.tag(html);
                }
            }
            range.start += text.len() - text.trim_start_matches(is_blank_char).len();
            let decoded = entity::unescape(text, false);
            spacer
                .text(&decoded, last, false, false)
                .unwrap_or_default()
        };
        if tag_after {
            tagged = true;
            let trimmed = text.trim_end_matches(is_blank_char).len();
            range.end = range.end.min(segment.start + trimmed);
        }
        from = segment.end;
        if range.start >= range.end {
            continue;
        }
        masked_run = reading.masked;
        last = if reading.masked {
            Some('*')
        } else {
            source
                .get(range.clone())
                .and_then(|kept| kept.chars().next_back())
        };
        kept.push(range);
        reads.push(reading);
    }
    if tagged {
        (kept, reads)
    } else {
        (segments, Vec::new())
    }
}
