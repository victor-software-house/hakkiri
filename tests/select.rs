//! Selection: the scope-selection scenarios of the rule-engine change, and the
//! scope scenarios of the rule-files spec.
//!
//! Each test selects the blocks a scope reads in one input, and checks their
//! text and where it lies in the file.

#![expect(
    clippy::panic_in_result_fn,
    reason = "A test returns Result for `?` and fails through an assertion."
)]

use std::error::Error;
use std::fs;
use std::path::Path;

use hakkiri::scope::Selector;
use hakkiri::select::{Block, select};
use hakkiri::words::words;
use hakkiri::{Language, Scope, parse};

type TestResult = Result<(), Box<dyn Error>>;

/// The blocks that `scope` reads in `source`, a file named `file`.
fn blocks(scope: &[&str], file: &str, source: &str) -> Result<Vec<Block>, Box<dyn Error>> {
    let path = Path::new(file);
    let language = Language::from_path(path).ok_or("a file with an unknown extension")?;
    let tree = parse(language, source)?;
    let items: Vec<String> = scope.iter().map(|item| (*item).to_owned()).collect();
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .ok_or("a file with no extension")?;
    Ok(select(&Selector::parse(&items)?, &tree, source, extension)?)
}

fn texts(blocks: &[Block]) -> Vec<&str> {
    blocks.iter().map(|block| block.text.as_str()).collect()
}

#[test]
fn heading_selects_every_level() -> TestResult {
    let found = blocks(&["heading"], "a.md", "# A marker\n\n### Another marker\n")?;
    assert_eq!(texts(&found), ["A marker", "Another marker"]);
    assert_eq!(found[0].scope, Scope::Heading1);
    assert_eq!(found[1].scope, Scope::Heading3);
    Ok(())
}

#[test]
fn text_comment_selects_every_comment() -> TestResult {
    let found = blocks(
        &["text.comment"],
        "a.rs",
        "// A marker.\n\n/// Another marker.\n",
    )?;
    assert_eq!(texts(&found), ["A marker.", "Another marker."]);
    assert_eq!(found[0].scope, Scope::CommentLine);
    assert_eq!(found[1].scope, Scope::CommentDocLine);
    Ok(())
}

#[test]
fn a_comment_reads_without_its_markers() -> TestResult {
    let source = "fn f() {}\n    /// A marker.\n";
    let found = blocks(&["text.comment"], "a.rs", source)?;
    let block = &found[0];
    let at = block.text.as_str().find("marker").ok_or("no marker")?;
    let range = block
        .text
        .file_range(at..at + "marker".len())
        .ok_or("no range")?;
    assert_eq!(&source[range.clone()], "marker");
    let line_start = source[..range.start].rfind('\n').map_or(0, |at| at + 1);
    // Columns 11 to 16 of the line, counted from 1.
    assert_eq!(range.start - line_start + 1, 11);
    Ok(())
}

#[test]
fn text_selects_the_prose_leaves() -> TestResult {
    let found = blocks(&["text"], "a.md", "# A marker\n\nA marker.\n\n- A marker\n")?;
    assert_eq!(texts(&found), ["A marker", "A marker.", "A marker"]);
    Ok(())
}

#[test]
fn text_reaches_a_paragraph_in_a_comment() -> TestResult {
    let found = blocks(&["text"], "a.rs", "// A marker.\n")?;
    assert_eq!(texts(&found), ["A marker."]);
    assert_eq!(found[0].scope, Scope::Paragraph);
    Ok(())
}

#[test]
fn an_unset_scope_reads_as_text() -> TestResult {
    let found = blocks(&[], "a.md", "| Name |\n|:--|\n| marker |\n")?;
    assert_eq!(texts(&found), ["Name", "marker"]);
    Ok(())
}

#[test]
fn paragraph_leaves_out_list_items_and_quotes() -> TestResult {
    let found = blocks(
        &["paragraph"],
        "a.md",
        "- A marker item.\n\nA marker.\n\n> A quoted marker.\n",
    )?;
    assert_eq!(texts(&found), ["A marker."]);
    Ok(())
}

