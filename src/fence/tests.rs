//! The unit tests of the place of a fence's code in its file.

use std::iter;

use super::Placement;

/// The place of the code `code` of the fence that is all of `source`.
fn placement(source: &str, code: &str) -> Option<Placement> {
    Placement::new(source, &(0..source.len()), code)
}

#[test]
fn a_line_that_ends_the_line_of_the_file_has_its_place() {
    let source = "> ```rust\n>   // a\n> let x;\n> ```\n";
    let placed = placement(source, "  // a\nlet x;");
    assert_eq!(placed.as_ref().map(|placed| placed.place(2)), Some(14));
    assert_eq!(placed.map(|placed| placed.place(8)), Some(22));
}

#[test]
fn a_line_that_does_not_end_the_line_of_the_file_gives_no_place() {
    let source = "```rust\n// a\n```\n";
    assert!(placement(source, "// b").is_none());
    assert!(placement(source, "// a\n// a").is_none());
}

#[test]
fn a_range_across_lines_is_one_range_for_each_line() {
    let source = "```rust\n/* a\n b */\n```\n";
    let placed = placement(source, "/* a\n b */");
    let ranges = placed.map(|placed| placed.ranges(0..10));
    assert_eq!(ranges, Some(vec![8..12, 13..18]));
}

#[test]
fn a_carriage_return_is_no_byte_of_the_file() {
    let source = "```rust\r\n// a\r\n```\r\n";
    let placed = placement(source, "// a\r");
    let ranges = placed.map(|placed| placed.ranges(0..5));
    assert_eq!(ranges, Some(iter::once(9..13).collect()));
}
