//! Whether an HTML block ends the run of tags a block of text is read under.
//!
//! Vale reads a block's scope off the tags opened since the last reset.
//! It walks the tokens of `golang.org/x/net/html` at v0.62.0 and resets that
//! run at an end tag that is not inline, and at the next block tag after
//! any text that is not blank.
//! An HTML block that has either one is the first block of its quote or item,
//! and the paragraph after it is a `paragraph`.
//! A start tag alone, a comment, a declaration and a processing instruction
//! reset nothing.
//! This follows [that tokenizer's rules][tokenizer] for where a tag ends and
//! what is text, so a `>` in a quoted attribute value, or inside a bogus
//! comment such as `<?php echo "<div>"; ?>`, is read as it reads it.
//!
//! [tokenizer]: https://github.com/golang/net/blob/v0.62.0/html/token.go

/// The inline tags: an end tag of one resets nothing.
const INLINE_TAGS: [&str; 23] = [
    "b", "big", "i", "small", "abbr", "acronym", "cite", "dfn", "em", "kbd", "strong", "a", "br",
    "img", "span", "sub", "sup", "code", "tt", "del", "q", "samp", "var",
];

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

/// The raw text tags whose text Vale leaves out of a block.
const SKIPPED_TAGS: [&str; 4] = ["script", "style", "noscript", "iframe"];

/// Whether the HTML in `html` has an end tag that is not inline, or text that
/// is not blank.
///
/// A block with an unfinished tag ends the walk there, as an error token does.
pub(crate) fn resets_scope(html: &str) -> bool {
    let mut scan = Scan {
        bytes: html.as_bytes(),
        at: 0,
    };
    while let Some(c) = scan.next() {
        if c != b'<' {
            if is_blank(c) {
                continue;
            }
            return true;
        }
        let Some(c) = scan.next() else {
            return true;
        };
        match c {
            b'a'..=b'z' | b'A'..=b'Z' => {
                let Some(name) = scan.tag() else {
                    return false;
                };
                if RAW_TEXT_TAGS.contains(&name.as_str()) {
                    return scan.raw_text_resets(&name);
                }
            }
            b'/' => match scan.next() {
                None => return true,
                Some(b'>') => {}
                Some(letter) if letter.is_ascii_alphabetic() => match scan.tag() {
                    Some(name) if !INLINE_TAGS.contains(&name.as_str()) => return true,
                    Some(_) => {}
                    None => return false,
                },
                Some(_) => {
                    scan.back();
                    scan.past_angle();
                }
            },
            b'?' => {
                scan.back();
                scan.past_angle();
            }
            b'!' => {
                if scan
                    .bytes
                    .get(scan.at..)
                    .is_some_and(|rest| rest.starts_with(b"--"))
                {
                    scan.at += 2;
                    scan.past_comment();
                } else {
                    scan.past_angle();
                }
            }
            _ => return true,
        }
    }
    false
}

/// A reader of bytes that can step back one.
struct Scan<'a> {
    bytes: &'a [u8],
    at: usize,
}

const fn is_blank(c: u8) -> bool {
    matches!(c, b' ' | b'\n' | b'\r' | b'\t' | 0x0c)
}

impl Scan<'_> {
    fn next(&mut self) -> Option<u8> {
        let c = *self.bytes.get(self.at)?;
        self.at += 1;
        Some(c)
    }

    const fn back(&mut self) {
        self.at -= 1;
    }

    const fn done(&self) -> bool {
        self.at >= self.bytes.len()
    }

    fn skip_blanks(&mut self) {
        while self.bytes.get(self.at).copied().is_some_and(is_blank) {
            self.at += 1;
        }
    }

    /// Reads past the next `>`, or to the end: a declaration, a processing
    /// instruction or other bogus comment ends there.
    fn past_angle(&mut self) {
        while let Some(c) = self.next() {
            if c == b'>' {
                return;
            }
        }
    }

    /// Reads past the end of a comment whose `<!--` is read.
    ///
    /// A comment ends at `-->`, at `--!>`, and at `>` right after `<!--` or
    /// `<!---`.
    fn past_comment(&mut self) {
        let mut dashes = 0_usize;
        let mut beginning = true;
        while let Some(c) = self.next() {
            match c {
                b'-' => {
                    dashes += 1;
                    continue;
                }
                b'>' if dashes >= 2 || beginning => return,
                b'!' if dashes >= 2 => match self.next() {
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

    /// Reads a tag whose first letter is read, and gives its name in
    /// lowercase, or `None` at the end of the input before its `>`.
    fn tag(&mut self) -> Option<String> {
        let first = self.at - 1;
        let end = loop {
            match self.next() {
                None => return None,
                Some(c) if is_blank(c) => break self.at - 1,
                Some(b'/' | b'>') => {
                    self.back();
                    break self.at;
                }
                Some(_) => {}
            }
        };
        let name = String::from_utf8_lossy(self.bytes.get(first..end)?).to_ascii_lowercase();
        self.skip_blanks();
        if self.done() {
            return None;
        }
        loop {
            match self.next() {
                None => return None,
                Some(b'>') => return Some(name),
                Some(_) => self.back(),
            }
            self.attribute_key();
            self.attribute_value();
            self.skip_blanks();
            if self.done() {
                return None;
            }
        }
    }

    /// Whether the content of a raw text tag `name`, which is read, has an
    /// end tag, or text that Vale keeps.
    fn raw_text_resets(&self, name: &str) -> bool {
        let rest = self.bytes.get(self.at..).unwrap_or_default();
        let closed = rest
            .windows(name.len() + 2)
            .enumerate()
            .any(|(at, window)| {
                window.starts_with(b"</")
                    && window
                        .get(2..)
                        .is_some_and(|tag| tag.eq_ignore_ascii_case(name.as_bytes()))
                    && rest
                        .get(at + name.len() + 2)
                        .is_some_and(|c| is_blank(*c) || *c == b'/' || *c == b'>')
            });
        closed || (!SKIPPED_TAGS.contains(&name) && rest.iter().any(|c| !is_blank(*c)))
    }

    /// Reads an attribute name.
    fn attribute_key(&mut self) {
        let start = self.at;
        while let Some(c) = self.next() {
            if c == b'=' && start + 1 == self.at {
                continue;
            }
            if c == b'=' || is_blank(c) || c == b'/' || c == b'>' {
                self.back();
                return;
            }
        }
    }

    /// Reads an attribute value: quoted, or ended by a blank or a `>`.
    fn attribute_value(&mut self) {
        self.skip_blanks();
        let Some(c) = self.next() else {
            return;
        };
        if c == b'/' {
            return;
        }
        if c != b'=' {
            self.back();
            return;
        }
        self.skip_blanks();
        let Some(quote) = self.next() else {
            return;
        };
        match quote {
            b'>' => self.back(),
            b'\'' | b'"' => while self.next().is_some_and(|c| c != quote) {},
            _ => {
                while let Some(c) = self.next() {
                    if is_blank(c) {
                        return;
                    }
                    if c == b'>' {
                        self.back();
                        return;
                    }
                }
            }
        }
    }
}
