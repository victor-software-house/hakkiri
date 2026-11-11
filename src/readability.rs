//! Readability: the counts of a text and the scores they give.
//!
//! This is a port of the `summarize` package of [prose] at `b33e60d`, which
//! Vale counts with, under the MIT license in `data/readability/prose.LICENSE`.
//! Sentences come from hakkiri's own splitter, and words from the pattern of
//! prose's tokenizer.
//!
//! [prose]: https://github.com/jdkato/prose

use std::collections::HashSet;
use std::sync::LazyLock;

use regex::Regex;

use crate::splitter::Model;
use crate::words::words;

/// The familiar words of the Dale-Chall list, one per line.
const EASY: &str = include_str!("../data/readability/easy-words.txt");

static EASY_WORDS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| EASY.lines().collect());

/// The words whose syllables the rules below get wrong, with their counts.
const CORNER_CASES: [(&str, i64); 51] = [
    ("abalone", 4),
    ("abare", 3),
    ("abed", 2),
    ("abruzzese", 4),
    ("abbruzzese", 4),
    ("aborigine", 5),
    ("aborigines", 5),
    ("acreage", 3),
    ("adame", 3),
    ("adieu", 2),
    ("adobe", 3),
    ("anemone", 4),
    ("apache", 3),
    ("aphrodite", 4),
    ("apostrophe", 4),
    ("ariadne", 4),
    ("cafe", 2),
    ("cafes", 2),
    ("calliope", 4),
    ("catastrophe", 4),
    ("chile", 2),
    ("chloe", 2),
    ("circe", 2),
    ("coyote", 3),
    ("epitome", 4),
    ("facsimile", 4),
    ("forever", 3),
    ("gethsemane", 4),
    ("guacamole", 4),
    ("hyperbole", 4),
    ("jesse", 2),
    ("jukebox", 2),
    ("karate", 3),
    ("machete", 3),
    ("maybe", 2),
    ("people", 2),
    ("recipe", 3),
    ("sesame", 3),
    ("shoreline", 2),
    ("simile", 3),
    ("syncope", 3),
    ("tamale", 3),
    ("yosemite", 4),
    ("daphne", 2),
    ("eurydice", 4),
    ("euterpe", 3),
    ("hermione", 4),
    ("penelope", 4),
    ("persephone", 4),
    ("phoebe", 2),
    ("zoe", 2),
];

/// Prefixes that add a syllable, in two groups: a prefix of the first group
/// adds two, and one of the second adds one.
const PREFIXES: [&[&str]; 2] = [
    &[
        "above", "anti", "ante", "counter", "hyper", "afore", "agri", "infra", "intra", "inter",
        "over", "semi", "ultra", "under", "extra", "dia", "micro", "mega", "kilo", "pico", "nano",
        "macro",
    ],
    &[
        "un", "fore", "ware", "none", "non", "out", "post", "sub", "pre", "pro", "dis", "side",
    ],
];

/// Suffixes that add a syllable, in three groups worth three, two and one.
const SUFFIXES: [&[&str]; 3] = [
    &["ology", "ologist", "onomy", "onomist"],
    &["fully", "berry", "woman", "women"],
    &[
        "ly", "less", "some", "ful", "er", "ers", "ness", "cian", "cians", "ment", "ments", "ette",
        "ettes", "ville", "villes", "ships", "ship", "side", "sides", "port", "ports", "shire",
        "shires", "tion", "tioned",
    ],
];

struct Patterns {
    monosyllabic_one: Regex,
    monosyllabic_two: Regex,
    double_one: Regex,
    double_two: Regex,
    double_three: Regex,
    double_four: Regex,
    vowels: Regex,
}

#[expect(
    clippy::expect_used,
    reason = "Each pattern is a constant that compiles."
)]
fn compile(pattern: &str) -> Regex {
    Regex::new(pattern).expect("a syllable pattern compiles")
}

static PATTERNS: LazyLock<Patterns> = LazyLock::new(|| Patterns {
    monosyllabic_one: compile(
        "cia(l|$)|tia|cius|cious|[^aeiou]giu|[aeiouy][^aeiouy]ion|iou|sia$|eous$|[oa]gue$|\
        .[^aeiuoycgltdb]{2,}ed$|.ely$|^jua|uai|eau|^busi$|\
        ([aeiouy](b|c|ch|dg|f|g|gh|gn|k|l|lch|ll|lv|m|mm|n|nc|ng|nch|nn|p|r|rc|rn|rs|rv|s|sc|sk|\
        sl|squ|ss|th|v|y|z)ed$)|\
        ([aeiouy](b|ch|d|f|gh|gn|k|l|lch|ll|lv|m|mm|n|nch|nn|p|r|rn|rs|rv|s|sc|sk|sl|squ|ss|st|t|\
        th|v|y)es$)",
    ),
    monosyllabic_two: compile(
        "[aeiouy](b|c|ch|d|dg|f|g|gh|gn|k|l|ll|lv|m|mm|n|nc|ng|nn|p|r|rc|rn|rs|rv|s|sc|sk|sl|squ|\
        ss|st|t|th|v|y|z)e$",
    ),
    double_one: compile(
        "(?:[^aeiouy]ie(r|st|t)|[aeiouym]bl|eo|ism|asm|thm|dnt|uity|dea|gean|oa|ua|eings?|\
        [aeiouy]sh?e[rsd])$",
    ),
    double_two: compile("[^gq]ua[^auieo]|[aeiou]{3}|^(ia|mc|coa[dglx].)"),
    double_three: compile(
        "[^aeiou]y[ae]|[^l]lien|riet|dien|iu|io|ii|uen|real|iell|eo[^aeiou]|[aeiou]y[aeiou]",
    ),
    double_four: compile("[^s]ia"),
    vowels: compile("[aeiouy]+"),
});

