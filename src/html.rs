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
//! The text of `script`, `style`, `noscript`, `iframe` and `pre` is left out,
//! as is the text of an element whose class is `code`, `pre` or `problematic`.
//! The first text after a `tt`, `code`, `kbd` or `samp` start tag is masked.
//! hakkiri reads the text of `tt` and `code` as a code span, which a rule reads
//! as one word, and leaves out the text of `kbd` and `samp`.
//!
//! This module follows [that tokenizer's rules][tokenizer] for where a tag
//! ends and what is text, so a `>` in a quoted attribute value, or inside a
//! bogus comment such as `<?php echo "<div>"; ?>`, is read as it reads it.
//! It does not follow the tokenizer's escaped states for the content of a
//! `script` element: that content ends at the first `</script`.
//!
//! [tokenizer]: https://github.com/golang/net/blob/v0.62.0/html/token.go

use std::ops::Range;

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

/// The tags whose first text is masked, and read as nothing.
/// Vale also masks the text of `tt` and `code`, which hakkiri reads as a code
/// span.
const MASKED_TAGS: [&str; 2] = ["kbd", "samp"];

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

const fn is_blank(c: u8) -> bool {
    matches!(c, b' ' | b'\n' | b'\r' | b'\t' | 0x0c | 0x0b)
}

/// Whether `text` holds nothing but the whitespace Vale trims.
fn blank(text: &str) -> bool {
    text.bytes().all(is_blank)
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
    /// A comment, a declaration, a processing instruction or `</>`.
    Comment,
    /// The end of the input, or a tag that it ends.
    Eof,
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
                    return Token::Comment;
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
                return Token::Comment;
            }
            if c == b'!' {
                self.read_markup_declaration();
            } else {
                self.back();
                self.read_until_close_angle();
            }
            return Token::Comment;
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
    /// A heading of this depth, 1 to 6.
    Heading(u8),
    /// A table header cell: a `th` is open.
    Header,
    /// A table cell: a `td` is open.
    Cell,
    /// A scope hakkiri has no node for, such as a caption: the text is not
    /// read.
    Skipped,
}

