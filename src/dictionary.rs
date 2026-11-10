//! Hunspell dictionaries: which words a pair of `.aff` and `.dic` files
//! accepts.
//!
//! A port of the `spell` package of [Vale] at `66bbb85`, which is derived from
//! gospell under the MIT license (`data/dictionary/vale-spell.LICENSE`).
//! [`Dictionary::english()`] is Vale's built-in `en_US-web`.
//!
//! The port reads a dictionary eagerly: it makes every form of every entry
//! when it loads, where Vale makes the forms of an entry when it looks a word
//! up. The words accepted are the same. A dictionary that makes compounds by
//! flag (`COMPOUNDFLAG`, `COMPOUNDBEGIN`, `COMPOUNDMIDDLE`, `COMPOUNDEND`),
//! and every directive about suggestions, Turkic casing and `ß`, loads with
//! those directives left out.
//!
//! [Vale]: https://github.com/vale-cli/vale/tree/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/spell

mod affix;
mod charset;
mod speller;
mod words;

#[cfg(test)]
mod tests;

use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;
use std::{error, fmt};

pub(crate) use affix::AffixFile;
use affix::{CompoundToken, Flag, has_flag};
use regex::Regex;
pub use speller::{Book, Speller};

use crate::case::{lower_str, upper_str};

/// The English affix file that ships in the binary.
const ENGLISH_AFF: &str = include_str!("../data/dictionary/en_US-web.aff");

/// The English dictionary that ships in the binary.
const ENGLISH_DIC: &str = include_str!("../data/dictionary/en_US-web.dic");

/// The parsed English dictionary, read once per process.
static ENGLISH: LazyLock<Result<Dictionary, DictionaryError>> =
    LazyLock::new(|| Dictionary::parse(ENGLISH_AFF.as_bytes(), ENGLISH_DIC.as_bytes()));

/// How many times a word may be split at a break.
const MAX_BREAK_DEPTH: usize = 10;

/// Whether each part of a word split at its breaks is a word, by the part and
/// the number of splits that led to it.
type Memo<'a> = HashMap<(&'a str, usize), bool>;

/// What the forms of a word allow: some form is not only for compounds.
const PLAIN: u8 = 1;
/// Some form is not only for compounds and is not kept to its case.
const FREE: u8 = 1 << 1;

/// Why a dictionary could not be read.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DictionaryError {
    /// A file is not valid UTF-8.
    NotUtf8,
    /// The affix file declares a character set this reader does not read.
    Charset {
        /// The value of `SET`.
        set: String,
    },
    /// A directive lacks an argument.
    Directive {
        /// The directive.
        directive: String,
        /// The line of the affix file, counted from 1.
        line: usize,
    },
    /// An affix condition has a class that is not closed.
    Condition {
        /// The condition.
        condition: String,
        /// The line of the affix file, counted from 1.
        line: usize,
    },
    /// A compound rule does not make a pattern.
    CompoundRule {
        /// The rule.
        rule: String,
    },
    /// Flags were given with no word.
    Flags {
        /// The flags.
        flags: String,
    },
    /// A flag names no affix class.
    UnknownFlag {
        /// The flag.
        flag: String,
    },
}

impl fmt::Display for DictionaryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotUtf8 => write!(f, "the file is not valid UTF-8"),
            Self::Charset { set } => write!(
                f,
                "`SET {set}` is not a character set hakkiri reads: use UTF-8 or ISO8859-1"
            ),
            Self::Directive { directive, line } => {
                write!(f, "line {line}: `{directive}` lacks its argument")
            }
            Self::Condition { condition, line } => {
                write!(
                    f,
                    "line {line}: the condition `{condition}` has a class that is not closed"
                )
            }
            Self::CompoundRule { rule } => write!(f, "the compound rule `{rule}` is not a pattern"),
            Self::Flags { flags } => write!(f, "the flags `{flags}` need a word"),
            Self::UnknownFlag { flag } => write!(f, "`{flag}` is not an affix flag"),
        }
    }
}

impl error::Error for DictionaryError {}

/// How a word is read when it holds capitals, as Hunspell reads it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Capitals {
    /// `hello`
    None,
    /// `Hello`
    Initial,
    /// `HELLO`
    All,
    /// `ULinda`
    InitialMixed,
    /// `uLinda`
    Mixed,
}

