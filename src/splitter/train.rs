//! The trainer: reads text with no labels and learns the model's four tables.
//!
//! It follows Kiss and Strunk (2006) with NLTK's thresholds, over three passes
//! on the text:
//!
//! 1. Count every word type, and find the abbreviations: a word type whose
//!    period sticks to it more often than chance, scaled by how short it is and
//!    how many inner periods it has.
//! 2. Mark each token that may end a sentence, and note how each word was
//!    capitalized after a break, inside a sentence and where either could hold.
//! 3. Look at each token that ends in a period and the word after it, to find
//!    rare abbreviations, words that start sentences, and collocations.
//!
//! The result does not depend on the order of the documents inside a pass, only
//! on their content, and the same documents give the same model.

use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};

use super::model::{Case, Model, Orthography, Position};
use super::token::{Annotation, Gap, Mark, NUMBER, Token, tokens};

/// A word type is an abbreviation when its score is at least this.
const ABBREVIATION: f64 = 0.3;
/// A pair is a collocation when its log likelihood ratio is at least this.
const COLLOCATION: f64 = 7.88;
/// A word starts sentences when its log likelihood ratio is at least this.
const SENTENCE_STARTER: f64 = 30.0;
/// The size of text that the penalty for a word with no period counts in.
///
/// Kiss and Strunk's penalty counts the occurrences with no period in a text of
/// about 469,000 tokens. On a longer text the count is scaled down to this
/// size. The value comes from scoring the model on the evaluation sets with
/// sizes from 50,000 to 10,000,000; the larger ones drop `Mr`, `St` and `Feb`
/// as abbreviations.
const PENALTY_TEXT_TOKENS: u64 = 300_000;
/// A word type seen fewer times than this, with or without a period, is rare.
const RARE: u32 = 5;
/// The words whose capitalization the model keeps, the most frequent first.
///
/// A word that is not in the table reads as unknown, so the table can stay
/// small: it only holds the words seen often enough to say something.
const ORTHOGRAPHY_ENTRIES: usize = 20_000;

/// A count for each word type.
type Counts = HashMap<String, u32>;

/// What a training run found.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Summary {
    /// The documents read.
    pub documents: usize,
    /// The tokens read.
    pub tokens: u64,
    /// The tokens that end in one period.
    pub period_tokens: u64,
    /// The tokens that ended a sentence in the first look.
    pub sentence_breaks: u64,
    /// The different word types.
    pub types: usize,
    /// The abbreviations in the model.
    pub abbreviations: usize,
    /// The collocations in the model.
    pub collocations: usize,
    /// The sentence starters in the model.
    pub starters: usize,
    /// The words with a capitalization entry before the table was cut.
    pub orthography_seen: usize,
    /// The words with a capitalization entry in the model.
    pub orthography: usize,
}

/// A trained model and what the run found.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Trained {
    /// The model.
    pub model: Model,
    /// The counts of the run.
    pub summary: Summary,
}

/// The first pass: how often each word type occurs.
struct Counted {
    types: Counts,
    tokens: u64,
    period_tokens: u64,
}

/// The second pass: how each word was capitalized.
struct Capitalized {
    orthography: HashMap<String, Orthography>,
    sentence_breaks: u64,
}

/// The third pass: what pairs of neighbours show.
#[derive(Default)]
struct Pairs {
    rare: HashSet<String>,
    starters: Counts,
    collocations: HashMap<(String, String), u32>,
}

/// Trains a model on `documents`, each the text of one document or paragraph.
///
/// A blank line inside a document marks a paragraph, and a line break marks a
/// line, which the trainer reads as weak evidence of a sentence break.
/// The counts saturate at 2^32 - 1; a text that large is out of scope.
#[must_use]
pub fn train(documents: &[&str]) -> Trained {
    let counted = count_types(documents);
    let mut abbreviations = find_abbreviations(&counted);
    let capitalized = read_capitalization(documents, &abbreviations);
    let pairs = read_pairs(documents, &abbreviations, &counted, &capitalized);
    abbreviations.extend(pairs.rare.iter().cloned());
    let starters = find_starters(&pairs, &counted, capitalized.sentence_breaks);
    let collocations = find_collocations(&pairs, &counted, &starters);
    let orthography_seen = capitalized.orthography.len();
    let orthography = most_frequent(capitalized.orthography, &counted);
    let summary = Summary {
        documents: documents.len(),
        tokens: counted.tokens,
        period_tokens: counted.period_tokens,
        sentence_breaks: capitalized.sentence_breaks,
        types: counted.types.len(),
        abbreviations: abbreviations.len(),
        collocations: collocations.values().map(HashSet::len).sum(),
        starters: starters.len(),
        orthography_seen,
        orthography: orthography.len(),
    };
    Trained {
        model: Model::new(abbreviations, collocations, starters, orthography),
        summary,
    }
}