impl Place {
    /// The place of a block read under the tags `history`, the outermost
    /// first.
    fn of(history: &[String]) -> Self {
        if history.iter().any(|tag| tag == "data") {
            return Self::Skipped;
        }
        for tag in history {
            let place = match tag.as_str() {
                "li" => Self::List,
                "blockquote" => Self::Blockquote,
                "th" => Self::Header,
                "td" => Self::Cell,
                "caption" | "figcaption" => Self::Skipped,
                other => match other.as_bytes() {
                    [b'h', depth @ b'0'..=b'9'] => match depth {
                        b'1'..=b'6' => Self::Heading(depth - b'0'),
                        _ => Self::Skipped,
                    },
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

/// A run of text in a block.
#[derive(Debug)]
pub(crate) struct Piece {
    /// The bytes of the text token in the HTML.
    pub(crate) range: Range<usize>,
    /// Vale masks this text, so it is not read.
    pub(crate) hidden: bool,
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
}

/// An inline element that is still open.
struct Open {
    kind: Inline,
    tag: String,
    /// The index of the first piece after its start tag.
    first: usize,
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
    open: Vec<Open>,
    captures: Vec<Capture>,
    /// A skipped element is open.
    in_block: bool,
    /// The element of a skipped class that is open, its name and the number
    /// of its start and end tags seen.
    class_block: Option<(String, usize, usize)>,
    /// The block elements that are open, and whether each has a skipped class.
    enclosing: Vec<(String, bool)>,
    /// The next text is masked.
    mask_next: bool,
    /// The class of the token before had a skipped class.
    previous_skipped: bool,
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
            Token::Comment => self.previous_skipped = false,
            Token::Start(tag) => self.start(html, &tag, false),
            Token::SelfClosing(tag) => self.start(html, &tag, true),
            Token::End(tag) => self.end(html, &tag),
        }
    }

    /// Records the alt text of an image.
    fn image(&mut self, html: &str, tag: &Tag, skipped_class: bool) {
        if tag.name != "img" || skipped_class || self.enclosing.iter().any(|(_, skipped)| *skipped)
        {
            return;
        }
        if let Some(alt) = tag
            .attributes
            .iter()
            .find(|attribute| attribute.key == "alt")
            && html.get(alt.value.clone()).is_some_and(|text| !blank(text))
        {
            self.walk.alts.push(alt.value.clone());
        }
    }

    fn start(&mut self, html: &str, tag: &Tag, self_closing: bool) {
        let name = tag.name.as_str();
        let inline = INLINE_TAGS.contains(&name);
        let void = VOID_TAGS.contains(&name);
        let skipped_class = tag.skipped_class(html);
        if let Some((class_tag, begin, end)) = &mut self.class_block
            && class_tag == name
        {
            if self_closing {
                *end += 1;
            } else {
                *begin += 1;
            }
        }
        let closes_class = self
            .class_block
            .as_ref()
            .is_some_and(|(_, begin, end)| begin == end && *begin > 0);
        self.image(html, tag, skipped_class);
        self.previous_skipped = skipped_class;
        if self_closing {
            // A self-closing tag is no start tag, and no end tag either.
            if self.in_block && closes_class {
                self.in_block = false;
                self.class_block = None;
            }
            return;
        }
        if !inline && !void {
            self.enclosing.push((tag.name.clone(), skipped_class));
        }
        let block_skip = skipped_class && !inline;
        if !inline && (SKIP_TAGS.contains(&name) || block_skip) {
            if block_skip {
                self.class_block = Some((tag.name.clone(), 1, 0));
            }
            self.in_block = true;
            return;
        }
        if self.in_block && closes_class {
            self.in_block = false;
            self.class_block = None;
        }
        if !inline && !void && self.nonblank {
            self.flush();
            self.reset();
        }
        self.mask_next = MASKED_TAGS.contains(&name);
        if let Some(kind) = Inline::of(name) {
            self.open.push(Open {
                kind,
                tag: tag.name.clone(),
                first: self.pieces.len(),
            });
        }
        self.history.push(tag.name.clone());
    }

    fn end(&mut self, html: &str, tag: &Tag) {
        let name = tag.name.as_str();
        let inline = INLINE_TAGS.contains(&name);
        if let Some((class_tag, _, end)) = &mut self.class_block
            && class_tag == name
        {
            *end += 1;
        }
        let closes_class = self
            .class_block
            .as_ref()
            .is_some_and(|(_, begin, end)| begin == end && *begin > 0);
        if !inline && let Some(at) = self.enclosing.iter().rposition(|(open, _)| open == name) {
            self.enclosing.remove(at);
        }
        self.previous_skipped = tag.skipped_class(html);
        if self.in_block && (closes_class || !inline && SKIP_TAGS.contains(&name)) {
            self.in_block = false;
            if closes_class {
                self.class_block = None;
            }
        } else if inline {
            if self.history.last().is_some_and(|last| last == name) {
                self.history.pop();
            }
            if self.open.last().is_some_and(|open| open.tag == name)
                && let Some(open) = self.open.pop()
            {
                self.captures.push(Capture {
                    kind: open.kind,
                    pieces: open.first..self.pieces.len(),
                });
            }
        }
        if !inline {
            self.walk.resets = true;
            self.flush();
            self.reset();
        }
    }

    fn text(&mut self, html: &str, range: Range<usize>) {
        let text = html.get(range.clone()).unwrap_or_default();
        let skipped = self.previous_skipped;
        self.previous_skipped = false;
        if self.in_block {
            return;
        }
        if blank(text) {
            self.pieces.push(Piece {
                range,
                hidden: false,
            });
            return;
        }
        let hidden = self.mask_next || skipped;
        self.mask_next = false;
        self.nonblank = true;
        self.walk.resets = true;
        self.pieces.push(Piece { range, hidden });
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
    }
}