/// How many times `pattern` matches `text`.
fn matches(pattern: &Regex, text: &str) -> i64 {
    i64::try_from(pattern.find_iter(text).count()).unwrap_or(i64::MAX)
}

/// The text without the first prefix of `group` it starts with.
fn trim_prefix(word: &str, group: &[&str]) -> String {
    group
        .iter()
        .find_map(|prefix| word.strip_prefix(prefix))
        .unwrap_or(word)
        .to_owned()
}

/// The text without the first suffix of `group` it ends with.
fn trim_suffix(word: &str, group: &[&str]) -> String {
    group
        .iter()
        .find_map(|suffix| word.strip_suffix(suffix))
        .unwrap_or(word)
        .to_owned()
}

/// Trims the first group that changes `word`, and says how many syllables
/// that group is worth: the first of `n` groups is worth `n`.
fn clear_part(
    word: String,
    groups: &[&[&str]],
    trim: fn(&str, &[&str]) -> String,
) -> (String, i64) {
    for (index, group) in groups.iter().enumerate() {
        let trimmed = trim(&word, group);
        if trimmed != word {
            let worth = i64::try_from(groups.len() - index).unwrap_or(0);
            return (trimmed, worth);
        }
    }
    (word, 0)
}

/// The number of syllables in `word`, which is a word and not running text.
#[must_use]
pub fn syllables(word: &str) -> usize {
    match word.chars().count() {
        0 => return 0,
        1 | 2 => return 1,
        _ => {}
    }
    let word = word.to_lowercase();
    let known = CORNER_CASES.iter().find(|(known, _)| *known == word);
    let count = if let Some((_, count)) = known {
        *count
    } else {
        let patterns = &*PATTERNS;
        let (text, prefix) = clear_part(word, &PREFIXES, trim_prefix);
        let (text, suffix) = clear_part(text, &SUFFIXES, trim_suffix);
        prefix + suffix + matches(&patterns.vowels, &text)
            - matches(&patterns.monosyllabic_one, &text)
            - matches(&patterns.monosyllabic_two, &text)
            + matches(&patterns.double_one, &text)
            + matches(&patterns.double_two, &text)
            + matches(&patterns.double_three, &text)
            + matches(&patterns.double_four, &text)
    };
    usize::try_from(count).unwrap_or(0).max(1)
}

/// The count of a whole number as a float. The counts of a text stay far
/// below 2^32.
fn float(count: usize) -> f64 {
    f64::from(u32::try_from(count).unwrap_or(u32::MAX))
}

/// The counts of a text.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Counts {
    /// The words.
    pub words: usize,
    /// The sentences.
    pub sentences: usize,
    /// The letters and digits of the words.
    pub characters: usize,
    /// The syllables of the words.
    pub syllables: usize,
    /// The words of more than two syllables, not counting the one an `es`,
    /// `ed` or `ing` ending adds.
    pub complex_words: usize,
    /// The words of more than two syllables.
    pub polysyllabic_words: usize,
    /// The words of more than six characters.
    pub long_words: usize,
    /// The different words that the Dale-Chall list holds.
    pub easy_words: usize,
    /// The words of each sentence.
    pub sentence_lengths: Vec<usize>,
}

impl Counts {
    /// The counts of `text`, cut into paragraphs at each blank line and into
    /// sentences by `model`.
    #[must_use]
    pub fn of(text: &str, model: &Model) -> Self {
        let mut counts = Self::default();
        let mut seen = HashSet::new();
        for paragraph in text.split("\n\n") {
            for range in model.split(paragraph) {
                let sentence = paragraph.get(range).unwrap_or_default();
                counts.sentences += 1;
                let before = counts.words;
                for range in words(sentence) {
                    let word = sentence.get(range).unwrap_or_default();
                    counts.add_word(word);
                    seen.insert(word);
                }
                counts.sentence_lengths.push(counts.words - before);
            }
        }
        counts.easy_words = seen
            .iter()
            .filter(|word| EASY_WORDS.contains(*word))
            .count();
        counts
    }

