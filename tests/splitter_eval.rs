//! Evaluation of the sentence splitter on the committed sets.
//!
//! Each set in `tests/fixtures/splitter/` is a list of records, and a record
//! is its sentences joined by one space. The gold boundaries are the end of
//! every sentence but the last. The splitter's boundary F1 on a set must be at
//! least the F1 that `vale-f1.tsv` holds for Vale 3.26.0. The F1 is summed over
//! the whole set and compared in ten-thousandths, the precision of the file.
//!
//! Two checks guard the sets: no record holds two identical consecutive
//! sentences, and every MASC record holds exactly four sentences.

#![expect(
    clippy::panic_in_result_fn,
    reason = "A test loads its fixture with `?` and fails through an assertion."
)]

use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

use hakkiri::splitter::Model;

type TestResult = Result<(), Box<dyn Error>>;

/// The committed sets, by file stem.
const SETS: [&str; 4] = ["docs", "hard", "golden-rules", "masc"];

/// One record of a set: where it came from and its gold sentences.
struct Record {
    source: String,
    sentences: Vec<String>,
}

impl Record {
    /// The sentences joined by one space.
    fn text(&self) -> String {
        self.sentences.join(" ")
    }

    /// The byte offset where each sentence but the last ends in [`Self::text`].
    fn gold(&self) -> Vec<usize> {
        let mut end = 0;
        let mut ends = Vec::new();
        for (index, sentence) in self.sentences.iter().enumerate() {
            if index > 0 {
                end += 1;
            }
            end += sentence.len();
            ends.push(end);
        }
        ends.pop();
        ends
    }
}

/// The boundaries of one set: gold, predicted and correct, summed over records.
#[derive(Default)]
struct Tally {
    gold: usize,
    predicted: usize,
    correct: usize,
}

impl Tally {
    /// The F1 in ten-thousandths, rounded half up.
    ///
    /// F1 is the harmonic mean of precision and recall, which is twice the
    /// correct boundaries over the gold and predicted ones together.
    fn f1(&self) -> usize {
        let total = self.gold + self.predicted;
        if total == 0 {
            return 10_000;
        }
        (20_000 * self.correct + total / 2) / total
    }
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/splitter")
        .join(name)
}

/// Reads `tests/fixtures/splitter/<set>.fixture`.
///
/// A record starts with a `# <source>` line, holds one sentence per line, and
/// ends at a blank line.
fn records(set: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let text = fs::read_to_string(fixture_path(&format!("{set}.fixture")))?;
    let mut records: Vec<Record> = Vec::new();
    let mut open = false;
    for line in text.lines() {
        if line.is_empty() {
            open = false;
        } else if open {
            let record = records.last_mut().ok_or("a sentence before any record")?;
            record.sentences.push(line.to_owned());
        } else {
            let source = line
                .strip_prefix("# ")
                .ok_or_else(|| format!("{set}: a record starts with `# <source>`, not {line:?}"))?;
            records.push(Record {
                source: source.to_owned(),
                sentences: Vec::new(),
            });
            open = true;
        }
    }
    Ok(records)
}

/// Reads a score such as `0.9849` as ten-thousandths.
fn ten_thousandths(score: &str) -> Result<usize, Box<dyn Error>> {
    let (whole, fraction) = score
        .split_once('.')
        .ok_or_else(|| format!("{score:?} is not a score with four decimals"))?;
    if fraction.len() != 4 {
        return Err(format!("{score:?} is not a score with four decimals").into());
    }
    Ok(whole.parse::<usize>()? * 10_000 + fraction.parse::<usize>()?)
}

/// Vale's F1 for `set` from `vale-f1.tsv`, in ten-thousandths.
fn vale_f1(set: &str) -> Result<usize, Box<dyn Error>> {
    let table = fs::read_to_string(fixture_path("vale-f1.tsv"))?;
    let mut lines = table.lines();
    let header = lines.next().ok_or("vale-f1.tsv is empty")?;
    let column = header
        .split('\t')
        .position(|name| name == "f1")
        .ok_or("vale-f1.tsv has no f1 column")?;
    for line in lines {
        let cells: Vec<&str> = line.split('\t').collect();
        if cells.first() == Some(&set) {
            let score = cells
                .get(column)
                .ok_or_else(|| format!("the {set} row of vale-f1.tsv is short"))?;
            return ten_thousandths(score);
        }
    }
    Err(format!("vale-f1.tsv has no row for {set}").into())
}

/// Splits every record of `set` and counts the boundaries.
fn score(model: &Model, set: &str) -> Result<Tally, Box<dyn Error>> {
    let mut tally = Tally::default();
    for record in records(set)? {
        let text = record.text();
        let gold = record.gold();
        let mut predicted: Vec<usize> = model.split(&text).iter().map(|range| range.end).collect();
        predicted.pop();
        let found: HashSet<usize> = predicted.iter().copied().collect();
        tally.gold += gold.len();
        tally.predicted += predicted.len();
        tally.correct += gold.iter().filter(|end| found.contains(end)).count();
    }
    Ok(tally)
}

#[test]
fn hakkiri_scores_at_least_vale_on_every_set() -> TestResult {
    let model = Model::english().map_err(ToString::to_string)?;
    let mut failures = Vec::new();
    for set in SETS {
        let tally = score(model, set)?;
        let vale = vale_f1(set)?;
        eprintln!(
            "{set}: hakkiri F1 {:.4}, Vale F1 {:.4}, {} of {} gold boundaries, {} predicted",
            f64::from(u32::try_from(tally.f1())?) / 10_000.0,
            f64::from(u32::try_from(vale)?) / 10_000.0,
            tally.correct,
            tally.gold,
            tally.predicted,
        );
        if tally.f1() < vale {
            failures.push(format!(
                "{set}: hakkiri F1 {} is below Vale's {vale} (in ten-thousandths)",
                tally.f1()
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn no_record_holds_two_identical_consecutive_sentences() -> TestResult {
    let mut repeats = Vec::new();
    for set in SETS {
        for record in records(set)? {
            for pair in record.sentences.windows(2) {
                if let [first, second] = pair
                    && first == second
                {
                    repeats.push(format!("{set} {}: {first:?} twice in a row", record.source));
                }
            }
        }
    }
    assert!(repeats.is_empty(), "{}", repeats.join("\n"));
    Ok(())
}

#[test]
fn every_masc_record_holds_four_sentences() -> TestResult {
    let wrong: Vec<String> = records("masc")?
        .iter()
        .filter(|record| record.sentences.len() != 4)
        .map(|record| {
            format!(
                "{} holds {} sentences",
                record.source,
                record.sentences.len()
            )
        })
        .collect();
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    Ok(())
}