#[test]
fn paragraph_skips_a_heading() -> TestResult {
    let source = "# A marker\n\nAnother marker.\n";
    let found = blocks(&["paragraph"], "a.md", source)?;
    assert_eq!(texts(&found), ["Another marker."]);
    let start = found[0].text.file_offset(0).ok_or("no offset")?;
    assert_eq!(source[..start].matches('\n').count() + 1, 3);
    Ok(())
}

#[test]
fn sentence_selects_each_sentence() -> TestResult {
    let found = blocks(&["sentence"], "a.md", "One two. Three four five six.\n")?;
    assert_eq!(texts(&found), ["One two.", "Three four five six."]);
    assert!(found.iter().all(|block| block.sentence));
    Ok(())
}

#[test]
fn a_part_selects_the_sentences_inside_it() -> TestResult {
    let found = blocks(&["sentence & list"], "a.md", "One two.\n\n- Three four.\n")?;
    assert_eq!(texts(&found), ["Three four."]);
    Ok(())
}

#[test]
fn a_heading_rule_reads_the_whole_heading() -> TestResult {
    let found = blocks(&["heading"], "a.md", "# One two. Three four.\n")?;
    assert_eq!(texts(&found), ["One two. Three four."]);
    Ok(())
}

#[test]
fn a_file_extension_narrows_a_scope() -> TestResult {
    assert_eq!(blocks(&["raw.rs"], "a.md", "A marker.\n")?, Vec::new());
    let found = blocks(&["raw.rs"], "a.rs", "// A marker.\n")?;
    assert_eq!(found.len(), 1);
    Ok(())
}

#[test]
fn an_inline_exclusion_blanks_the_element() -> TestResult {
    let source = "See marker and [marker](https://x.example).\n";
    let found = blocks(&["text & ~link"], "a.md", source)?;
    assert_eq!(texts(&found), ["See marker and       ."]);
    let text = &found[0].text;
    let first = text.as_str().find("marker").ok_or("no marker")?;
    // Line 1, column 5: the offset is 4.
    assert_eq!(text.file_offset(first), Some(4));
    assert_eq!(text.as_str().matches("marker").count(), 1);
    Ok(())
}

#[test]
fn a_phrase_reads_across_markup() -> TestResult {
    let found = blocks(
        &["text"],
        "a.md",
        "A **bold** [link](https://a.example) here.\n",
    )?;
    let text = &found[0].text;
    assert_eq!(text.as_str(), "A bold link here.");
    let at = text.as_str().find("bold link").ok_or("no phrase")?;
    assert_eq!(text.file_offset(at), Some(4));
    Ok(())
}

#[test]
fn a_code_span_reads_as_one_word() -> TestResult {
    let found = blocks(&["text"], "a.md", "Then run `cargo test` now.\n")?;
    let text = &found[0].text;
    assert_eq!(text.as_str(), "Then run x now.");
    assert!(text.overlaps_code(&(5..10)));
    assert!(!text.overlaps_code(&(0..4)));
    Ok(())
}

#[test]
fn a_word_count_with_a_code_span() -> TestResult {
    let found = blocks(
        &["sentence"],
        "a.md",
        "One two `alpha beta gamma delta` five.\n",
    )?;
    assert_eq!(texts(&found), ["One two x five."]);
    assert_eq!(words(found[0].text.as_str()).len(), 4);
    Ok(())
}

#[test]
fn a_word_count_with_a_link() -> TestResult {
    let found = blocks(
        &["sentence"],
        "a.md",
        "One two [three](https://example.com/a/b/c) four five.\n",
    )?;
    assert_eq!(texts(&found), ["One two three four five."]);
    assert_eq!(words(found[0].text.as_str()).len(), 5);
    Ok(())
}

#[test]
fn a_list_of_scopes() -> TestResult {
    let source = "# A marker heading\n\n| marker |\n|:--|\n| cell |\n\nA marker.\n";
    let found = blocks(&["heading", "table.header"], "a.md", source)?;
    assert_eq!(texts(&found), ["A marker heading", "marker"]);
    Ok(())
}

