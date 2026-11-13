//! The affix file of a Hunspell dictionary, and the words an entry makes with
//! it.
//!
//! A port of `aff.go` in [Vale]'s `spell` package at `66bbb85`, without the
//! directives for compounds made by flag, for suggestions and for Turkic
//! casing.
//!
//! [Vale]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/spell/aff.go

use std::cmp::Reverse;
use std::collections::HashMap;

use super::DictionaryError;

/// How many times a continuation class may be followed.
///
/// Hunspell follows one. This follows two, for dictionaries that chain more,
/// and the bound keeps a class that names itself from running on.
const MAX_AFFIX_DEPTH: usize = 2;

/// A flag, as the bytes it is written with.
pub(super) type Flag = Vec<u8>;

/// Whether an affix goes before a word or after it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Kind {
    /// An affix before the word.
    Prefix,
    /// An affix after the word.
    Suffix,
}

/// How the flags of an entry are written, from the `FLAG` directive.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FlagType {
    /// One character each, or one byte each in a UTF-8 file.
    Default,
    /// Numbers, separated by commas.
    Number,
    /// Two ASCII characters each.
    Long,
    /// One UTF-8 character each.
    Utf8,
}

/// One part of an affix condition: a character, a class, or any character.
#[derive(Clone, Debug)]
enum Atom {
    /// Any character.
    Any,
    /// One of the characters.
    OneOf(Vec<char>),
    /// None of the characters.
    NoneOf(Vec<char>),
}

impl Atom {
    fn accepts(&self, c: char) -> bool {
        match self {
            Self::Any => true,
            Self::OneOf(set) => set.contains(&c),
            Self::NoneOf(set) => !set.contains(&c),
        }
    }
}

/// The condition of an affix rule on the word it applies to: Hunspell's
/// characters, classes and dots, read at the start of a word for a prefix and
/// at its end for a suffix.
#[derive(Clone, Debug)]
struct Condition(Vec<Atom>);

impl Condition {
    /// Reads a condition, or `None` when a class is not closed.
    fn parse(text: &str) -> Option<Self> {
        let mut atoms = Vec::new();
        let mut chars = text.chars();
        while let Some(c) = chars.next() {
            match c {
                '.' => atoms.push(Atom::Any),
                '[' => {
                    let mut set = Vec::new();
                    let mut closed = false;
                    for member in chars.by_ref() {
                        if member == ']' {
                            closed = true;
                            break;
                        }
                        set.push(member);
                    }
                    if !closed {
                        return None;
                    }
                    if set.first() == Some(&'^') {
                        set.remove(0);
                        atoms.push(Atom::NoneOf(set));
                    } else {
                        atoms.push(Atom::OneOf(set));
                    }
                }
                other => atoms.push(Atom::OneOf(vec![other])),
            }
        }
        Some(Self(atoms))
    }

    /// Whether `word` meets the condition at the start or the end.
    fn holds(&self, word: &str, kind: Kind) -> bool {
        let count = self.0.len();
        let chars: Vec<char> = word.chars().collect();
        let Some(skip) = chars.len().checked_sub(count) else {
            return false;
        };
        let window = match kind {
            Kind::Prefix => chars.get(..count),
            Kind::Suffix => chars.get(skip..),
        };
        window.is_some_and(|window| window.iter().zip(&self.0).all(|(c, atom)| atom.accepts(*c)))
    }
}

/// One rule of an affix class.
#[derive(Clone, Debug)]
struct Rule {
    /// The text taken off the word first.
    strip: String,
    /// The text added.
    text: String,
    /// The flags the rule carries, which name classes that apply again to the
    /// word it makes.
    cont: Vec<Flag>,
    /// What the word must be like, when the rule has a condition.
    condition: Option<Condition>,
}

/// An affix class: the rules a flag stands for.
#[derive(Clone, Debug)]
struct Class {
    kind: Kind,
    /// Whether a prefix and a suffix of this kind may both apply.
    cross: bool,
    rules: Vec<Rule>,
}

