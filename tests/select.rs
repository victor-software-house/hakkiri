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
use std::time::{Duration, Instant};

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
    let own = langdex::by_path(file);
    Ok(select(
        &Selector::parse(&items)?,
        &tree,
        source,
        own.as_slice(),
    )?)
}

/// The lines joined with a line feed after each.
fn concat_lines(lines: &[&str]) -> String {
    let mut text = lines.join("\n");
    text.push('\n');
    text
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
fn title_selects_the_title_of_a_link_an_image_and_a_definition() -> TestResult {
    let source = concat_lines(&[
        "A [link](u.html \"a marker\") and ![img](i.png 'an marker') end.",
        "",
        "[ref]: u.html (a third marker)",
        "",
    ]);
    let found = blocks(&["title"], "a.md", &source)?;
    assert_eq!(texts(&found), ["a marker", "an marker", "a third marker"]);
    assert!(found.iter().all(|block| block.scope == Scope::Title));
    Ok(())
}

#[test]
fn a_title_of_a_long_run_of_backslashes_costs_its_length() -> TestResult {
    let run = "\\\\".repeat(100_000);
    let source = format!("[a](u.html \"{run}\")\n");
    let started = Instant::now();
    let found = blocks(&["title"], "a.md", &source)?;
    assert_eq!(texts(&found), [run.as_str()]);
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "{:?}",
        started.elapsed()
    );
    Ok(())
}

/// A link, an image and a definition, each with a title that holds `marker`.
fn titled() -> String {
    concat_lines(&[
        "A [link](u.html \"a link marker\") and ![img](i.png 'an image marker') end.",
        "",
        "[ref]: u.html (a definition marker)",
        "",
    ])
}

#[test]
fn title_and_link_select_only_the_title_of_a_link() -> TestResult {
    let source = titled();
    let found = |scope: &str| -> Result<Vec<String>, Box<dyn Error>> {
        Ok(texts(&blocks(&[scope], "a.md", &source)?)
            .into_iter()
            .map(str::to_owned)
            .collect())
    };
    assert_eq!(found("title & link")?, ["a link marker"]);
    assert_eq!(found("title & image")?, ["an image marker"]);
    assert_eq!(
        found("title")?,
        ["a link marker", "an image marker", "a definition marker"]
    );
    // The element around a title is no scope of the title itself.
    assert!(found("link")?.iter().all(|text| !text.contains("marker")));
    assert!(found("image")?.iter().all(|text| !text.contains("marker")));
    Ok(())
}

#[test]
fn a_negated_element_leaves_out_the_titles_it_holds() -> TestResult {
    let source = titled();
    let found = |scope: &str| -> Result<Vec<String>, Box<dyn Error>> {
        Ok(texts(&blocks(&[scope], "a.md", &source)?)
            .into_iter()
            .map(str::to_owned)
            .collect())
    };
    assert_eq!(
        found("title & ~link")?,
        ["an image marker", "a definition marker"]
    );
    assert_eq!(
        found("title & ~image")?,
        ["a link marker", "a definition marker"]
    );
    let html = concat_lines(&[
        "<p><a href=\"u\" title=\"an a marker\">x</a></p>",
        "",
        "<img src=\"i.png\" title=\"an img marker\">",
        "",
    ]);
    let html = html.as_str();
    let in_html = |scope: &str| -> Result<Vec<String>, Box<dyn Error>> {
        Ok(texts(&blocks(&[scope], "a.md", html)?)
            .into_iter()
            .map(str::to_owned)
            .collect())
    };
    assert_eq!(in_html("title & ~link")?, ["an img marker"]);
    assert_eq!(in_html("title & ~image")?, ["an a marker"]);
    Ok(())
}

#[test]
fn image_selects_the_alt_text_of_an_image() -> TestResult {
    let found = blocks(
        &["image"],
        "a.md",
        "A ![an alt marker](i.png \"t\") and ![](j.png \"u\").\n",
    )?;
    assert_eq!(texts(&found), ["an alt marker"]);
    let found = blocks(
        &["title & image"],
        "a.md",
        "A ![](j.png \"an empty marker\") end.\n",
    )?;
    assert_eq!(texts(&found), ["an empty marker"]);
    Ok(())
}

#[test]
fn a_title_in_a_heading_and_in_a_table_cell_is_a_link_title() -> TestResult {
    let source = concat_lines(&[
        "# A [heading](u.html \"a heading marker\")",
        "",
        "| Cell |",
        "|:--|",
        "| [cell](u.html \"a cell marker\") |",
        "",
    ]);
    let found = blocks(&["title & link"], "a.md", &source)?;
    assert_eq!(texts(&found), ["a heading marker", "a cell marker"]);
    Ok(())
}

#[test]
fn title_selects_the_title_attribute_of_an_html_block() -> TestResult {
    let source = "<p title=\"an html marker\">Some text.</p>\n";
    let found = blocks(&["title"], "a.md", source)?;
    assert_eq!(texts(&found), ["an html marker"]);
    let found = blocks(&["text"], "a.md", source)?;
    assert_eq!(texts(&found), ["Some text."]);
    Ok(())
}