/// Adds one to the count of `key`.
fn bump(counts: &mut Counts, key: &str) {
    if let Some(count) = counts.get_mut(key) {
        *count = count.saturating_add(1);
    } else {
        counts.insert(key.to_owned(), 1);
    }
}

/// How often `kind` occurs, with and without a period after it.
fn total(types: &Counts, kind: &str) -> u32 {
    let without = types.get(kind).copied().unwrap_or(0);
    let with = types.get(&format!("{kind}.")).copied().unwrap_or(0);
    without.saturating_add(with)
}

/// A count as a float. A count above 2^32 - 1 reads as 2^32 - 1.
fn real(count: u64) -> f64 {
    u32::try_from(count).map_or_else(|_| f64::from(u32::MAX), f64::from)
}

/// A length as a float, with the same limit as [`real`].
fn real_len(length: usize) -> f64 {
    u32::try_from(length).map_or_else(|_| f64::from(u32::MAX), f64::from)
}

/// `x * ln(y)`, and 0 when `x` is 0, whatever `y` is.
fn weighted_ln(x: f64, y: f64) -> f64 {
    if x == 0.0 { 0.0 } else { x * y.ln() }
}

/// The log likelihood of `hits` in `trials` when each is a hit with
/// probability `p`.
fn binomial(hits: f64, trials: f64, p: f64) -> f64 {
    weighted_ln(hits, p) + weighted_ln(trials - hits, 1.0 - p)
}

/// Dunning's log likelihood ratio for two events `a` and `b` that occur
/// together `ab` times in `n` tokens.
///
/// A high value shows that `a` and `b` go together more often than chance, and
/// a low value shows that they are independent.
fn log_likelihood(a: u64, b: u64, ab: u64, n: u64) -> f64 {
    let all_have_a = n == a;
    let (a, b, ab, n) = (real(a), real(b), real(ab), real(n));
    let p = b / n;
    let p_given_a = ab / a;
    let p_without_a = if all_have_a { 1.0 } else { (b - ab) / (n - a) };
    let value = binomial(ab, a, p) + binomial(b - ab, n - a, p)
        - binomial(ab, a, p_given_a)
        - binomial(b - ab, n - a, p_without_a);
    -2.0 * value
}

/// The first pass: counts every word type.
fn count_types(documents: &[&str]) -> Counted {
    let mut counted = Counted {
        types: Counts::new(),
        tokens: 0,
        period_tokens: 0,
    };
    for document in documents {
        for token in tokens(document) {
            bump(&mut counted.types, &token.kind());
            counted.tokens = counted.tokens.saturating_add(1);
            if token.mark == Mark::Period {
                counted.period_tokens = counted.period_tokens.saturating_add(1);
            }
        }
    }
    counted
}

/// Whether `stem` could be an abbreviation: two or more letters and inner
/// periods, such as `mr` or `e.g`, that start and end with a letter.
///
/// A version, a file extension, a path or a code fragment is not an
/// abbreviation, and neither is a single letter: an initial has its own
/// rules, and a letter in the table would hide them.
fn is_abbreviation_shaped(stem: &str) -> bool {
    stem.chars().filter(|c| c.is_alphabetic()).count() >= 2
        && stem.chars().next().is_some_and(char::is_alphabetic)
        && stem.chars().next_back().is_some_and(char::is_alphabetic)
        && stem.chars().all(|c| c.is_alphabetic() || c == '.')
}