/// A word an entry makes, with the flags it carries.
#[derive(Clone, Debug)]
pub(super) struct Form {
    /// The word.
    pub(super) word: String,
    /// The flags of the stem and of every rule applied.
    pub(super) flags: Vec<Flag>,
    /// The flags of the rule that made the word.
    cont: Vec<Flag>,
    /// `NEEDAFFIX` is on the stem or a rule.
    wants_affix: bool,
    /// A rule without `NEEDAFFIX` was applied, which meets the want.
    plain_affix: bool,
    /// A `CIRCUMFIX` rule waits for its partner of the other kind.
    open: Option<Kind>,
}

impl Form {
    /// Whether the word is only a step toward another word.
    fn is_step(&self) -> bool {
        self.open.is_some() || (self.wants_affix && !self.plain_affix)
    }
}

/// An affix file, as far as this reader goes.
#[derive(Clone, Debug)]
pub(crate) struct AffixFile {
    flag_type: FlagType,
    /// A file that declares `SET UTF-8` has flags of one byte unless `FLAG`
    /// says otherwise. In any other file a flag is one character, so `é` and
    /// `è` are two flags in a Latin-1 file.
    byte_flags: bool,
    /// The flag sets that an entry names by number, from `AF`.
    aliases: Vec<String>,
    classes: HashMap<Flag, Class>,
    /// The `ICONV` pairs, the longest first.
    conversions: Vec<(String, String)>,
    /// The `COMPOUNDRULE` patterns.
    pub(super) compound_rules: Vec<String>,
    /// The `BREAK` patterns.
    pub(super) breaks: Vec<String>,
    /// Whether any `BREAK` line was there.
    pub(super) breaks_declared: bool,
    /// `ONLYINCOMPOUND`: a word only in compounds.
    pub(super) only_in_compound: Option<Flag>,
    /// `FORBIDDENWORD`: the forms of the entry are not words.
    pub(super) forbidden: Option<Flag>,
    /// `NEEDAFFIX`: the bare stem is not a word.
    need_affix: Option<Flag>,
    /// `KEEPCASE`: a word only as written.
    pub(super) keep_case: Option<Flag>,
    /// `CIRCUMFIX`: a prefix and a suffix that go together.
    circumfix: Option<Flag>,
    /// `IGNORE`: characters dropped from every word.
    ignore_chars: Vec<char>,
}

/// What reading the lines of an affix file keeps between lines.
#[derive(Default)]
struct Reading {
    /// The first `BREAK` line is a count.
    saw_break_count: bool,
    /// The first `AF` line is a count.
    saw_alias_count: bool,
    /// The flags of each rule, as written, until the whole file is read.
    written: Vec<(Flag, usize, String)>,
}

/// A directive that is too short, as a [`DictionaryError`].
fn short(directive: &str, line: usize) -> DictionaryError {
    DictionaryError::Directive {
        directive: directive.to_owned(),
        line,
    }
}

/// Whether `text` is one or more ASCII digits.
fn all_digits(text: &str) -> bool {
    !text.is_empty() && text.chars().all(|c| c.is_ascii_digit())
}

/// Whether `flag` is set and among `flags`.
pub(super) fn has_flag(flags: &[Flag], flag: Option<&Flag>) -> bool {
    flag.is_some_and(|flag| !flag.is_empty() && flags.contains(flag))
}

/// `first`, then what `second` adds to it.
fn union(first: &[Flag], second: &[Flag]) -> Vec<Flag> {
    let mut out = first.to_vec();
    for flag in second {
        if !out.contains(flag) {
            out.push(flag.clone());
        }
    }
    out
}