#[test]
fn title_and_link_select_the_title_of_an_html_a_tag_and_title_and_image_that_of_an_img_tag()
-> TestResult {
    let source = concat_lines(&[
        "<p>See <a href=\"u.html\" title=\"an a marker\">the link</a>.</p>",
        "",
        "<img src=\"i.png\" alt=\"an alt\" title=\"an img marker\">",
        "",
        "<div title=\"a div marker\">Some text.</div>",
        "",
    ]);
    let found = |scope: &str| -> Result<Vec<String>, Box<dyn Error>> {
        Ok(texts(&blocks(&[scope], "a.md", &source)?)
            .into_iter()
            .map(str::to_owned)
            .collect())
    };
    assert_eq!(found("title & link")?, ["an a marker"]);
    assert_eq!(found("title & image")?, ["an img marker"]);
    assert_eq!(
        found("title")?,
        ["an a marker", "an img marker", "a div marker"]
    );
    // The tag around a title is no scope of the title itself.
    assert!(found("link")?.iter().all(|text| !text.contains("marker")));
    assert!(found("image")?.iter().all(|text| !text.contains("marker")));
    Ok(())
}

#[test]
fn raw_still_reads_an_html_title_in_a_block_and_in_a_line() -> TestResult {
    let block = "<p title=\"an html marker\">Some text.</p>\n";
    let line = "A <span title=\"a line marker\">word</span> here.\n";
    let raw = |source: &str, marker: &str| -> Result<bool, Box<dyn Error>> {
        Ok(texts(&blocks(&["raw"], "a.md", source)?)
            .iter()
            .any(|text| text.contains(marker)))
    };
    assert!(raw(block, "an html marker")?);
    assert!(raw(line, "a line marker")?);
    // In a line of text, as in Vale, no prose scope reads the attribute.
    let text = blocks(&["text"], "a.md", line)?;
    assert!(texts(&text).iter().all(|text| !text.contains("marker")));
    Ok(())
}

#[test]
fn title_selects_the_title_attribute_of_inline_html_and_of_a_jsx_element() -> TestResult {
    let source = concat_lines(&[
        "A <a href=\"u\" title=\"an a marker\">x</a> and <span title=\"a span marker\">y</span>",
        "and <img src=\"i.png\" title=\"an img marker\"> end.",
        "",
        "# A heading <b title=\"a heading marker\">x</b>",
        "",
        "| Cell |",
        "|:--|",
        "| <i title=\"a cell marker\">x</i> |",
        "",
        "- An item <u title=\"an item marker\">x</u>",
        "",
        "> A quote <a title=\"a quote marker\"",
        "> href=\"u\">x</a> here.",
        "",
    ]);
    let found = |scope: &str| -> Result<Vec<String>, Box<dyn Error>> {
        Ok(texts(&blocks(&[scope], "a.md", &source)?)
            .into_iter()
            .map(str::to_owned)
            .collect())
    };
    assert_eq!(
        found("title")?,
        [
            "an a marker",
            "a span marker",
            "an img marker",
            "a heading marker",
            "a cell marker",
            "an item marker",
            "a quote marker",
        ]
    );
    assert_eq!(found("title & link")?, ["an a marker", "a quote marker"]);
    assert_eq!(found("title & image")?, ["an img marker"]);
    assert!(found("text")?.iter().all(|text| !text.contains("marker")));
    let mdx = "A <Foo title=\"a jsx marker\" /> and <Bar title=\"a block marker\">text</Bar>.\n";
    let found = blocks(&["title"], "a.mdx", mdx)?;
    assert_eq!(texts(&found), ["a jsx marker", "a block marker"]);
    let rust = "/// A <a title=\"a doc marker\" href=\"u\">x</a> here.\nfn f() {}\n";
    let found = blocks(&["title"], "a.rs", rust)?;
    assert_eq!(texts(&found), ["a doc marker"]);
    Ok(())
}

#[test]
fn the_title_of_a_tag_in_link_text_is_that_tags_and_not_the_links() -> TestResult {
    let source = concat_lines(&[
        "[A <span title=\"a nested marker\">x</span> end](u \"a link marker\")",
        "",
        "[<img src=\"i.png\" title=\"an inner img marker\"> end](u)",
        "",
        "[A <i title=\"a reference marker\">y</i>][r]",
        "",
        "[r]: u \"a definition marker\"",
        "",
    ]);
    let found = |scope: &str| -> Result<Vec<String>, Box<dyn Error>> {
        let mut found: Vec<String> = texts(&blocks(&[scope], "a.md", &source)?)
            .into_iter()
            .map(str::to_owned)
            .collect();
        found.sort();
        Ok(found)
    };
    assert_eq!(
        found("title")?,
        [
            "a definition marker",
            "a link marker",
            "a nested marker",
            "a reference marker",
            "an inner img marker",
        ]
    );
    assert_eq!(found("title & link")?, ["a link marker"]);
    assert_eq!(found("title & image")?, ["an inner img marker"]);
    assert_eq!(
        found("title & ~link")?,
        [
            "a definition marker",
            "a nested marker",
            "a reference marker",
            "an inner img marker",
        ]
    );
    Ok(())
}