/// A pattern a word may be split at.
#[derive(Clone, Debug)]
struct Break {
    pattern: String,
    /// Written `^pattern`.
    at_start: bool,
    /// Written `pattern$`.
    at_end: bool,
}

/// The words a pair of `.aff` and `.dic` files accepts.
#[derive(Debug)]
pub struct Dictionary {
    affixes: AffixFile,
    /// The words of the `.dic` file.
    roots: HashSet<String>,
    /// Every word an entry makes, with what its forms allow.
    forms: HashMap<String, u8>,
    /// The words made from a root with a capital, by their upper-case form.
    upper_forms: HashMap<String, Vec<(String, u8)>>,
    /// The words that `FORBIDDENWORD` entries make.
    forbidden: HashSet<String>,
    /// The `COMPOUNDRULE` patterns, as patterns over the words.
    compounds: Vec<Regex>,
    breaks: Vec<Break>,
}

/// A line of a `.dic` file cut into its word and its flags.
///
/// A `\/` is a slash in the word, and so is a slash that starts it.
fn split_entry(entry: &str) -> (String, Option<String>) {
    const ESCAPED: &str = "\u{ffff}";
    let entry = entry.replace("\\/", ESCAPED);
    let first = entry.chars().next().map_or(0, char::len_utf8);
    match entry.get(first..).and_then(|tail| tail.find('/')) {
        Some(at) => {
            let (word, flags) = entry.split_at(first + at);
            (
                word.replace(ESCAPED, "/"),
                Some(flags.get(1..).unwrap_or_default().replace(ESCAPED, "/")),
            )
        }
        None => (entry.replace(ESCAPED, "/"), None),
    }
}

/// Whether a field of a `.dic` line is morphology, such as `po:noun`.
fn is_morph_field(field: &str) -> bool {
    let bytes = field.as_bytes();
    field.len() > 2
        && bytes.get(2) == Some(&b':')
        && bytes.first().is_some_and(u8::is_ascii_alphabetic)
        && bytes.get(1).is_some_and(u8::is_ascii_alphabetic)
}

/// How a word is read by its capitals.
fn capitals(word: &str) -> Capitals {
    let (mut letters, mut uppers) = (0_usize, 0_usize);
    let mut first_upper = false;
    for (index, c) in word.chars().enumerate() {
        if !c.is_alphabetic() {
            continue;
        }
        letters += 1;
        if c.is_uppercase() {
            uppers += 1;
            if index == 0 {
                first_upper = true;
            }
        }
    }
    if uppers == 0 {
        Capitals::None
    } else if uppers == letters {
        Capitals::All
    } else if uppers == 1 && first_upper {
        Capitals::Initial
    } else if first_upper {
        Capitals::InitialMixed
    } else {
        Capitals::Mixed
    }
}

/// `word` with its first character in upper case.
fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    chars.next().map_or_else(String::new, |first| {
        format!("{}{}", crate::case::upper(first), chars.as_str())
    })
}

/// Whether `word` starts with `İ`, which has no lower case to fold to.
fn dotted_first(word: &str) -> bool {
    word.starts_with('\u{130}')
}

/// The byte offset of the first `pattern` in `word` at or after `from`.
fn find_from(word: &str, pattern: &str, from: usize) -> Option<usize> {
    let haystack = word.as_bytes().get(from..)?;
    let needle = pattern.as_bytes();
    if needle.is_empty() {
        return None;
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|at| at + from)
}

impl Dictionary {
    /// The English dictionary that ships in the binary, Vale's `en_US-web`.
    ///
    /// The files are read once per process.
    ///
    /// # Errors
    ///
    /// Returns the [`DictionaryError`] of the embedded files. A test reads
    /// them, so a released binary does not return one.
    pub fn english() -> Result<&'static Self, &'static DictionaryError> {
        LazyLock::force(&ENGLISH).as_ref()
    }

