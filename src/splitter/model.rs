//! The model: the four tables Punkt decides with, and its file format.
//!
//! The file is tab-separated text, one record per line, sorted, so a change
//! to the model reads as a diff of lines.
//! The first line names the format and its version.
//! Each other line is one of:
//!
//! | Record | Fields | Meaning |
//! |:--|:--|:--|
//! | `abbreviation` | word | A word whose period does not end a sentence. |
//! | `collocation` | word, word | A pair a period between them does not split. |
//! | `starter` | word | A word that starts sentences. |
//! | `orthography` | word, bits | How the word was capitalized where it was seen. |

use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;
use std::{error, fmt};

/// The name and version on the first line of a model file.
const HEADER: &str = "hakkiri-splitter-model\t1";

/// The English model, trained by `examples/train_splitter.rs`.
const ENGLISH: &str = include_str!("../../data/splitter/en.tsv");

/// The parsed English model, read once per process.
static ENGLISH_MODEL: LazyLock<Result<Model, FormatError>> =
    LazyLock::new(|| Model::parse(ENGLISH));

/// Where in a sentence a word stood.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Position {
    /// Right after a sentence break.
    Initial,
    /// Inside a sentence.
    Internal,
    /// After a line break or an abbreviation, where either could hold.
    Unknown,
}

/// The case of a word's first letter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Case {
    /// An upper-case letter.
    Upper,
    /// A lower-case letter.
    Lower,
}

/// How a word was capitalized where it was seen, at each [`Position`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct Orthography(u8);

impl Orthography {
    const BEGIN_UPPER: u8 = 1 << 1;
    const MIDDLE_UPPER: u8 = 1 << 2;
    const UNKNOWN_UPPER: u8 = 1 << 3;
    const BEGIN_LOWER: u8 = 1 << 4;
    const MIDDLE_LOWER: u8 = 1 << 5;
    const UNKNOWN_LOWER: u8 = 1 << 6;
    const UPPER: u8 = Self::BEGIN_UPPER | Self::MIDDLE_UPPER | Self::UNKNOWN_UPPER;
    const LOWER: u8 = Self::BEGIN_LOWER | Self::MIDDLE_LOWER | Self::UNKNOWN_LOWER;
    /// Every bit a file may hold.
    const ALL: u8 = Self::UPPER | Self::LOWER;

    /// Notes that the word was seen in `case` at `position`.
    pub(super) fn record(&mut self, position: Position, case: Case) {
        self.0 |= match (case, position) {
            (Case::Upper, Position::Initial) => Self::BEGIN_UPPER,
            (Case::Upper, Position::Internal) => Self::MIDDLE_UPPER,
            (Case::Upper, Position::Unknown) => Self::UNKNOWN_UPPER,
            (Case::Lower, Position::Initial) => Self::BEGIN_LOWER,
            (Case::Lower, Position::Internal) => Self::MIDDLE_LOWER,
            (Case::Lower, Position::Unknown) => Self::UNKNOWN_LOWER,
        };
    }

    /// Seen with an upper-case first letter after a sentence break.
    pub(super) const fn begins_upper(self) -> bool {
        self.0 & Self::BEGIN_UPPER != 0
    }

    /// Seen with an upper-case first letter inside a sentence.
    pub(super) const fn middle_upper(self) -> bool {
        self.0 & Self::MIDDLE_UPPER != 0
    }

    /// Seen with a lower-case first letter after a sentence break.
    pub(super) const fn begins_lower(self) -> bool {
        self.0 & Self::BEGIN_LOWER != 0
    }

    /// Seen with an upper-case first letter anywhere.
    pub(super) const fn has_upper(self) -> bool {
        self.0 & Self::UPPER != 0
    }

    /// Seen with a lower-case first letter anywhere.
    pub(super) const fn has_lower(self) -> bool {
        self.0 & Self::LOWER != 0
    }
}

/// A sentence splitter's model of one language.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Model {
    /// Words whose final period does not end a sentence, in lower case.
    abbreviations: HashSet<String>,
    /// For a word, the words after it that a period between them does not
    /// split.
    collocations: HashMap<String, HashSet<String>>,
    /// Words that start sentences, in lower case.
    starters: HashSet<String>,
    /// How each word was capitalized.
    orthography: HashMap<String, Orthography>,
}

/// Why a model file could not be read.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FormatError {
    /// The first line is not the header of a format this version reads.
    Header,
    /// A line is not a record of the format.
    Record {
        /// The line number, counted from 1.
        line: usize,
        /// What is wrong with the line.
        problem: &'static str,
    },
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Header => write!(f, "the first line is not `{}`", HEADER.replace('\t', " ")),
            Self::Record { line, problem } => write!(f, "line {line}: {problem}"),
        }
    }
}

