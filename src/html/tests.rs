//! Tests for the comments the HTML tokenizer finds.

use super::comments;

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