    /// Reads a dictionary from the bytes of its two files.
    ///
    /// # Errors
    ///
    /// Returns a [`DictionaryError`] when a file is in a character set this
    /// reader does not read, a directive lacks its argument, or a condition
    /// does not read.
    pub fn parse(aff: &[u8], dic: &[u8]) -> Result<Self, DictionaryError> {
        let (aff_text, dic_text) = charset::decode(aff, dic)?;
        let affixes = AffixFile::parse(&aff_text)?;
        let mut dictionary = Self {
            roots: HashSet::new(),
            forms: HashMap::new(),
            upper_forms: HashMap::new(),
            forbidden: HashSet::new(),
            compounds: Vec::new(),
            breaks: Vec::new(),
            affixes,
        };
        let mut flagged: HashMap<Flag, Vec<String>> = HashMap::new();
        for rule in &dictionary.affixes.compound_rules {
            for token in dictionary.affixes.compound_tokens(rule) {
                if let CompoundToken::Flag(flag) = token {
                    flagged.entry(flag).or_default();
                }
            }
        }
        // The first line is a count.
        for line in dic_text.lines().skip(1) {
            let line = line.split('\t').next().unwrap_or_default();
            let fields: Vec<&str> = line.split_whitespace().collect();
            let Some(first) = fields.first() else {
                continue;
            };
            // A space inside the word makes a pair, `compound word`.
            if let Some(second) = fields.get(1)
                && !is_morph_field(second)
            {
                dictionary.add_root(&format!("{first} {second}"), Vec::new());
                continue;
            }
            let (word, written) = split_entry(first);
            if word.is_empty() || written.as_deref() == Some("") {
                continue;
            }
            let flags = written
                .as_deref()
                .map(|flags| dictionary.affixes.parse_flags(flags))
                .unwrap_or_default();
            if has_flag(&flags, dictionary.affixes.forbidden.as_ref()) {
                dictionary.add_forbidden(&word, flags);
                continue;
            }
            for flag in &flags {
                if let Some(words) = flagged.get_mut(flag) {
                    words.push(word.clone());
                }
            }
            dictionary.add_root(&word, flags);
        }
        dictionary.compounds = dictionary.compound_patterns(&flagged)?;
        dictionary.breaks = if dictionary.affixes.breaks_declared {
            Self::break_rules(&dictionary.affixes.breaks)
        } else {
            Self::break_rules(&["-".to_owned(), "^-".to_owned(), "-$".to_owned()])
        };
        Ok(dictionary)
    }

    /// The break rules for `patterns`.
    fn break_rules(patterns: &[String]) -> Vec<Break> {
        patterns
            .iter()
            .take(64)
            .filter_map(|written| {
                let mut pattern = written.as_str();
                let mut at_start = false;
                let mut at_end = false;
                if pattern.len() > 1
                    && let Some(rest) = pattern.strip_prefix('^')
                {
                    at_start = true;
                    pattern = rest;
                }
                if pattern.len() > 1
                    && let Some(rest) = pattern.strip_suffix('$')
                {
                    at_end = true;
                    pattern = rest;
                }
                (!pattern.is_empty()).then(|| Break {
                    pattern: pattern.to_owned(),
                    at_start,
                    at_end,
                })
            })
            .collect()
    }

    /// Each `COMPOUNDRULE` as a pattern: every flag is the group of the words
    /// that carry it, and `*` and `?` apply to the group before them.
    fn compound_patterns(
        &self,
        flagged: &HashMap<Flag, Vec<String>>,
    ) -> Result<Vec<Regex>, DictionaryError> {
        let mut patterns = Vec::new();
        for rule in &self.affixes.compound_rules {
            let mut text = String::from("^");
            for token in self.affixes.compound_tokens(rule) {
                match token {
                    CompoundToken::Repeat(mark) => text.push(mark),
                    CompoundToken::Flag(flag) => {
                        let words = flagged.get(&flag).map(Vec::as_slice).unwrap_or_default();
                        if words.is_empty() {
                            // Nothing carries the flag.
                            text.push_str(r"([^\s\S])");
                        } else {
                            let quoted: Vec<String> =
                                words.iter().map(|word| regex::escape(word)).collect();
                            text.push('(');
                            text.push_str(&quoted.join("|"));
                            text.push(')');
                        }
                    }
                }
            }
            text.push('$');
            patterns.push(
                regex::RegexBuilder::new(&text)
                    .size_limit(1 << 26)
                    .build()
                    .or(Err(DictionaryError::CompoundRule { rule: rule.clone() }))?,
            );
        }
        Ok(patterns)
    }