impl AffixFile {
    /// Reads the text of an affix file, already in UTF-8.
    ///
    /// # Errors
    ///
    /// Returns a [`DictionaryError`] for a directive that lacks its argument,
    /// an affix class that is neither `Y` nor `N`, and a condition that does
    /// not read.
    pub(crate) fn parse(text: &str) -> Result<Self, DictionaryError> {
        let mut file = Self {
            flag_type: FlagType::Default,
            byte_flags: false,
            aliases: Vec::new(),
            classes: HashMap::new(),
            conversions: Vec::new(),
            compound_rules: Vec::new(),
            breaks: Vec::new(),
            breaks_declared: false,
            only_in_compound: None,
            forbidden: None,
            need_affix: None,
            keep_case: None,
            circumfix: None,
            ignore_chars: Vec::new(),
        };
        let mut read = Reading::default();
        for (index, line) in text.lines().enumerate() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            file.directive(&parts, index + 1, &mut read)?;
        }
        // The flags of a rule read with the flag type the whole file ends on.
        let parsed: Vec<(Flag, usize, Vec<Flag>)> = read
            .written
            .into_iter()
            .map(|(flag, at, cont)| (flag, at, file.parse_flags(&cont)))
            .collect();
        for (flag, at, cont) in parsed {
            if let Some(rule) = file
                .classes
                .get_mut(&flag)
                .and_then(|class| class.rules.get_mut(at))
            {
                rule.cont = cont;
            }
        }
        file.conversions
            .sort_by_key(|(from, _)| Reverse(from.len()));
        file.conversions.retain(|(from, _)| !from.is_empty());
        file.drop_ignored();
        Ok(file)
    }

    /// Reads one line of the file: a directive, or something Hunspell skips.
    fn directive(
        &mut self,
        parts: &[&str],
        number: usize,
        read: &mut Reading,
    ) -> Result<(), DictionaryError> {
        let Some(name) = parts.first() else {
            return Ok(());
        };
        let file = self;
        let argument = parts.get(1).copied();
        match *name {
            "SET" => {
                if let Some(set) = argument {
                    file.byte_flags =
                        set.eq_ignore_ascii_case("UTF-8") || set.eq_ignore_ascii_case("UTF8");
                }
            }
            "ICONV" => {
                if parts.len() == 2 {
                    return Ok(());
                }
                let (Some(from), Some(to)) = (parts.get(1), parts.get(2)) else {
                    return Err(short("ICONV", number));
                };
                file.conversions
                    .push(((*from).to_owned(), (*to).to_owned()));
            }
            "ONLYINCOMPOUND" => {
                file.only_in_compound = Some(
                    file.single_flag(argument.ok_or_else(|| short("ONLYINCOMPOUND", number))?),
                );
            }
            "COMPOUNDRULE" => {
                let rule = argument.ok_or_else(|| short("COMPOUNDRULE", number))?;
                // A number is the count of rules and preallocates only.
                if !all_digits(rule) {
                    file.compound_rules.push(rule.to_owned());
                }
            }
            "FORBIDDENWORD" => {
                file.forbidden = argument.map(|flag| file.single_flag(flag));
            }
            "NEEDAFFIX" | "PSEUDOROOT" => {
                file.need_affix = argument.map(|flag| file.single_flag(flag));
            }
            "KEEPCASE" => {
                file.keep_case = argument.map(|flag| file.single_flag(flag));
            }
            "CIRCUMFIX" => {
                file.circumfix = argument.map(|flag| file.single_flag(flag));
            }
            "IGNORE" => {
                if let Some(chars) = argument {
                    file.ignore_chars = chars.chars().collect();
                }
            }
            "AF" => {
                let alias = argument.ok_or_else(|| short("AF", number))?;
                // The first line is the count.
                if !read.saw_alias_count && all_digits(alias) {
                    read.saw_alias_count = true;
                    return Ok(());
                }
                file.aliases.push(alias.to_owned());
            }
            "BREAK" => {
                let pattern = argument.ok_or_else(|| short("BREAK", number))?;
                file.breaks_declared = true;
                // The first line is the count.
                if !read.saw_break_count && all_digits(pattern) {
                    read.saw_break_count = true;
                    return Ok(());
                }
                file.breaks.push(pattern.to_owned());
            }
            "FLAG" => {
                file.flag_type = match argument.ok_or_else(|| short("FLAG", number))? {
                    "num" => FlagType::Number,
                    "long" => FlagType::Long,
                    "UTF-8" => FlagType::Utf8,
                    _ => FlagType::Default,
                };
            }
            "PFX" | "SFX" => {
                let kind = if *name == "PFX" {
                    Kind::Prefix
                } else {
                    Kind::Suffix
                };
                file.affix_line(kind, parts, number, &mut read.written)?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Reads one `PFX` or `SFX` line: the header of a class or a rule of it.
    fn affix_line(
        &mut self,
        kind: Kind,
        parts: &[&str],
        number: usize,
        written: &mut Vec<(Flag, usize, String)>,
    ) -> Result<(), DictionaryError> {
        let directive = if kind == Kind::Prefix { "PFX" } else { "SFX" };
        let is_header = parts.len() >= 4
            && parts
                .get(2)
                .is_some_and(|cross| *cross == "Y" || *cross == "N")
            && parts.get(3).is_some_and(|count| all_digits(count));
        let (Some(name), Some(second)) = (parts.get(1), parts.get(2)) else {
            return Ok(());
        };
        if is_header {
            let flag = self.single_flag(name);
            self.classes.insert(
                flag,
                Class {
                    kind,
                    cross: *second == "Y",
                    rules: Vec::new(),
                },
            );
            return Ok(());
        }
        let Some(add) = parts.get(3) else {
            // A line with a missing field is not a rule.
            return if parts.len() >= 4 {
                Err(short(directive, number))
            } else {
                Ok(())
            };
        };
        let flag = self.single_flag(name);
        let Some(class) = self.classes.get_mut(&flag) else {
            // Hunspell skips a rule whose class was never declared.
            return Ok(());
        };
        let condition = match parts.get(4).copied().unwrap_or(".") {
            "." => None,
            text => Some(
                Condition::parse(text).ok_or_else(|| DictionaryError::Condition {
                    condition: text.to_owned(),
                    line: number,
                })?,
            ),
        };
        let add: &str = add;
        let (text, cont) = add.split_once('/').unwrap_or((add, ""));
        class.rules.push(Rule {
            strip: if *second == "0" {
                String::new()
            } else {
                (*second).to_owned()
            },
            text: if text == "0" {
                String::new()
            } else {
                text.to_owned()
            },
            cont: Vec::new(),
            condition,
        });
        let at = class.rules.len() - 1;
        written.push((flag, at, cont.to_owned()));
        Ok(())
    }

    /// Takes the `IGNORE` characters out of the texts of the rules, as they
    /// are taken out of every word.
    fn drop_ignored(&mut self) {
        if self.ignore_chars.is_empty() {
            return;
        }
        let ignored = self.ignore_chars.clone();
        for class in self.classes.values_mut() {
            for rule in &mut class.rules {
                rule.text.retain(|c| !ignored.contains(&c));
                rule.strip.retain(|c| !ignored.contains(&c));
            }
        }
    }

    /// `text` without the `IGNORE` characters.
    pub(super) fn ignore(&self, text: &str) -> String {
        if self.ignore_chars.is_empty() {
            return text.to_owned();
        }
        text.chars()
            .filter(|c| !self.ignore_chars.contains(c))
            .collect()
    }

    /// `text` with the `ICONV` pairs applied, the longest match first.
    pub(super) fn convert(&self, text: &str) -> String {
        if self.conversions.is_empty() {
            return text.to_owned();
        }
        let mut out = String::with_capacity(text.len());
        let mut rest = text;
        'text: while let Some(c) = rest.chars().next() {
            for (from, to) in &self.conversions {
                if let Some(after) = rest.strip_prefix(from.as_str()) {
                    out.push_str(to);
                    rest = after;
                    continue 'text;
                }
            }
            out.push(c);
            rest = rest.get(c.len_utf8()..).unwrap_or_default();
        }
        out
    }

    /// The flags a string of flags names.
    pub(super) fn parse_flags(&self, written: &str) -> Vec<Flag> {
        // With AF, an entry's flags are the number of an alias line.
        let written = if !self.aliases.is_empty() && all_digits(written) {
            let Some(alias) = written
                .parse::<usize>()
                .ok()
                .and_then(|number| number.checked_sub(1))
                .and_then(|index| self.aliases.get(index))
            else {
                return Vec::new();
            };
            alias.as_str()
        } else {
            written
        };
        match self.flag_type {
            FlagType::Number => written
                .split(',')
                .map(|flag| flag.as_bytes().to_vec())
                .collect(),
            FlagType::Long => written
                .as_bytes()
                .chunks(2)
                .filter(|pair| pair.len() == 2)
                .map(<[u8]>::to_vec)
                .collect(),
            FlagType::Utf8 => written
                .chars()
                .map(|c| c.to_string().into_bytes())
                .collect(),
            FlagType::Default if self.byte_flags => {
                written.bytes().map(|byte| vec![byte]).collect()
            }
            FlagType::Default => written
                .chars()
                .map(|c| c.to_string().into_bytes())
                .collect(),
        }
    }

    /// The flag a directive that names one flag gives.
    fn single_flag(&self, written: &str) -> Flag {
        self.parse_flags(written)
            .into_iter()
            .next()
            .unwrap_or_else(|| written.as_bytes().to_vec())
    }

    /// The tokens of a `COMPOUNDRULE`: a flag, or the `*` or `?` after one.
    ///
    /// A flag in parentheses may be several characters, as `FLAG long` and
    /// `FLAG num` need. Any other is one character, or one byte in a UTF-8
    /// file without `FLAG`.
    pub(super) fn compound_tokens(&self, rule: &str) -> Vec<CompoundToken> {
        let mut out = Vec::new();
        let bytes = rule.as_bytes();
        let mut at = 0;
        while let Some(&byte) = bytes.get(at) {
            if byte == b'*' || byte == b'?' {
                out.push(CompoundToken::Repeat(char::from(byte)));
                at += 1;
            } else if byte == b'(' {
                let rest = bytes.get(at + 1..).unwrap_or_default();
                if let Some(end) = rest.iter().position(|b| *b == b')') {
                    out.push(CompoundToken::Flag(
                        rest.get(..end).unwrap_or_default().to_vec(),
                    ));
                    at += end + 2;
                } else {
                    out.push(CompoundToken::Flag(rest.to_vec()));
                    return out;
                }
            } else {
                let length = if self.byte_flags {
                    1
                } else {
                    rule.get(at..)
                        .and_then(|tail| tail.chars().next())
                        .map_or(1, char::len_utf8)
                };
                out.push(CompoundToken::Flag(
                    bytes.get(at..at + length).unwrap_or_default().to_vec(),
                ));
                at += length;
            }
        }
        out
    }

    /// The words that `word` makes with the flags `flags`.
    ///
    /// # Errors
    ///
    /// Returns a [`DictionaryError`] when a flag names no affix class.
    pub(crate) fn expand(&self, word: &str, flags: &str) -> Result<Vec<String>, DictionaryError> {
        if flags.is_empty() {
            return Ok(vec![word.to_owned()]);
        }
        if word.is_empty() {
            return Err(DictionaryError::Flags {
                flags: flags.to_owned(),
            });
        }
        let keys = self.parse_flags(flags);
        if let Some(unknown) = keys.iter().find(|key| !self.classes.contains_key(*key)) {
            return Err(DictionaryError::UnknownFlag {
                flag: String::from_utf8_lossy(unknown).into_owned(),
            });
        }
        let mut words: Vec<String> = Vec::new();
        for form in self.root(word, keys) {
            if !words.contains(&form.word) {
                words.push(form.word);
            }
        }
        Ok(words)
    }

    /// The forms of a root with the given flags.
    pub(super) fn root(&self, word: &str, flags: Vec<Flag>) -> Vec<Form> {
        let wants_affix = has_flag(&flags, self.need_affix.as_ref());
        let stem = Form {
            word: word.to_owned(),
            cont: flags.clone(),
            flags,
            wants_affix,
            plain_affix: false,
            open: None,
        };
        self.emit(&stem, 0)
    }

    /// The form, unless it is only a step toward another, and then the forms
    /// the affixes its flags name build on it.
    fn emit(&self, form: &Form, depth: usize) -> Vec<Form> {
        let mut out = Vec::new();
        let cont = form.cont.clone();
        if !form.is_step() {
            out.push(form.clone());
        }
        if depth > MAX_AFFIX_DEPTH {
            return out;
        }
        out.extend(self.affixed(form, &cont, depth));
        out
    }

    /// The forms that the affixes named by `keys` make from `base`.
    fn affixed(&self, base: &Form, keys: &[Flag], depth: usize) -> Vec<Form> {
        let mut out = Vec::new();
        let mut prefix_classes: Vec<&Class> = Vec::new();
        let mut suffix_classes: Vec<&Class> = Vec::new();
        for key in keys {
            let Some(class) = self.classes.get(key) else {
                continue;
            };
            if !class.cross {
                self.emit_all(&self.derive(class, base), &mut out, depth);
            } else if class.kind == Kind::Prefix {
                prefix_classes.push(class);
            } else {
                suffix_classes.push(class);
            }
        }
        for suffix in &suffix_classes {
            self.emit_all(&self.derive(suffix, base), &mut out, depth);
        }
        for prefix in &prefix_classes {
            let prefixed = self.derive(prefix, base);
            self.emit_all(&prefixed, &mut out, depth);
            for suffix in &suffix_classes {
                for form in &prefixed {
                    self.emit_all(&self.derive(suffix, form), &mut out, depth);
                }
            }
        }
        out
    }

    /// Emits each form, following its continuation flags one level further.
    fn emit_all(&self, forms: &[Form], out: &mut Vec<Form>, depth: usize) {
        for form in forms {
            out.extend(self.emit(form, depth + 1));
        }
    }

    /// The forms the rules of `class` make from `base`, one for each rule whose
    /// condition `base` meets.
    fn derive(&self, class: &Class, base: &Form) -> Vec<Form> {
        let mut out = Vec::new();
        for rule in &class.rules {
            if let Some(condition) = &rule.condition
                && !condition.holds(&base.word, class.kind)
            {
                continue;
            }
            // A rule whose strip text is not on the word makes nothing, as in
            // Hunspell.
            let word = match class.kind {
                Kind::Prefix => base
                    .word
                    .strip_prefix(rule.strip.as_str())
                    .map(|stripped| format!("{}{stripped}", rule.text)),
                Kind::Suffix => base
                    .word
                    .strip_suffix(rule.strip.as_str())
                    .map(|stripped| format!("{stripped}{}", rule.text)),
            };
            let Some(word) = word else {
                continue;
            };
            let mut form = base.clone();
            form.word = word;
            form.cont.clone_from(&rule.cont);
            if !rule.cont.is_empty() {
                form.flags = union(&base.flags, &rule.cont);
            }
            if has_flag(&form.cont, self.need_affix.as_ref()) {
                form.wants_affix = true;
            } else {
                form.plain_affix = true;
            }
            if has_flag(&form.cont, self.circumfix.as_ref()) {
                form.open = match base.open {
                    Some(open) if open != class.kind => None,
                    Some(_) | None => Some(class.kind),
                };
            }
            out.push(form);
        }
        out
    }
}

/// One element of a `COMPOUNDRULE`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum CompoundToken {
    /// A flag.
    Flag(Flag),
    /// The `*` or `?` after a flag.
    Repeat(char),
}