#[test]
fn an_exclusion_leaves_out_blocks_of_that_scope() -> TestResult {
    let found = blocks(
        &["sentence & ~blockquote"],
        "a.md",
        "A marker here.\n\n> - A marker in a list.\n",
    )?;
    assert_eq!(texts(&found), ["A marker here."]);
    let found = blocks(
        &["sentence & ~blockquote"],
        "a.md",
        "A marker here.\n\n> Quote text.\n>\n> - A marker in a list.\n",
    )?;
    assert_eq!(texts(&found), ["A marker here.", "A marker in a list."]);
    Ok(())
}

#[test]
fn a_block_comment_and_a_doc_comment() -> TestResult {
    let found = blocks(&["text.comment.block"], "a.rs", "/* A marker here. */\n")?;
    assert_eq!(texts(&found), ["A marker here."]);
    let found = blocks(
        &["text.comment.doc"],
        "a.rs",
        "// A marker here.\n/// A marker there.\n",
    )?;
    assert_eq!(texts(&found), ["A marker there."]);
    Ok(())
}

#[test]
fn a_url_in_a_comment_is_a_link() -> TestResult {
    let source = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/url-comments.rs.fixture"),
    )?;
    let found = blocks(&["link"], "a.rs", &source)?;
    assert_eq!(texts(&found), ["https://bare.example"]);
    Ok(())
}

#[test]
fn a_string_literal_is_never_prose() -> TestResult {
    let found = blocks(
        &["raw"],
        "a.rs",
        "let s = \"basically fine\";\n// basically fine\n",
    )?;
    assert_eq!(texts(&found), ["// basically fine"]);
    Ok(())
}

#[test]
fn an_unknown_scope_is_refused() {
    let error = Selector::parse(&["chapter".to_owned()]).err();
    assert!(error.is_some_and(|error| error.to_string().contains("chapter")));
}

#[test]
fn words_follow_the_prose_pattern() {
    let counted = |text: &str| -> Vec<String> {
        words(text)
            .into_iter()
            .map(|range| text[range].to_owned())
            .collect()
    };
    assert_eq!(
        counted("Don't stop the U.S. team."),
        ["Don't", "stop", "the", "U.S.", "team"]
    );
    assert_eq!(counted("Ab, 12 c."), ["Ab", "12", "c"]);
    assert_eq!(counted("naïve café"), ["naïve", "café"]);
    assert_eq!(counted("U.S.A"), ["U.S.", "A"]);
    assert_eq!(counted("-- ;;"), Vec::<String>::new());
}

#[test]
fn a_combining_mark_ends_a_word() {
    // U+0345 has the Alphabetic property but is a mark, not a letter, so
    // prose's `\p{L}` stops at it.
    let text = "a\u{0345}b";
    let found: Vec<&str> = words(text).into_iter().map(|range| &text[range]).collect();
    assert_eq!(found, ["a", "b"]);
}

#[test]
fn a_pipe_in_a_link_destination_outside_a_table_reads_as_nothing() -> TestResult {
    let found = blocks(
        &["paragraph"],
        "a.md",
        "Open [the page](https://x.example/a|b) now.\n",
    )?;
    assert_eq!(texts(&found), ["Open the page now."]);
    Ok(())
}

#[test]
fn a_negated_part_with_another_extension_leaves_nothing_out() -> TestResult {
    let source = "# Head\n\nBody.\n";
    let found = blocks(&["text & ~heading.rs"], "a.md", source)?;
    assert_eq!(texts(&found), ["Head", "Body."]);
    let found = blocks(&["text & ~heading.md"], "a.md", source)?;
    assert_eq!(texts(&found), ["Body."]);
    Ok(())
}