    /// Records an entry of the `.dic` file and every form it makes.
    fn add_root(&mut self, word: &str, flags: Vec<Flag>) {
        let word = self.affixes.ignore(word);
        let capital = word != lower_str(&word);
        for form in self.affixes.root(&word, flags) {
            let mut allows = 0;
            if !has_flag(&form.flags, self.affixes.only_in_compound.as_ref()) {
                allows |= PLAIN;
                if !has_flag(&form.flags, self.affixes.keep_case.as_ref()) {
                    allows |= FREE;
                }
            }
            if capital {
                self.upper_forms
                    .entry(upper_str(&form.word))
                    .or_default()
                    .push((form.word.clone(), allows));
            }
            *self.forms.entry(form.word).or_insert(0) |= allows;
        }
        self.roots.insert(word);
    }

    /// Records an entry of the `.dic` file marked `FORBIDDENWORD`.
    fn add_forbidden(&mut self, word: &str, flags: Vec<Flag>) {
        let word = self.affixes.ignore(word);
        for form in self.affixes.root(&word, flags) {
            self.forbidden.insert(form.word);
        }
    }

    /// `text` as the dictionary's `ICONV` pairs convert it, such as a curly
    /// apostrophe to a straight one.
    #[must_use]
    pub fn convert(&self, text: &str) -> String {
        self.affixes.convert(text)
    }

    /// Whether the dictionary accepts `word`, or `listed` holds it.
    ///
    /// The word is read as the dictionary reads its own: through `ICONV`, and
    /// without the `IGNORE` characters.
    #[must_use]
    pub fn spell(&self, word: &str, listed: &HashSet<String>) -> bool {
        let converted = self.affixes.convert(word);
        let ignored = self.affixes.ignore(&converted);
        self.spell_depth(&ignored, 0, listed, &mut Memo::new())
    }

    /// The one spelling the dictionary has for `word` in another case, such as
    /// `GitHub` for `github`. Two spellings that differ in case, or none, give
    /// `None`.
    #[must_use]
    pub fn recase(&self, word: &str, listed: &HashSet<String>) -> Option<String> {
        let mut found: Option<&str> = None;
        for (candidate, _) in self.upper_forms.get(&upper_str(word))? {
            if candidate == word || self.is_forbidden(candidate) || !self.spell(candidate, listed) {
                continue;
            }
            if found.is_some_and(|earlier| earlier != candidate) {
                return None;
            }
            found = Some(candidate.as_str());
        }
        found.map(str::to_owned)
    }

    /// Whether a `FORBIDDENWORD` entry makes `word`. A word that is also the
    /// root of another entry stays valid.
    fn is_forbidden(&self, word: &str) -> bool {
        !self.forbidden.is_empty() && !self.roots.contains(word) && self.forbidden.contains(word)
    }

    /// Whether some entry makes `word` and it may stand on its own. A folded
    /// word, one whose case was changed to look it up, is not a `KEEPCASE`
    /// form.
    fn is_word(&self, word: &str, folded: bool) -> bool {
        if self.is_forbidden(word) {
            return false;
        }
        let needed = if folded { FREE } else { PLAIN };
        self.forms
            .get(word)
            .is_some_and(|allows| allows & needed != 0)
    }

    /// Whether an all-caps `word` is the upper-case form of a word some entry
    /// makes.
    fn is_word_upper(&self, word: &str) -> bool {
        self.upper_forms.get(&upper_str(word)).is_some_and(|forms| {
            forms
                .iter()
                .any(|(form, allows)| allows & FREE != 0 && !self.is_forbidden(form))
        })
    }

