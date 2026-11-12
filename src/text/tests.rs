//! The unit tests of `Text`.

use std::time::{Duration, Instant};

use super::{Parts, Replace, Text};

/// A text of many segments with a code span between each pair builds in time
/// that grows with its length, not with the product of its segments and spans.
#[test]
fn many_segments_and_replacements_build_in_linear_time() {
    // One segment per line, and one code span in each line, as a paragraph of
    // many lines holds them.
    let count = 100_000;
    let source = "ab `c` d\n".repeat(count);
    let (mut segments, mut replacements) = (Vec::new(), Vec::new());
    for line in 0..count {
        let at = line * 9;
        segments.push(at..at + 8);
        replacements.push((at + 3..at + 6, Replace::Code));
    }
    let start = Instant::now();
    let built = Text::build(
        &source,
        &Parts {
            segments: &segments,
            reads: &[],
            rows: &[],
            replacements: &replacements,
            decode: super::Decode::No,
        },
    );
    let took = start.elapsed();
    let built = built.map(|text| text.as_str().len());
    assert_eq!(built, Some(count * 7 - 1));
    assert!(
        took < Duration::from_secs(2),
        "building took {took:?}, which grows with segments times spans"
    );
}

/// A line break between two segments that sit inside one code span is no
/// separator, and the replacement that holds them is the one found.
#[test]
fn a_gap_inside_a_code_span_has_no_separator() {
    let source = "a `b\nc` d";
    let segments = [0..4, 5..9];
    let replacements = [(2..7, Replace::Code)];
    let built = Text::build(
        source,
        &Parts {
            segments: &segments,
            reads: &[],
            rows: &[],
            replacements: &replacements,
            decode: super::Decode::No,
        },
    );
    assert_eq!(built.as_ref().map(Text::as_str), Some("a x d"));
}