    fn add_word(&mut self, word: &str) {
        self.words += 1;
        self.characters += word.chars().filter(|c| *c != '\'' && *c != '.').count();
        let syllables = syllables(word);
        self.syllables += syllables;
        if word.chars().count() > 6 {
            self.long_words += 1;
        }
        if syllables > 2 {
            self.polysyllabic_words += 1;
        }
        let ending = ["es", "ed", "ing"].iter().any(|end| word.ends_with(end));
        if syllables.saturating_sub(usize::from(ending)) > 2 {
            self.complex_words += 1;
        }
    }

    /// The standard deviation of the words in a sentence, or 0 for a text
    /// with no sentence.
    #[must_use]
    pub fn sentence_length_sd(&self) -> f64 {
        let n = float(self.sentence_lengths.len());
        if n == 0.0 {
            return 0.0;
        }
        let mean = self.sentence_lengths.iter().map(|l| float(*l)).sum::<f64>() / n;
        let squares: f64 = self
            .sentence_lengths
            .iter()
            .map(|l| (float(*l) - mean).powi(2))
            .sum();
        (squares / n).sqrt()
    }

    /// The scores, each with its name in a formula, or nothing when the text
    /// has no sentence.
    #[must_use]
    pub fn scores(&self) -> Vec<(&'static str, f64)> {
        if self.sentences == 0 || self.words == 0 {
            return Vec::new();
        }
        let words = float(self.words);
        let sentences = float(self.sentences);
        let characters = float(self.characters);
        let syllables = float(self.syllables);
        let hard = words - float(self.easy_words);
        vec![
            (
                "automated_readability",
                4.71 * (characters / words) + 0.5 * (words / sentences) - 21.43,
            ),
            (
                "coleman_liau",
                0.0588 * (characters / words) * 100.0 - 0.296 * (sentences / words) * 100.0 - 15.8,
            ),
            (
                "dale_chall",
                0.1579 * ((hard / words) * 100.0) + 0.0496 * (words / sentences),
            ),
            (
                "flesch_kincaid",
                0.39 * words / sentences + 11.8 * syllables / words - 15.59,
            ),
            (
                "flesch_reading_ease",
                206.835 - 1.015 * words / sentences - 84.6 * syllables / words,
            ),
            (
                "gunning_fog",
                0.4 * (words / sentences + 100.0 * (float(self.complex_words) / words)),
            ),
            (
                "lix",
                words / sentences + (float(self.long_words) * 100.0) / words,
            ),
            (
                "smog",
                1.043 * (float(self.polysyllabic_words) * 30.0 / sentences).sqrt() + 3.1291,
            ),
        ]
    }
}

/// The quotation marks that open a quotation, with the mark that closes it.
fn closer(opening: char) -> Option<char> {
    match opening {
        '“' => Some('”'),
        '"' => Some('"'),
        '‘' => Some('’'),
        _ => None,
    }
}

/// Whether the mark `opening` at byte `at` of `text` opens a quotation. A
/// straight mark opens only after a space, a bracket or a dash.
fn opens(text: &str, at: usize, opening: char) -> bool {
    let Some(previous) = text.get(..at).and_then(|before| before.chars().next_back()) else {
        return true;
    };
    if opening == '"' {
        previous.is_whitespace() || "([{—–-/".contains(previous)
    } else {
        !previous.is_alphabetic() && !previous.is_numeric()
    }
}

/// The byte immediately after the mark `closer` that ends a quotation in
/// `text`, searching from `from`. A curly single mark closes only when no
/// letter follows it, since it is also an apostrophe.
fn closing(text: &str, closer: char, from: usize) -> Option<usize> {
    let rest = text.get(from..)?;
    rest.char_indices().find_map(|(offset, c)| {
        let end = from + offset + c.len_utf8();
        let next = text.get(end..).and_then(|after| after.chars().next());
        (c == closer && (closer != '’' || !next.is_some_and(char::is_alphabetic))).then_some(end)
    })
}

/// The words inside the quotations of `text`, marks included. A quotation
/// with no closing mark runs to the end of its paragraph.
#[must_use]
pub fn quoted_words(text: &str) -> usize {
    let mut count = 0;
    let mut from = 0;
    while let Some((at, opening)) = text.get(from..).and_then(|rest| {
        rest.char_indices().find_map(|(offset, c)| {
            closer(c)
                .filter(|_| opens(text, from + offset, c))
                .map(|_| (from + offset, c))
        })
    }) {
        let Some(closer) = closer(opening) else { break };
        let stop = text
            .get(at..)
            .and_then(|rest| rest.find("\n\n"))
            .map_or(text.len(), |gap| at + gap);
        let end = text
            .get(..stop)
            .and_then(|head| closing(head, closer, at + opening.len_utf8()))
            .unwrap_or(stop);
        count += text
            .get(at..end)
            .map_or(0, |quotation| quotation.split_whitespace().count());
        from = end;
    }
    count
}
