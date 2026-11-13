//! Tests for the comments the HTML tokenizer finds.

use super::{comments, flushes_at};

#[test]
fn a_comment_is_found_where_it_stands() {
    let html = "<p>a <!-- one --> b <!--two--></p>";
    let found: Vec<&str> = comments(html)
        .into_iter()
        .filter_map(|range| html.get(range))
        .collect();
    assert_eq!(found, ["<!-- one -->", "<!--two-->"]);
}

#[test]
fn a_declaration_and_a_processing_instruction_are_no_comment() {
    let html = "<!DOCTYPE html><?php echo 1; ?><!-- yes -->";
    let found: Vec<&str> = comments(html)
        .into_iter()
        .filter_map(|range| html.get(range))
        .collect();
    assert_eq!(found, ["<!-- yes -->"]);
}

#[test]
fn a_comment_may_end_with_a_bang() {
    let html = "<!-- a --!>x";
    let found: Vec<&str> = comments(html)
        .into_iter()
        .filter_map(|range| html.get(range))
        .collect();
    assert_eq!(found, ["<!-- a --!>"]);
}

#[test]
fn a_comment_inside_a_script_is_text() {
    let html = "<script>x <!-- vale off --> y</script><!-- z -->";
    let found: Vec<&str> = comments(html)
        .into_iter()
        .filter_map(|range| html.get(range))
        .collect();
    assert_eq!(found, ["<!-- z -->"]);
}

#[test]
fn a_block_tag_inside_an_inline_tag_flushes_nothing() {
    let html = "<span><div>a</div></span><p>b</p>";
    assert_eq!(flushes_at(html), Some(25));
}

#[test]
fn a_comment_after_a_promoted_block_tag_has_no_flush_before_it() {
    let html = "<span><div>a</div><!-- vale off --></span>";
    assert_eq!(flushes_at(html), None);
}

#[test]
fn a_skipped_element_flushes_at_its_end_tag() {
    assert_eq!(flushes_at("<pre>a</pre><p>b</p>"), Some(6));
    assert_eq!(flushes_at("<div class=\"code\">a</div>"), Some(19));
}

#[test]
fn a_skipped_tag_inside_an_inline_tag_is_promoted() {
    assert_eq!(flushes_at("<span><pre>a</pre></span><p>b</p>"), Some(25));
}