/// The abbreviations: word types whose period binds to them.
fn find_abbreviations(counted: &Counted) -> HashSet<String> {
    counted
        .types
        .iter()
        .filter_map(|(kind, &with_period)| {
            let stem = kind.strip_suffix('.')?;
            if !is_abbreviation_shaped(stem) {
                return None;
            }
            let without_period = counted.types.get(stem).copied().unwrap_or(0);
            let with = u64::from(with_period);
            let likelihood = log_likelihood(
                with + u64::from(without_period),
                counted.period_tokens,
                with,
                counted.tokens,
            );
            // Short words and words with inner periods, such as `e.g`, are
            // likelier abbreviations. A word that also occurs with no period is
            // less likely one: Kiss and Strunk divide by the number of letters
            // to the power of that count. On a text of many millions, the same
            // power drives `Mr` or `etc` to zero, because each is spelled both
            // ways a few hundred times. The count is scaled to the text the
            // penalty was set on.
            let periods = real_len(stem.matches('.').count().saturating_add(1));
            let letters = real_len(stem.chars().filter(|&c| c != '.').count());
            let scale = (real(PENALTY_TEXT_TOKENS) / real(counted.tokens)).min(1.0);
            let score = likelihood
                * (-letters).exp()
                * periods
                * letters.powf(-f64::from(without_period) * scale);
            (score >= ABBREVIATION).then(|| stem.to_owned())
        })
        .collect()
}

/// The second pass: how each word was capitalized, and how many tokens ended a
/// sentence.
fn read_capitalization(documents: &[&str], abbreviations: &HashSet<String>) -> Capitalized {
    let mut capitalized = Capitalized {
        orthography: HashMap::new(),
        sentence_breaks: 0,
    };
    for document in documents {
        let mut position = Position::Internal;
        for token in tokens(document) {
            let note = token.first_look(abbreviations);
            // A blank line is a good sign of a sentence break, unless an
            // abbreviation came just before it.
            if token.gap == Gap::Paragraph && position != Position::Unknown {
                position = Position::Initial;
            }
            // At the start of a line, a word inside a sentence could also
            // start one.
            if token.gap != Gap::Space && position == Position::Internal {
                position = Position::Unknown;
            }
            let case = if token.first_upper() {
                Some(Case::Upper)
            } else if token.first_lower() {
                Some(Case::Lower)
            } else {
                None
            };
            if let Some(case) = case {
                let kind = token.kind_without_sentence_period(note.sentence_break);
                capitalized
                    .orthography
                    .entry(kind)
                    .or_default()
                    .record(position, case);
            }
            position = if note.sentence_break {
                capitalized.sentence_breaks = capitalized.sentence_breaks.saturating_add(1);
                if token.internal_after {
                    Position::Internal
                } else if token.is_initial() || token.kind() == NUMBER {
                    Position::Unknown
                } else {
                    Position::Initial
                }
            } else if note.ellipsis || note.abbreviation {
                Position::Unknown
            } else {
                Position::Internal
            };
        }
    }
    capitalized
}

/// What the second and third pass read, to judge a pair of neighbours.
struct Evidence<'a> {
    abbreviations: &'a HashSet<String>,
    counted: &'a Counted,
    capitalized: &'a Capitalized,
}

impl Evidence<'_> {
    /// Whether `current`, which ends a sentence in the first look, is a rare
    /// word that the word after it shows to be an abbreviation.
    ///
    /// It is, when a comma, semicolon or colon follows it, or when a word in
    /// lower case follows it that is only ever capitalized after a break.
    fn is_rare_abbreviation(
        &self,
        current: &Token<'_>,
        note: Annotation,
        next: &Token<'_>,
        next_note: Annotation,
    ) -> bool {
        if note.abbreviation || !note.sentence_break {
            return false;
        }
        let kind = current.kind_without_period();
        if self.abbreviations.contains(&kind) || total(&self.counted.types, &kind) >= RARE {
            return false;
        }
        if current.internal_after {
            return true;
        }
        if !next.first_lower() {
            return false;
        }
        let seen = self
            .capitalized
            .orthography
            .get(&next.kind_without_sentence_period(next_note.sentence_break));
        seen.is_some_and(|seen| seen.begins_upper() && !seen.middle_upper())
    }
}

