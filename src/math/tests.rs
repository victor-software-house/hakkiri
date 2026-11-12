//! Tests for the math spans: the cases of `TestMathInline` and
//! `TestMathDisplay` in Vale, and the ones the probe of Vale added.

use markdown::{ParseOptions, to_mdast};

use super::{find, inline_spans, mask};

/// `source` masked: each inline span a code span, each display line blank.
fn masked(source: &str) -> String {
    let root = to_mdast(source, &ParseOptions::gfm()).expect("the Markdown parses");
    mask(source, &find(&root, source, false))
}

/// The spans of `source`, as the text of each.
fn spans(source: &str) -> Vec<String> {
    let root = to_mdast(source, &ParseOptions::gfm()).expect("the Markdown parses");
    find(&root, source, false)
        .inline
        .into_iter()
        .filter_map(|span| source.get(span).map(str::to_owned))
        .collect()
}

#[test]
fn a_span_has_no_space_inside_its_delimiters() {
    assert_eq!(spans("A test $g_i$ here."), ["$g_i$"]);
    assert_eq!(spans("A test $g_i = p_i$ here."), ["$g_i = p_i$"]);
    assert_eq!(spans("A test $ g_i$ here."), Vec::<String>::new());
    assert_eq!(spans("A test $g_i $ here."), Vec::<String>::new());
}

#[test]
fn money_is_not_math() {
    assert_eq!(spans("It costs $5 and $10 today."), Vec::<String>::new());
    assert_eq!(spans("Between US$5 and CA$10 there."), Vec::<String>::new());
    assert_eq!(spans("A lone $5 in a sentence."), Vec::<String>::new());
}

#[test]
fn an_escaped_delimiter_is_text() {
    assert_eq!(
        spans("It costs \\$5 and \\$10 today."),
        Vec::<String>::new()
    );
}

#[test]
fn a_span_wraps_over_the_lines_of_a_paragraph() {
    assert_eq!(spans("A test $g_i =\ng(p)_i$ here."), ["$g_i =\ng(p)_i$"]);
    assert_eq!(spans("A test $g_i\n\ng(p)_i$ here."), Vec::<String>::new());
}

#[test]
fn a_closing_dollar_is_not_followed_by_a_digit() {
    assert_eq!(
        spans("After $a$5 very then $b$ very."),
        ["$a$5 very then $b$"]
    );
}

#[test]
fn a_code_span_holds_no_math() {
    assert_eq!(spans("A test `$g_i$` here."), Vec::<String>::new());
    assert_eq!(spans("A test ``$g_i$`` here."), Vec::<String>::new());
}

#[test]
fn a_double_dollar_in_a_line_opens_at_the_second() {
    assert_eq!(spans("Line $$x very$$ inline."), ["$x very$"]);
}

#[test]
fn a_heading_and_a_table_cell_hold_spans() {
    assert_eq!(spans("# A $very$ heading"), ["$very$"]);
    assert_eq!(spans("| a | b |\n|:--|:--|\n| $very$ | c |\n"), ["$very$"]);
}

#[test]
fn an_inline_span_becomes_a_code_span_of_the_same_length() {
    assert_eq!(masked("A $x$ b"), "A `x` b");
    assert_eq!(masked("A $a `b` c$ d"), "A `a 'b' c` d");
}

/// `text` with every character blank, as a masked line of display math is.
fn blank(text: &str) -> String {
    " ".repeat(text.chars().count())
}

#[test]
fn a_display_block_becomes_blank_lines() {
    assert_eq!(
        masked("Intro.\n\n$$\nx = 1\n$$\n\nAfter."),
        format!(
            "Intro.\n\n{}\n{}\n{}\n\nAfter.",
            blank("$$"),
            blank("x = 1"),
            blank("$$")
        )
    );
    assert_eq!(
        masked("Intro.\n\n$$x = 1$$\n\nAfter."),
        format!("Intro.\n\n{}\n\nAfter.", blank("$$x = 1$$"))
    );
}

#[test]
fn a_block_can_close_with_a_label_or_after_content() {
    assert_eq!(
        masked("$$\nx = 1\n$$ {#eq-foo}\n\nAfter."),
        format!(
            "{}\n{}\n{}\n\nAfter.",
            blank("$$"),
            blank("x = 1"),
            blank("$$ {#eq-foo}")
        )
    );
    assert_eq!(
        masked("$$\\begin{a}\nx\n\\end{a}$$\n\nAfter."),
        format!(
            "{}\n{}\n{}\n\nAfter.",
            blank("$$\\begin{a}"),
            blank("x"),
            blank("\\end{a}$$")
        )
    );
}