#[test]
fn an_item_of_parts_for_another_language_selects_nothing() -> TestResult {
    let source = "# Head\n\nBody.\n";
    let found = blocks(&["~heading.rs"], "a.md", source)?;
    assert_eq!(texts(&found), Vec::<&str>::new());
    let found = blocks(&["~heading.md"], "a.md", source)?;
    assert_eq!(texts(&found), ["Body."]);
    Ok(())
}

#[test]
fn an_exclusion_belongs_to_its_item() -> TestResult {
    let source = "# See [the guide](u.md) now\n\nSee [the guide](u.md) now.\n";
    let found = blocks(&["heading", "paragraph & ~link"], "a.md", source)?;
    assert_eq!(texts(&found), ["See the guide now", "See           now."]);
    Ok(())
}

#[test]
fn a_later_paragraph_of_an_item_is_a_paragraph() -> TestResult {
    let source = "- First.\n\n  A marker here.\n";
    let found = blocks(&["list"], "a.md", source)?;
    assert_eq!(texts(&found), ["First."]);
    let found = blocks(&["paragraph"], "a.md", source)?;
    assert_eq!(texts(&found), ["A marker here."]);
    Ok(())
}

#[test]
fn a_heading_inside_a_quote_is_a_heading() -> TestResult {
    let source = "> A quote.\n>\n> # A heading\n";
    let found = blocks(&["blockquote"], "a.md", source)?;
    assert_eq!(texts(&found), ["A quote."]);
    let found = blocks(&["heading"], "a.md", source)?;
    assert_eq!(texts(&found), ["A heading"]);
    Ok(())
}

#[test]
fn a_quote_that_opens_an_item_is_the_item() -> TestResult {
    let source = "- > Opens the item.\n\n- First.\n\n  > Second quote.\n";
    let found = blocks(&["list"], "a.md", source)?;
    assert_eq!(texts(&found), ["Opens the item.", "First."]);
    let found = blocks(&["blockquote"], "a.md", source)?;
    assert_eq!(texts(&found), ["Second quote."]);
    Ok(())
}

#[test]
fn a_dotted_sentence_is_a_sentence_in_the_scope() -> TestResult {
    let source = "A paragraph.\n\n- An item.\n";
    let found = blocks(&["sentence.list"], "a.md", source)?;
    assert_eq!(texts(&found), ["An item."]);
    Ok(())
}

#[test]
fn table_selects_cells_and_not_rows() -> TestResult {
    let source = "| H1 | H2 |\n|:--|:--|\n| B1 | B2 |\n";
    let found = blocks(&["table"], "a.md", source)?;
    assert_eq!(texts(&found), ["H1", "H2", "B1", "B2"]);
    let found = blocks(&["table.row"], "a.md", source)?;
    assert_eq!(found.len(), 2);
    Ok(())
}

#[test]
fn a_paragraph_after_a_fenced_block_in_an_item_is_a_paragraph() -> TestResult {
    let source = "- ```text\n  code\n  ```\n\n  After the fence.\n";
    let found = blocks(&["paragraph"], "a.md", source)?;
    assert_eq!(texts(&found), ["After the fence."]);
    let found = blocks(&["list"], "a.md", source)?;
    assert_eq!(texts(&found), Vec::<&str>::new());
    Ok(())
}

#[test]
fn a_paragraph_after_a_fenced_block_in_a_quote_is_a_paragraph() -> TestResult {
    let source = "> ```text\n> code\n> ```\n>\n> After the fence.\n";
    let found = blocks(&["paragraph"], "a.md", source)?;
    assert_eq!(texts(&found), ["After the fence."]);
    let found = blocks(&["blockquote"], "a.md", source)?;
    assert_eq!(texts(&found), Vec::<&str>::new());
    Ok(())
}

#[test]
fn a_quote_in_a_list_item_has_its_first_paragraph_as_the_quote() -> TestResult {
    let source = "- > Para one.\n  >\n  > Para two.\n";
    let found = blocks(&["blockquote"], "a.md", source)?;
    assert_eq!(texts(&found), ["Para one."]);
    let found = blocks(&["paragraph"], "a.md", source)?;
    assert_eq!(texts(&found), ["Para two."]);
    Ok(())
}