/// The third pass: finds rare abbreviations, and counts the words that follow
/// a sentence break and the pairs around an initial or a number.
fn read_pairs(
    documents: &[&str],
    abbreviations: &HashSet<String>,
    counted: &Counted,
    capitalized: &Capitalized,
) -> Pairs {
    let evidence = Evidence {
        abbreviations,
        counted,
        capitalized,
    };
    let mut pairs = Pairs::default();
    for document in documents {
        let found = tokens(document);
        let notes: Vec<Annotation> = found
            .iter()
            .map(|token| token.first_look(abbreviations))
            .collect();
        for (index, window) in found.windows(2).enumerate() {
            let [current, next] = window else { continue };
            let (Some(&note), Some(&next_note)) = (notes.get(index), notes.get(index + 1)) else {
                continue;
            };
            if current.mark != Mark::Period {
                continue;
            }
            if evidence.is_rare_abbreviation(current, note, next, next_note) {
                let kind = current.kind_without_period();
                if is_abbreviation_shaped(&kind) {
                    pairs.rare.insert(kind);
                }
            }
            let number_or_initial = current.is_initial() || current.kind() == NUMBER;
            if !note.sentence_break || current.internal_after {
                continue;
            }
            if !number_or_initial && next.is_alphabetic() {
                bump(&mut pairs.starters, &next.kind());
            }
            if number_or_initial && current.is_non_punctuation() && next.is_non_punctuation() {
                let pair = (
                    current.kind_without_period(),
                    next.kind_without_sentence_period(next_note.sentence_break),
                );
                let count = pairs.collocations.entry(pair).or_insert(0);
                *count = count.saturating_add(1);
            }
        }
    }
    pairs
}

/// The words that start sentences more often than chance.
fn find_starters(pairs: &Pairs, counted: &Counted, sentence_breaks: u64) -> HashSet<String> {
    if sentence_breaks == 0 {
        return HashSet::new();
    }
    pairs
        .starters
        .iter()
        .filter_map(|(kind, &at_break)| {
            if kind.is_empty() {
                return None;
            }
            let all = total(&counted.types, kind);
            if all < at_break {
                return None;
            }
            let likelihood = log_likelihood(
                sentence_breaks,
                u64::from(all),
                u64::from(at_break),
                counted.tokens,
            );
            let beyond_chance =
                real(counted.tokens) / real(sentence_breaks) > f64::from(all) / f64::from(at_break);
            (likelihood >= SENTENCE_STARTER && beyond_chance).then(|| kind.clone())
        })
        .collect()
}

/// The pairs an initial or a number and a word form, more often than chance.
///
/// The result maps each first word to the words that follow it.
fn find_collocations(
    pairs: &Pairs,
    counted: &Counted,
    starters: &HashSet<String>,
) -> HashMap<String, HashSet<String>> {
    let mut found: HashMap<String, HashSet<String>> = HashMap::new();
    for ((first, second), &together) in &pairs.collocations {
        if starters.contains(second) {
            continue;
        }
        let first_count = total(&counted.types, first);
        let second_count = total(&counted.types, second);
        if first_count <= 1 || second_count <= 1 {
            continue;
        }
        if together <= 1 || together > first_count.min(second_count) {
            continue;
        }
        let likelihood = log_likelihood(
            u64::from(first_count),
            u64::from(second_count),
            u64::from(together),
            counted.tokens,
        );
        let beyond_chance = real(counted.tokens) / f64::from(first_count)
            > f64::from(second_count) / f64::from(together);
        if likelihood >= COLLOCATION && beyond_chance {
            found
                .entry(first.clone())
                .or_default()
                .insert(second.clone());
        }
    }
    found
}

/// The capitalization entries of the most frequent words.
///
/// Ties go to the word that sorts first, so the cut is the same on every run.
fn most_frequent(
    orthography: HashMap<String, Orthography>,
    counted: &Counted,
) -> HashMap<String, Orthography> {
    let mut ranked: Vec<(Reverse<u32>, String, Orthography)> = orthography
        .into_iter()
        .map(|(kind, seen)| (Reverse(total(&counted.types, &kind)), kind, seen))
        .collect();
    ranked.sort_unstable_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
    ranked.truncate(ORTHOGRAPHY_ENTRIES);
    ranked
        .into_iter()
        .map(|(_, kind, seen)| (kind, seen))
        .collect()
}