#[test]
fn a_block_opens_in_the_middle_of_a_paragraph() {
    assert_eq!(
        masked("Mid para\n$$\nz\n$$ {#eq}\ntail."),
        format!(
            "Mid para\n{}\n{}\n{}\ntail.",
            blank("$$"),
            blank("z"),
            blank("$$ {#eq}")
        )
    );
}

#[test]
fn an_unclosed_block_ends_with_the_paragraph() {
    assert_eq!(
        masked("$$\nunclosed\n\nAfter blank."),
        "  \n        \n\nAfter blank."
    );
}

#[test]
fn an_escaped_closer_does_not_close_a_block() {
    assert_eq!(masked("$$\nx \\$$\ny\n$$"), "  \n     \n \n  ");
}

#[test]
fn a_quote_keeps_its_markers_around_a_block() {
    assert_eq!(
        masked("> Quote\n> $$\n> q\n> $$"),
        "> Quote\n>   \n>  \n>   "
    );
}

#[test]
fn a_run_of_backticks_that_opens_no_code_span_cannot_pair_with_a_span() {
    assert_eq!(masked("A ` b $x$ c"), "A ' b `x` c");
    assert_eq!(masked("A ``$x$ c"), "A ''`x` c");
    assert_eq!(masked("A $x$ then ` c"), "A `x` then ' c");
    assert_eq!(masked("A ` b c"), "A ` b c");
}

#[test]
fn a_code_span_and_a_span_that_touch_are_one_code_span() {
    assert_eq!(masked("A `x`$y$ b"), "A `x``y` b");
    assert_eq!(masked("A $x$$y$ b"), "A `x``y` b");
}

#[test]
fn a_line_indented_four_columns_or_more_opens_no_block() {
    let open = |text: &str| !masked(text).contains('z');
    assert!(open("Para\n   $$\n   z\n   $$\ntail."));
    assert!(!open("Para\n    $$\n    z\n    $$\ntail."));
    assert!(!open("Para\n\t$$\n\tz\n\t$$\ntail."));
}

#[test]
fn the_indentation_of_a_container_is_not_the_indentation_of_a_line() {
    let open = |text: &str| !masked(text).contains('z');
    assert!(open("- Item\n    $$\n    z\n    $$\n- Two\n"));
    assert!(!open("- Item\n      $$\n      z\n      $$\n- Two\n"));
    assert!(open("1. Item\n   $$\n   z\n   $$\n"));
    assert!(!open("1. Item\n       $$\n       z\n       $$\n"));
    assert!(open("> Quote\n>   $$\n>   z\n>   $$\n"));
    assert!(!open("> Quote\n>     $$\n>     z\n>     $$\n"));
}

#[test]
fn a_paragraph_of_unclosed_openers_costs_its_length() {
    let started = std::time::Instant::now();
    for text in [
        "$a ".repeat(100_000),
        "<a".repeat(100_000),
        "$a$5 ".repeat(100_000),
    ] {
        let _ = inline_spans(&text, 0, false);
    }
    assert!(
        started.elapsed() < std::time::Duration::from_secs(5),
        "{:?}",
        started.elapsed()
    );
}

#[test]
fn a_line_without_the_marker_of_a_quote_keeps_its_spaces() {
    let open = |text: &str| !masked(text).contains('z');
    assert!(open("> Quote\n   $$\n   z\n   $$\n"));
    assert!(!open("> Quote\n      $$\n      z\n      $$\n"));
}

#[test]
fn a_quote_in_a_list_item_and_a_list_item_in_a_quote_count_both_containers() {
    let open = |text: &str| !masked(text).contains('z');
    assert!(open("> - Item\n>     $$\n>     z\n>     $$\n"));
    assert!(!open("> - Item\n>       $$\n>       z\n>       $$\n"));
    assert!(open("- Item\n  > Quote\n  >   $$\n  >   z\n  >   $$\n"));
    assert!(!open(
        "- Item\n  > Quote\n  >     $$\n  >     z\n  >     $$\n"
    ));
}