    /// `spell` with a count of the splits that led here, and what the parts
    /// of the word already gave.
    fn spell_depth<'a>(
        &self,
        word: &'a str,
        depth: usize,
        listed: &HashSet<String>,
        memo: &mut Memo<'a>,
    ) -> bool {
        if self.is_forbidden(word) {
            return false;
        }
        // A trailing period is an abbreviation's, or the sentence's.
        if word.ends_with('.') {
            let trimmed = word.trim_end_matches('.');
            if !trimmed.is_empty() && self.spell_depth(trimmed, depth, listed, memo) {
                return true;
            }
        }
        if listed.contains(word) || self.is_word(word, false) {
            return true;
        }
        let folded = lower_str(word);
        if listed.contains(&folded) {
            return true;
        }
        let accepted = match capitals(word) {
            Capitals::None | Capitals::Mixed => false,
            Capitals::Initial => !dotted_first(word) && self.is_word(&folded, true),
            Capitals::All => self.is_word_upper(word) || self.is_folded_word(word, &folded),
            Capitals::InitialMixed => {
                let first = word.chars().next().map_or(0, char::len_utf8);
                !dotted_first(word)
                    && word.get(first..).is_some_and(|rest| {
                        let head = word.get(..first).map(lower_str).unwrap_or_default();
                        self.is_word(&format!("{head}{rest}"), true)
                    })
            }
        };
        if accepted
            || words::is_number(word)
            || words::is_hex(word)
            || words::is_binary(word)
            || words::is_hash(word)
        {
            return true;
        }
        // A compound rule reads the word as written or, for a capitalized or
        // all-caps word, in lower case, so an ordinal in capitals reads as one
        // in lower case. An entry on its own is not a compound.
        for pattern in &self.compounds {
            if !self.roots.contains(word) && pattern.is_match(word) {
                return true;
            }
            if !self.roots.contains(&folded)
                && folded != word
                && capitals(word) != Capitals::Mixed
                && pattern.is_match(&folded)
            {
                return true;
            }
        }
        // A number with units, such as `100GB`.
        if let Some(units) = words::units(word)
            && self.is_word(units, false)
        {
            return true;
        }
        self.spell_parts(word, depth, listed, memo)
    }

    /// Whether an all-caps word is a word in lower case or capitalized, which
    /// no entry keeps to its case.
    fn is_folded_word(&self, word: &str, folded: &str) -> bool {
        let forms = if dotted_first(word) {
            let rest = word.get('\u{130}'.len_utf8()..).unwrap_or_default();
            vec![format!("\u{130}{}", lower_str(rest))]
        } else {
            vec![folded.to_owned(), capitalize(folded)]
        };
        forms.iter().any(|form| self.is_word(form, true))
    }

    /// Whether `word` is valid once split at the breaks of the dictionary.
    ///
    /// A break written at the start or end is taken off that end of the word,
    /// and any other splits the word at each place it occurs. The parts are
    /// then checked on their own, and split again as needed.
    ///
    /// The same part comes up again from other splits, so each part is
    /// checked once for each depth. The depth is in the key because a part
    /// checked deeper has fewer splits left.
    fn spell_parts<'a>(
        &self,
        word: &'a str,
        depth: usize,
        listed: &HashSet<String>,
        memo: &mut Memo<'a>,
    ) -> bool {
        if depth >= MAX_BREAK_DEPTH {
            return false;
        }
        for rule in &self.breaks {
            let length = rule.pattern.len();
            if rule.at_start {
                if word.len() > length
                    && let Some(rest) = word.strip_prefix(rule.pattern.as_str())
                    && self.spell_part(rest, depth + 1, listed, memo)
                {
                    return true;
                }
            } else if rule.at_end {
                if word.len() > length
                    && let Some(rest) = word.strip_suffix(rule.pattern.as_str())
                    && self.spell_part(rest, depth + 1, listed, memo)
                {
                    return true;
                }
            } else {
                // Only interior places: both sides must hold something.
                let mut from = 1;
                while let Some(at) = find_from(word, &rule.pattern, from) {
                    if at + length >= word.len() {
                        break;
                    }
                    if let (Some(left), Some(right)) = (word.get(..at), word.get(at + length..))
                        && self.spell_part(left, depth + 1, listed, memo)
                        && self.spell_part(right, depth + 1, listed, memo)
                    {
                        return true;
                    }
                    from = at + 1;
                }
            }
        }
        false
    }

    /// `spell_depth` for a part of a word, from `memo` when it was checked.
    fn spell_part<'a>(
        &self,
        part: &'a str,
        depth: usize,
        listed: &HashSet<String>,
        memo: &mut Memo<'a>,
    ) -> bool {
        if let Some(known) = memo.get(&(part, depth)) {
            return *known;
        }
        let spelled = self.spell_depth(part, depth, listed, memo);
        memo.insert((part, depth), spelled);
        spelled
    }

    /// The forms that `word` makes with the flags `flags`, such as `utilize`
    /// and `DSG`, under this dictionary's affix file.
    ///
    /// # Errors
    ///
    /// Returns a [`DictionaryError`] when a flag names no affix class.
    pub fn expand(&self, word: &str, flags: &str) -> Result<Vec<String>, DictionaryError> {
        self.affixes.expand(word, flags)
    }
}
