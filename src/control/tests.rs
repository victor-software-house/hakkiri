//! Tests for the comments that turn rules off and on: the forms Vale reads and
//! the ones hakkiri adds, and where each one reaches.

use super::{Controls, Setting, cut, outside, setting};
use crate::rule::Level;
use crate::tree::{Comment, Node, Scope};

fn key(name: &str, disabled: bool) -> Setting {
    Setting::Key {
        key: name.to_owned(),
        disabled,
    }
}

#[test]
fn vale_turns_every_rule_off_and_on() {
    assert_eq!(setting("vale off"), Some(Setting::All { disabled: true }));
    assert_eq!(setting("vale on"), Some(Setting::All { disabled: false }));
    assert_eq!(setting("Vale Off"), None);
    assert_eq!(setting("note: vale off"), None);
}

#[test]
fn vale_turns_a_rule_or_a_style_off_and_on() {
    assert_eq!(
        setting("vale Team.Words = NO"),
        Some(key("Team.Words", true))
    );
    assert_eq!(
        setting("vale Team.Words = YES"),
        Some(key("Team.Words", false))
    );
    assert_eq!(
        setting("vale Team.Really = off"),
        Some(key("Team.Really", true))
    );
    assert_eq!(
        setting("vale Team.Really = on"),
        Some(key("Team.Really", false))
    );
    assert_eq!(setting("vale Team = NO"), Some(key("Team", true)));
    assert_eq!(
        setting("vale team.words = NO"),
        Some(key("team.words", true))
    );
}

#[test]
fn vale_reads_a_key_only_with_spaces_around_the_equals_sign() {
    assert_eq!(setting("vale Team.Words=NO"), None);
    assert_eq!(setting("vale Team.Words = MAYBE"), None);
    assert_eq!(setting("vale Team. = NO"), None);
}