impl error::Error for FormatError {}

impl Model {
    /// The English model that ships in the binary.
    ///
    /// The file is parsed once per process.
    ///
    /// # Errors
    ///
    /// Returns the [`FormatError`] of the embedded file. A test reads it, so a
    /// released binary does not return one.
    pub fn english() -> Result<&'static Self, &'static FormatError> {
        LazyLock::force(&ENGLISH_MODEL).as_ref()
    }

    /// Reads a model from the text of a model file.
    ///
    /// # Errors
    ///
    /// Returns a [`FormatError`] when the header is not the one this version
    /// reads, or a line is not a record.
    pub fn parse(file: &str) -> Result<Self, FormatError> {
        let mut lines = file.lines();
        if lines.next() != Some(HEADER) {
            return Err(FormatError::Header);
        }
        let mut model = Self::default();
        for (index, line) in lines.enumerate() {
            let number = index.saturating_add(2);
            model.add_record(line, number)?;
        }
        Ok(model)
    }

    /// Adds the record on `line`, which is line `number` of the file.
    fn add_record(&mut self, line: &str, number: usize) -> Result<(), FormatError> {
        let fail = |problem| FormatError::Record {
            line: number,
            problem,
        };
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.iter().skip(1).any(|field| field.is_empty()) {
            return Err(fail("a field is empty"));
        }
        match fields.as_slice() {
            ["abbreviation", word] => {
                self.abbreviations.insert((*word).to_owned());
            }
            ["collocation", first, second] => {
                self.collocations
                    .entry((*first).to_owned())
                    .or_default()
                    .insert((*second).to_owned());
            }
            ["starter", word] => {
                self.starters.insert((*word).to_owned());
            }
            ["orthography", word, bits] => {
                let Ok(bits) = bits.parse::<u8>() else {
                    return Err(fail("the bits are not a number"));
                };
                if bits & !Orthography::ALL != 0 {
                    return Err(fail("the bits name a case that does not exist"));
                }
                self.orthography
                    .insert((*word).to_owned(), Orthography(bits));
            }
            _ => {
                return Err(fail(
                    "not an abbreviation, collocation, starter or orthography",
                ));
            }
        }
        Ok(())
    }

    /// The text of the model file: the header, then every record, sorted.
    ///
    /// Equal models give equal text.
    #[must_use]
    pub fn to_tsv(&self) -> String {
        let mut abbreviations: Vec<String> = self
            .abbreviations
            .iter()
            .map(|word| format!("abbreviation\t{word}"))
            .collect();
        let mut collocations: Vec<String> = self
            .collocations
            .iter()
            .flat_map(|(first, seconds)| {
                seconds
                    .iter()
                    .map(move |second| format!("collocation\t{first}\t{second}"))
            })
            .collect();
        let mut starters: Vec<String> = self
            .starters
            .iter()
            .map(|word| format!("starter\t{word}"))
            .collect();
        let mut orthography: Vec<String> = self
            .orthography
            .iter()
            .map(|(word, flags)| format!("orthography\t{word}\t{}", flags.0))
            .collect();
        let mut text = String::from(HEADER);
        text.push('\n');
        for records in [
            &mut abbreviations,
            &mut collocations,
            &mut starters,
            &mut orthography,
        ] {
            records.sort_unstable();
            for record in records.iter() {
                text.push_str(record);
                text.push('\n');
            }
        }
        text
    }

    /// Builds a model from its four tables.
    pub(super) fn new(
        abbreviations: HashSet<String>,
        collocations: HashMap<String, HashSet<String>>,
        starters: HashSet<String>,
        orthography: HashMap<String, Orthography>,
    ) -> Self {
        Self {
            abbreviations,
            collocations,
            starters,
            orthography,
        }
    }

    /// The abbreviations, in lower case and without their period.
    pub(super) const fn abbreviations(&self) -> &HashSet<String> {
        &self.abbreviations
    }

    /// Whether a period between `first` and `second` splits nothing.
    pub(super) fn is_collocation(&self, first: &str, second: &str) -> bool {
        self.collocations
            .get(first)
            .is_some_and(|seconds| seconds.contains(second))
    }

    /// Whether `word` often starts a sentence.
    pub(super) fn is_starter(&self, word: &str) -> bool {
        self.starters.contains(word)
    }

    /// How `word` was capitalized, or nothing known.
    pub(super) fn orthography(&self, word: &str) -> Orthography {
        self.orthography.get(word).copied().unwrap_or_default()
    }
}