#[test]
fn no_prose_scope_reads_a_title() -> TestResult {
    let source = concat_lines(&[
        "A [link](u.html \"a title marker\") and ![img](i.png 'an image marker') end.",
        "",
        "[ref]: u.html (a definition marker)",
        "",
    ]);
    for scope in ["text", "paragraph", "link", "alt", "sentence"] {
        for block in blocks(&[scope], "a.md", &source)? {
            assert!(
                !block.text.as_str().contains("title marker")
                    && !block.text.as_str().contains("image marker")
                    && !block.text.as_str().contains("definition marker"),
                "scope {scope} read {:?}",
                block.text.as_str()
            );
        }
    }
    Ok(())
}

#[test]
fn a_link_without_a_title_and_an_escaped_quote_give_the_right_title() -> TestResult {
    let source = concat_lines(&[
        "A [plain](u.html) and [quoted](u.html \"say \\\"hi\\\" marker\") and [empty](u.html \"\") end.",
        "",
    ]);
    let found = blocks(&["title"], "a.md", &source)?;
    assert_eq!(texts(&found), ["say \\\"hi\\\" marker"]);
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
fn raw_rs_in_two_files() -> TestResult {
    assert_eq!(blocks(&["raw.rs"], "a.md", "A marker.\n")?, Vec::new());
    let found = blocks(&["raw.rs"], "a.rs", "// A marker.\n")?;
    assert_eq!(found.len(), 1);
    Ok(())
}

#[test]
fn one_language_several_extensions() -> TestResult {
    for file in ["a.js", "a.jsx", "a.mjs", "a.cjs"] {
        let found = blocks(&["text.comment.js"], file, "// A marker.\n")?;
        assert_eq!(found.len(), 1, "{file}");
    }
    Ok(())
}

#[test]
fn a_name_of_the_language() -> TestResult {
    let found = blocks(&["text.comment.rust"], "a.rs", "// A marker.\n")?;
    assert_eq!(found.len(), 1);
    Ok(())
}

#[test]
fn a_scope_name_that_ends_like_a_language() -> TestResult {
    for file in ["a.rs", "a.js"] {
        let found = blocks(&["text.comment.line"], file, "// A marker.\n")?;
        assert_eq!(found.len(), 1, "{file}");
    }
    Ok(())
}

#[test]
fn a_piece_that_names_no_language() {
    let refused = Selector::parse(&["text.nolang".to_owned()]);
    assert_eq!(
        refused.map_err(|error| error.to_string()),
        Err("`text.nolang` is not a scope hakkiri defines".to_owned())
    );
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
    // U+0345 has the Alphabetic property but is a mark and no letter, so
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
fn a_negated_part_for_another_language() -> TestResult {
    let source = "# Head\n\nBody.\n";
    let found = blocks(&["text & ~heading.rs"], "a.md", source)?;
    assert_eq!(texts(&found), ["Head", "Body."]);
    let found = blocks(&["text & ~heading.md"], "a.md", source)?;
    assert_eq!(texts(&found), ["Body."]);
    Ok(())
}

#[test]
fn an_item_that_names_only_another_language() -> TestResult {
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
fn a_quote_in_a_list_item_gives_its_first_paragraph_to_the_item() -> TestResult {
    let source = "- > Para one.\n  >\n  > Para two.\n";
    let found = blocks(&["list"], "a.md", source)?;
    assert_eq!(texts(&found), ["Para one."]);
    let found = blocks(&["paragraph"], "a.md", source)?;
    assert_eq!(texts(&found), ["Para two."]);
    let found = blocks(&["blockquote"], "a.md", source)?;
    assert_eq!(texts(&found), Vec::<&str>::new());
    Ok(())
}

#[test]
fn a_front_matter_part_ends_in_the_extension_of_a_language() -> TestResult {
    let source = "---\ntitle: A very good start.\n---\n\nBody.\n";
    let markdown = blocks(&["frontmatter.title.md"], "a.md", source)?;
    let mdx = blocks(&["frontmatter.title.md"], "a.mdx", source)?;
    let named = blocks(&["frontmatter.title.markdown"], "a.md", source)?;
    assert_eq!(named.len(), 1, "a name of Markdown ends the part too");
    assert_eq!(markdown.len(), 1, "the title of the Markdown file");
    assert_eq!(mdx.len(), 0, "no block of an MDX file");
    Ok(())
}

#[test]
fn a_front_matter_key_that_is_an_extension_stays_a_key() -> TestResult {
    let source = "---\ntoc: A very good start.\n---\n\nBody.\n";
    let found = blocks(&["frontmatter.toc"], "a.md", source)?;
    assert_eq!(found.len(), 1, "the value of the key toc");
    Ok(())
}