#[test]
fn vale_turns_the_matches_of_a_rule_off_and_on() {
    assert_eq!(
        setting(r#"vale Team.Words["very"] = NO"#),
        Some(Setting::Matches {
            rule: "Team.Words".to_owned(),
            matches: vec!["very".to_owned()],
            disabled: true,
        })
    );
    assert_eq!(
        setting(r#"vale Team.Words["really", "very"] = YES"#),
        Some(Setting::Matches {
            rule: "Team.Words".to_owned(),
            matches: vec!["really".to_owned(), "very".to_owned()],
            disabled: false,
        })
    );
    assert_eq!(setting(r"vale Team.Words[very] = NO"), None);
}

#[test]
fn vale_limits_the_styles_of_a_file() {
    assert_eq!(
        setting("vale styles = Team, Other"),
        Some(Setting::Styles(vec!["Team".to_owned(), "Other".to_owned()]))
    );
    assert_eq!(
        setting("vale style = Team"),
        Some(Setting::Styles(vec!["Team".to_owned()]))
    );
    assert_eq!(setting("vale styles ="), None);
}

#[test]
fn hakkiri_writes_the_words_of_the_config() {
    assert_eq!(
        setting("hakkiri off"),
        Some(Setting::All { disabled: true })
    );
    assert_eq!(
        setting("hakkiri on"),
        Some(Setting::All { disabled: false })
    );
    assert_eq!(
        setting("hakkiri Team.Words: off"),
        Some(key("Team.Words", true))
    );
    assert_eq!(
        setting("hakkiri Team.Words: on"),
        Some(key("Team.Words", false))
    );
    assert_eq!(setting("hakkiri Team: off"), Some(key("Team", true)));
    assert_eq!(
        setting("hakkiri Team.Words: suggestion"),
        Some(Setting::Level {
            rule: "Team.Words".to_owned(),
            level: Level::Suggestion
        })
    );
    assert_eq!(
        setting("hakkiri Team.Words: error"),
        Some(Setting::Level {
            rule: "Team.Words".to_owned(),
            level: Level::Error
        })
    );
}

#[test]
fn hakkiri_reads_no_level_for_a_style_and_no_other_word() {
    assert_eq!(setting("hakkiri Team: warning"), None);
    assert_eq!(setting("hakkiri Team.Words: loud"), None);
    assert_eq!(setting("hakkiri Team.Words = NO"), None);
    assert_eq!(setting("hakkiri styles: Team"), None);
}

/// The comments of `source`, each `<!-- text -->` on its own.
fn comments(source: &str) -> Vec<Comment> {
    let mut found = Vec::new();
    let mut from = 0;
    while let Some(start) = source[from..].find("<!--") {
        let at = from + start;
        let end = at + source[at..].find("-->").expect("the comment closes");
        found.push(Comment {
            at,
            text: source[at + 4..end].trim().to_owned(),
        });
        from = end + 3;
    }
    found
}

const SOURCE: &str = "a\n<!-- vale Team.Words = NO -->\nb\n<!-- vale Team.Words = YES -->\nc\n";

#[test]
fn an_alert_between_the_two_comments_is_hidden() {
    let controls = Controls::new(&comments(SOURCE), SOURCE, &[]);
    assert!(!controls.hides("Team.Words", "very", (1, 1), 0));
    assert!(controls.hides("Team.Words", "very", (3, 1), 0));
    assert!(!controls.hides("Team.Words", "very", (4, 1), 0));
    assert!(!controls.hides("Team.Words", "very", (5, 1), 0));
    assert!(!controls.hides("Team.Really", "very", (3, 1), 0));
}

#[test]
fn a_comment_inside_an_element_turns_the_rule_off_for_the_whole_element() {
    let controls = Controls::new(&comments(SOURCE), SOURCE, &[]);
    let b = SOURCE.find("b\n").expect("b is there");
    // An element that closes before the first comment, between the two, and
    // after the second.
    assert!(!controls.gated("Team.Words", 1));
    assert!(controls.gated("Team.Words", b));
    assert!(!controls.gated("Team.Words", SOURCE.len()));
}

#[test]
fn a_style_and_every_rule_gate_their_rules() {
    let source = "<!-- vale Team = NO -->\nx\n<!-- vale Team = YES -->\n<!-- vale off -->\ny\n";
    let controls = Controls::new(&comments(source), source, &[]);
    let x = source.find('x').expect("x is there");
    let y = source.find('y').expect("y is there");
    assert!(controls.gated("Team.Words", x));
    assert!(!controls.gated("Other.Good", x));
    assert!(controls.gated("Other.Good", y));
}

#[test]
fn the_styles_comment_turns_off_the_styles_it_leaves_out() {
    let source = "<!-- vale styles = Other -->\nx\n";
    let styles = vec!["Team".to_owned(), "Other".to_owned()];
    let controls = Controls::new(&comments(source), source, &styles);
    let x = source.find('x').expect("x is there");
    assert!(controls.gated("Team.Words", x));
    assert!(!controls.gated("Other.Good", x));
}

#[test]
fn the_matches_comment_gates_one_match() {
    let source = "<!-- vale Team.Words[\"very\"] = NO -->\nx\n";
    let controls = Controls::new(&comments(source), source, &[]);
    let x = source.find('x').expect("x is there");
    assert!(controls.hides("Team.Words", "very", (2, 1), x));
    assert!(!controls.hides("Team.Words", "really", (2, 1), x));
}

#[test]
fn a_level_holds_until_a_later_comment_names_the_rule() {
    let source = "a\n<!-- hakkiri Team.Words: error -->\nb\n<!-- hakkiri Team.Words: on -->\nc\n";
    let controls = Controls::new(&comments(source), source, &[]);
    assert_eq!(controls.level("Team.Words", (1, 1)), None);
    assert_eq!(controls.level("Team.Words", (3, 1)), Some(Level::Error));
    assert_eq!(controls.level("Team.Words", (5, 1)), None);
    assert_eq!(controls.level("Team.Really", (3, 1)), None);
}

#[test]
fn outside_keeps_the_parts_no_range_holds() {
    let ranges = [10..20, 30..40];
    let kept = |span: std::ops::Range<usize>| -> Vec<(usize, usize)> {
        outside(&span, &ranges)
            .into_iter()
            .map(|part| (part.start, part.end))
            .collect()
    };
    assert_eq!(kept(0..50), [(0, 10), (20, 30), (40, 50)]);
    assert_eq!(kept(12..18), Vec::<(usize, usize)>::new());
    assert_eq!(kept(15..35), [(20, 30)]);
    assert_eq!(kept(0..5), [(0, 5)]);
    assert_eq!(kept(25..25), [(25, 25)]);
    assert_eq!(kept(15..15), Vec::<(usize, usize)>::new());
}

#[test]
fn cut_drops_the_text_between_off_and_on_and_the_nodes_it_empties() {
    let source = "a b\n<!-- vale off -->\nc\n<!-- vale on -->\nd e\n";
    let c = source.find('c').expect("c is there");
    let d = source.find('d').expect("d is there");
    let mut nodes = vec![
        Node::new(Scope::Paragraph, 0..3, Vec::new()),
        Node::new(Scope::Paragraph, c..c + 1, Vec::new()),
        Node::new(Scope::Paragraph, d..d + 3, Vec::new()),
    ];
    cut(&mut nodes, &comments(source), source.len());
    let spans: Vec<_> = nodes.iter().map(|node| node.span.clone()).collect();
    assert_eq!(spans, [0..3, d..d + 3]);
}
