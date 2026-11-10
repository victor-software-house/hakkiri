//! Snapshot and span tests for the document tree.
//!
//! Each snapshot lists the nodes of one fixture, one per line: the scope, the
//! byte span, and the text the span covers. The span tests assert the byte
//! spans that the document-tree spec states.

#![expect(
    clippy::panic_in_result_fn,
    reason = "A test loads its fixture with `?` and fails through an assertion."
)]

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use hakkiri::{Error as ParseError, Language, Node, Scope, Tree, parse};

type TestResult = Result<(), Box<dyn Error>>;

struct Fixture {
    source: String,
    tree: Tree,
}

/// Reads `tests/fixtures/<name>.fixture`.
///
/// The suffix keeps the Markdown and prose linters from reading the fixture,
/// and `name` keeps the extension that picks its language.
fn load(name: &str) -> Result<Fixture, Box<dyn Error>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(format!("{name}.fixture"));
    let source = fs::read_to_string(path)?;
    let language =
        Language::from_path(Path::new(name)).ok_or("a fixture with an unknown extension")?;
    let tree = parse(language, &source)?;
    Ok(Fixture { source, tree })
}

impl Fixture {
    /// Every node, parents before children, indented by depth.
    fn render(&self) -> String {
        let mut lines = Vec::new();
        for root in &self.tree.roots {
            self.render_node(root, 0, &mut lines);
        }
        lines.join("\n")
    }

    fn render_node(&self, node: &Node, depth: usize, lines: &mut Vec<String>) {
        let segments: Vec<String> = node
            .segments
            .iter()
            .map(|segment| format!("{}..{}", segment.start, segment.end))
            .collect();
        lines.push(format!(
            "{}{} {}..{} [{}] {:?}",
            "  ".repeat(depth),
            node.scope,
            node.span.start,
            node.span.end,
            segments.join(", "),
            self.text(node),
        ));
        for child in &node.children {
            self.render_node(child, depth + 1, lines);
        }
    }

    /// The text of `node`: its segments joined by a line feed.
    fn text(&self, node: &Node) -> String {
        node.text(&self.source)
            .unwrap_or_else(|| "<segment outside the file>".to_owned())
    }

    /// Every node of `scope`, parents before children.
    fn nodes(&self, scope: Scope) -> Vec<&Node> {
        fn collect<'a>(node: &'a Node, scope: Scope, found: &mut Vec<&'a Node>) {
            if node.scope == scope {
                found.push(node);
            }
            for child in &node.children {
                collect(child, scope, found);
            }
        }
        let mut found = Vec::new();
        for root in &self.tree.roots {
            collect(root, scope, &mut found);
        }
        found
    }

    /// The text of every node below `raw` and its comment node: the Markdown
    /// nodes.
    fn prose(&self) -> Vec<String> {
        fn collect(fixture: &Fixture, node: &Node, found: &mut Vec<String>) {
            let wrapper = matches!(
                node.scope,
                Scope::Raw
                    | Scope::CommentLine
                    | Scope::CommentBlock
                    | Scope::CommentDocLine
                    | Scope::CommentDocBlock
            );
            if !wrapper {
                found.push(fixture.text(node));
            }
            for child in &node.children {
                collect(fixture, child, found);
            }
        }
        let mut found = Vec::new();
        for root in &self.tree.roots {
            collect(self, root, &mut found);
        }
        found
    }

    fn texts(&self, scope: Scope) -> Vec<String> {
        self.nodes(scope)
            .into_iter()
            .map(|node| self.text(node))
            .collect()
    }
}

macro_rules! snapshots {
    ($($test:ident: $fixture:literal,)*) => {
        $(
            #[test]
            fn $test() -> TestResult {
                insta::assert_snapshot!(load($fixture)?.render());
                Ok(())
            }
        )*
    };
}

snapshots! {
    snapshot_heading_paragraph: "heading-paragraph.md",
    snapshot_headings: "headings.md",
    snapshot_heading_syntax: "heading-syntax.md",
    snapshot_nested_list: "nested-list.md",
    snapshot_table: "table.md",
    snapshot_code_and_markup: "code-and-markup.md",
    snapshot_inline: "inline.md",
    snapshot_mdx: "mdx.mdx",
    snapshot_block_comment: "block-comment.rs",
    snapshot_block_comments: "block-comments.rs",
    snapshot_string_url: "string-url.rs",
    snapshot_doc_lines: "doc-lines.rs",
    snapshot_doc_link_indented: "doc-link-indented.rs",
    snapshot_bare_url: "bare-url.rs",
    snapshot_jsdoc: "jsdoc.ts",
    snapshot_jsdoc_tags: "jsdoc-tags.js",
    snapshot_jsdoc_braces: "jsdoc-braces.ts",
    snapshot_line_comments: "line-comments.rs",
    snapshot_cuts_md: "cuts.md",
    snapshot_cuts_rs: "cuts.rs",
    snapshot_esm: "esm.mdx",
    snapshot_esm_literals: "esm-literals.mdx",
    snapshot_table_no_pipes: "table-no-pipes.md",
    snapshot_jsdoc_more: "jsdoc-more.ts",
    snapshot_banner: "banner.rs",
    snapshot_crlf_md: "crlf.md",
    snapshot_quote_paragraph: "quote-paragraph.md",
    snapshot_crlf_rs: "crlf.rs",
    snapshot_tabs_rs: "tabs.rs",
    snapshot_tabs_md: "tabs.md",
    snapshot_unicode: "unicode.rs",
    snapshot_no_stars: "no-stars.ts",
    snapshot_jsx_text: "jsx-text.tsx",
    snapshot_unterminated: "unterminated.js",
    snapshot_doc_indented_lines: "doc-indented-lines.rs",
    snapshot_tsx: "view.tsx",
    snapshot_mjs: "module.mjs",
}

#[test]
fn a_paragraph_in_a_markdown_file() -> TestResult {
    let fixture = load("heading-paragraph.md")?;
    let heading = fixture.nodes(Scope::Heading1);
    let paragraph = fixture.nodes(Scope::Paragraph);
    assert_eq!(heading.len(), 1);
    assert_eq!(heading[0].span, 2..7);
    assert_eq!(paragraph.len(), 1);
    assert_eq!(paragraph[0].span, 9..18);
    Ok(())
}

#[test]
fn a_nested_list() -> TestResult {
    let fixture = load("nested-list.md")?;
    let lists = fixture.nodes(Scope::List);
    let one = lists.first().ok_or("no list node")?;
    assert!(fixture.text(one).starts_with("One"));
    let two = one
        .children
        .iter()
        .find(|child| child.scope == Scope::List)
        .ok_or("no nested list node")?;
    assert!(fixture.text(two).starts_with("Two"));
    Ok(())
}

#[test]
fn a_table() -> TestResult {
    let fixture = load("table.md")?;
    let rows = fixture.nodes(Scope::TableRow);
    assert_eq!(rows.len(), 3);
    // A row reads its cells without the pipes, one space apart.
    assert_eq!(
        fixture.texts(Scope::TableRow),
        ["Name Note", "a b", "longer cell `code` and **strong**"]
    );
    let headers = fixture.texts(Scope::TableHeader);
    assert_eq!(headers, ["Name", "Note"]);
    let cells = fixture.texts(Scope::TableCell);
    assert_eq!(cells, ["a", "b", "longer cell", "`code` and **strong**"]);
    for row in &rows[..2] {
        assert!(row.children.iter().all(|cell| cell.scope != Scope::Raw));
    }
    assert_eq!(rows[0].children.len(), 2);
    assert!(
        rows[0]
            .children
            .iter()
            .all(|cell| cell.scope == Scope::TableHeader)
    );
    assert_eq!(rows[1].children.len(), 2);
    assert!(
        rows[1]
            .children
            .iter()
            .all(|cell| cell.scope == Scope::TableCell)
    );
    Ok(())
}

#[test]
fn a_url_in_a_fenced_block() -> TestResult {
    let fixture = load("code-and-markup.md")?;
    for url in [
        "https://example.com",
        "https://html.example",
        "https://definition.example",
        "A title",
    ] {
        assert!(
            !fixture.prose().iter().any(|text| text.contains(url)),
            "{url} reached a prose node"
        );
    }
    let raw = fixture.nodes(Scope::Raw);
    assert_eq!(raw.len(), 1);
    assert_eq!(raw[0].span, 0..fixture.source.len());
    assert!(fixture.text(raw[0]).contains("https://example.com"));
    Ok(())
}

#[test]
fn spans_are_bytes_not_characters() -> TestResult {
    let fixture = load("inline.md")?;
    assert_eq!(
        fixture.texts(Scope::Link),
        [
            "[link](https://a.example)",
            "[a reference][ref]",
            "https://bare.example",
            "[naïve link](https://b.example)",
        ]
    );
    assert_eq!(fixture.texts(Scope::Code), ["`code`"]);
    assert_eq!(fixture.texts(Scope::Strong), ["**strong**"]);
    assert_eq!(fixture.texts(Scope::Emphasis), ["*emphasis*"]);
    assert_eq!(fixture.texts(Scope::Alt), ["alt *text*", "logo"]);
    Ok(())
}

#[test]
fn a_jsx_element_with_text() -> TestResult {
    let fixture = load("mdx.mdx")?;
    let paragraphs = fixture.texts(Scope::Paragraph);
    assert!(paragraphs.iter().any(|text| text == "Read this."));
    for text in fixture.prose() {
        for markup in [
            "import",
            "export",
            "title=\"t\"",
            "<Note",
            "</Note>",
            "{expression}",
            "<b>",
            "<Callout>",
        ] {
            assert!(
                !text.contains(markup),
                "{markup} reached the prose node {text:?}"
            );
        }
    }
    assert!(paragraphs.iter().any(|text| text == "A paragraph inside."));
    assert_eq!(fixture.texts(Scope::List), ["A list item"]);
    // One paragraph, with the expression and the tags as gaps in its segments.
    assert_eq!(
        paragraphs
            .iter()
            .filter(|text| text.starts_with("Text with"))
            .collect::<Vec<_>>(),
        ["Text with  inside and bold too."]
    );
    Ok(())
}

#[test]
fn a_rust_block_comment() -> TestResult {
    let fixture = load("block-comment.rs")?;
    let comments = fixture.nodes(Scope::CommentBlock);
    assert_eq!(comments.len(), 1);
    let paragraph = comments[0].children.first().ok_or("no paragraph")?;
    assert_eq!(paragraph.scope, Scope::Paragraph);
    assert_eq!(paragraph.span, 3..10);
    assert_eq!(fixture.text(paragraph), "A note.");
    Ok(())
}

#[test]
fn a_string_that_holds_a_url() -> TestResult {
    let fixture = load("string-url.rs")?;
    assert_eq!(fixture.texts(Scope::CommentLine), ["// see the docs"]);
    assert_eq!(fixture.texts(Scope::Paragraph), ["see the docs"]);
    assert_eq!(fixture.tree.roots.len(), 1);
    assert_eq!(fixture.texts(Scope::Raw), ["// see the docs"]);
    Ok(())
}

#[test]
fn a_doc_comment_over_three_lines() -> TestResult {
    let fixture = load("doc-lines.rs")?;
    let comments = fixture.nodes(Scope::CommentDocLine);
    // The `///` group, and the `//!` line.
    assert_eq!(comments.len(), 2);
    assert!(
        comments[0]
            .children
            .iter()
            .all(|child| child.scope == Scope::Paragraph)
    );
    let paragraphs: Vec<_> = comments[0]
        .children
        .iter()
        .map(|child| fixture.text(child))
        .collect();
    assert_eq!(paragraphs, ["First line", "Second paragraph."]);
    let plain = fixture.nodes(Scope::CommentLine);
    assert_eq!(plain.len(), 2);
    assert_eq!(
        fixture.text(plain[0]),
        "// A plain comment\n// over two lines."
    );
    Ok(())
}

#[test]
fn a_link_in_an_indented_doc_comment() -> TestResult {
    let fixture = load("doc-link-indented.rs")?;
    let links = fixture.nodes(Scope::Link);
    assert_eq!(links.len(), 1);
    let doc_line = "    /// See [the guide](https://g.example).";
    let line_start = fixture
        .source
        .find(doc_line)
        .ok_or("fixture line not found")?;
    let target = "[the guide](https://g.example)";
    let start = line_start + doc_line.find(target).ok_or("link not in line")?;
    assert_eq!(links[0].span, start..start + target.len());
    Ok(())
}

#[test]
fn a_bare_url_in_a_plain_comment_is_a_link() -> TestResult {
    let fixture = load("bare-url.rs")?;
    assert_eq!(fixture.texts(Scope::Link), ["https://bare.example"]);
    assert_eq!(fixture.texts(Scope::Code), ["`https://span.example`"]);
    Ok(())
}

#[test]
fn a_parameter_tag() -> TestResult {
    let fixture = load("jsdoc.ts")?;
    let paragraphs = fixture.texts(Scope::Paragraph);
    assert_eq!(
        paragraphs,
        [
            "Sum two numbers.",
            "The first number.",
            "The second number.",
            "The sum, as a [number](https://n.example).",
            "add them",
        ]
    );
    for text in fixture.prose() {
        assert!(!text.contains("@param"), "{text:?}");
    }
    Ok(())
}

#[test]
fn braces_in_a_parameter_tag() -> TestResult {
    let fixture = load("jsdoc-braces.ts")?;
    assert_eq!(
        fixture.texts(Scope::Paragraph),
        [
            "A stray brace in the name.",
            "See {@link Other} for the description.",
            "A nested type."
        ]
    );
    Ok(())
}

#[test]
fn a_paragraph_over_two_line_comments() -> TestResult {
    let fixture = load("line-comments.rs")?;
    let paragraphs = fixture.nodes(Scope::Paragraph);
    assert_eq!(paragraphs.len(), 1);
    // `// Read the guide` starts at byte 0 and `// before you start.` at 18.
    assert_eq!(paragraphs[0].segments, [3..17, 21..38]);
    assert_eq!(paragraphs[0].span, 3..38);
    assert_eq!(
        fixture.text(paragraphs[0]),
        "Read the guide\nbefore you start."
    );
    Ok(())
}

#[test]
fn a_paragraph_over_indented_doc_lines() -> TestResult {
    let fixture = load("doc-indented-lines.rs")?;
    let paragraphs = fixture.nodes(Scope::Paragraph);
    assert_eq!(paragraphs.len(), 1);
    assert_eq!(paragraphs[0].segments.len(), 2);
    assert_eq!(
        fixture.text(paragraphs[0]),
        "First line of the summary,\nsecond line."
    );
    for segment in &paragraphs[0].segments {
        let line_start = fixture.source[..segment.start]
            .rfind('\n')
            .map_or(0, |i| i + 1);
        assert_eq!(&fixture.source[line_start..segment.start], "    /// ");
    }
    Ok(())
}

#[test]
fn code_html_and_definitions_are_gaps() -> TestResult {
    let fixture = load("cuts.md")?;
    for url in [
        "https://example.com",
        "https://def.example",
        "https://h.example",
        "https://x.example",
    ] {
        assert!(
            !fixture.prose().iter().any(|text| text.contains(url)),
            "{url} reached a prose node"
        );
    }
    assert_eq!(fixture.texts(Scope::Paragraph), ["An item", "Click here."]);
    assert_eq!(fixture.texts(Scope::List), ["An item"]);
    Ok(())
}

#[test]
fn html_in_a_doc_comment_is_a_gap() -> TestResult {
    let fixture = load("cuts.rs")?;
    assert_eq!(
        fixture.texts(Scope::Paragraph),
        ["Use Vec here, and `Vec<u8>` there."]
    );
    Ok(())
}

#[test]
fn an_export_block_with_a_blank_line() -> TestResult {
    let fixture = load("esm.mdx")?;
    assert_eq!(fixture.texts(Scope::Paragraph), ["Text."]);
    assert_eq!(fixture.texts(Scope::Heading1), ["Heading"]);
    assert!(
        !fixture
            .prose()
            .iter()
            .any(|text| text.contains("esm.example"))
    );
    Ok(())
}

#[test]
fn table_rows_without_outer_pipes_and_in_a_quote() -> TestResult {
    let fixture = load("table-no-pipes.md")?;
    assert_eq!(
        fixture.texts(Scope::TableRow),
        ["Name Note", "a b", "Q R", "c d"]
    );
    assert_eq!(
        fixture.texts(Scope::TableHeader),
        ["Name", "Note", "Q", "R"]
    );
    assert_eq!(fixture.texts(Scope::TableCell), ["a", "b", "c", "d"]);
    Ok(())
}

#[test]
fn jsdoc_types_names_and_examples_are_not_prose() -> TestResult {
    let fixture = load("jsdoc-more.ts")?;
    assert_eq!(
        fixture.texts(Scope::Paragraph),
        [
            "Summary line.",
            "The count.",
            "The name.",
            "The type.",
            "Called when finished.",
            "{@link Other} for more.\nText with a fence:",
        ]
    );
    let prose = fixture.prose().join("\n");
    for code in ["const x", "@Component", "class A", "{number}", "{string}"] {
        assert!(!prose.contains(code), "{code} reached the prose");
    }
    Ok(())
}

#[test]
fn a_banner_leaves_no_punctuation() -> TestResult {
    let fixture = load("banner.rs")?;
    assert_eq!(
        fixture.texts(Scope::Paragraph),
        ["Banner\nx", "Banner block", "Text."]
    );
    Ok(())
}

#[test]
fn line_endings_and_tabs_stay_out_of_the_text() -> TestResult {
    for name in ["crlf.rs", "tabs.rs", "tabs.md"] {
        let fixture = load(name)?;
        let paragraphs = fixture.texts(Scope::Paragraph);
        assert!(!paragraphs.is_empty(), "{name} gave no paragraph");
        for text in &paragraphs {
            assert!(!text.contains('\r'), "{name}: {text:?}");
            assert!(!text.contains("///"), "{name}: {text:?}");
        }
    }
    assert_eq!(
        load("crlf.md")?.texts(Scope::Paragraph),
        ["First line\nsecond line.", "item"]
    );
    assert_eq!(
        load("crlf.rs")?.texts(Scope::Paragraph),
        ["First line\nsecond line.", "Block\ntext."]
    );
    assert_eq!(
        load("tabs.rs")?.texts(Scope::Paragraph),
        [
            "Tabbed line one\ntabbed line two",
            "Starred with tabs.\nSecond."
        ]
    );
    Ok(())
}

#[test]
fn non_ascii_text_in_a_comment() -> TestResult {
    let fixture = load("unicode.rs")?;
    assert_eq!(
        fixture.texts(Scope::Link),
        ["[lien](https://é.example)", "[lien](https://é.example)"]
    );
    assert_eq!(
        fixture.texts(Scope::Paragraph),
        [
            "Café naïve [lien](https://é.example) — ünï",
            "Ünï [lien](https://é.example)",
            "Ünïcode block"
        ]
    );
    Ok(())
}

#[test]
fn a_doc_block_without_stars() -> TestResult {
    let fixture = load("no-stars.ts")?;
    assert_eq!(
        fixture.texts(Scope::Paragraph),
        ["Plain text line one.\nLine two."]
    );
    Ok(())
}

#[test]
fn slashes_in_jsx_text_are_not_a_comment() -> TestResult {
    let fixture = load("jsx-text.tsx")?;
    assert_eq!(fixture.texts(Scope::CommentLine), ["// real comment"]);
    assert_eq!(fixture.texts(Scope::Raw), ["// real comment"]);
    Ok(())
}

/// The number of levels in the deepest branch of `tree`, counted without
/// recursion.
fn depth(tree: &Tree) -> usize {
    let mut deepest = 0;
    let mut pending: Vec<(&Node, usize)> = tree.roots.iter().map(|root| (root, 1)).collect();
    while let Some((node, level)) = pending.pop() {
        deepest = deepest.max(level);
        pending.extend(node.children.iter().map(|child| (child, level + 1)));
    }
    deepest
}

/// The tree builder keeps its own stack.
/// A document nests as deep as markdown-rs reads it: 1,000 levels of block
/// quotes or lists.
#[test]
fn deeply_nested_quotes_and_lists() -> TestResult {
    let quotes = format!("{} deep\n", ">".repeat(1000));
    let tree = parse(Language::Markdown, &quotes)?;
    assert_eq!(tree.roots.len(), 1);
    assert!(depth(&tree) > 1000);
    let mut lists = String::new();
    for level in 0..1000 {
        lists.push_str(&"  ".repeat(level));
        lists.push_str("- item\n");
    }
    let tree = parse(Language::Markdown, &lists)?;
    assert_eq!(tree.roots.len(), 1);
    assert!(depth(&tree) > 1000);
    Ok(())
}

#[test]
fn literals_in_an_export_block() -> TestResult {
    let fixture = load("esm-literals.mdx")?;
    assert_eq!(
        fixture.texts(Scope::Paragraph),
        [
            "Prose after a template.",
            "Prose after a template with a blank line.",
            "Prose after a regular expression.",
            "Prose after a regular expression with a quote.",
            "Prose after an apostrophe.",
            "Prose after a keyword and a regular expression.",
            "Prose after typeof and a quote in a regular expression.",
            "Prose after a character class.",
            "Prose after an escaped slash.",
            "Prose after a backtick in JSX text.",
            "Prose after a closing tag in parentheses.",
            "Prose after a template with a blank line in JSX.",
            "Prose after an apostrophe in a function.",
            "Prose after an object split by a blank line.",
            "Prose after an argument list split by a blank line.",
            "Prose after a JSX element split by a blank line.",
        ]
    );
    Ok(())
}

#[test]
fn an_unclosed_export_block_fails_the_file() {
    let result = parse(Language::Mdx, "export const a = {\n\nProse.\n");
    assert!(matches!(result, Err(ParseError::Mdx(_))), "{result:?}");
}

#[test]
fn a_missing_bracket_at_the_end_reads_on() -> TestResult {
    let fixture = {
        let source = "export const a = f(1\n\n)\n\nProse.\n".to_owned();
        let tree = parse(Language::Mdx, &source)?;
        Fixture { source, tree }
    };
    assert_eq!(fixture.texts(Scope::Paragraph), ["Prose."]);
    Ok(())
}

#[test]
fn a_syntax_error_in_an_export_block_fails_the_file() -> TestResult {
    let result = parse(
        Language::Mdx,
        "Intro.\n\nexport const = 1\n\nProse after.\n",
    );
    let Err(ParseError::Mdx(message)) = result else {
        return Err(format!("expected an MDX error, got {result:?}").into());
    };
    let text = message.to_string();
    assert!(text.contains("3:"), "{text}");
    assert!(text.contains("Unexpected syntax"), "{text}");
    Ok(())
}

#[test]
fn a_quote_with_only_a_definition_has_no_node() -> TestResult {
    let tree = parse(Language::Markdown, "> [a]: https://a.example\n")?;
    assert!(tree.roots.iter().all(|root| root.children.is_empty()));
    Ok(())
}

#[test]
fn a_paragraph_in_a_quote_or_a_list_item_leaves_out_the_markers() -> TestResult {
    let fixture = load("quote-paragraph.md")?;
    let paragraphs = fixture.nodes(Scope::Paragraph);
    let texts: Vec<String> = paragraphs
        .iter()
        .map(|paragraph| fixture.text(paragraph))
        .collect();
    assert_eq!(
        texts,
        [
            "First line\nsecond line.",
            "Nested one\nnested two.",
            "Item first\nitem second",
            "Child first\nchild second",
            "Quoted item\ncontinues here.",
            "Lazy start\nlazy line.",
        ]
    );
    for paragraph in paragraphs {
        assert_eq!(paragraph.segments.len(), 2);
        for segment in &paragraph.segments {
            let line = fixture.source.get(segment.clone()).unwrap_or_default();
            assert_eq!(line, line.trim(), "{line:?} keeps a marker or indentation");
            assert!(!line.starts_with('>'), "{line:?} keeps a marker");
        }
    }
    Ok(())
}

#[test]
fn a_heading_leaves_out_its_hashes_and_underline() -> TestResult {
    let fixture = load("heading-syntax.md")?;
    assert_eq!(
        fixture.texts(Scope::Heading1),
        ["Closed", "Multi line\nsetext", "Quoted heading"]
    );
    assert_eq!(
        fixture.texts(Scope::Heading2),
        ["Closed with more", "Setext two"]
    );
    assert_eq!(fixture.texts(Scope::Heading3), ["Hash# stays"]);
    assert_eq!(fixture.texts(Scope::Heading4), Vec::<String>::new());
    Ok(())
}

#[test]
fn a_quote_or_a_list_item_reads_as_its_content() -> TestResult {
    let fixture = load("quote-paragraph.md")?;
    let quotes = fixture.texts(Scope::Blockquote);
    assert_eq!(quotes.len(), 4);
    let lists = fixture.texts(Scope::List);
    assert_eq!(lists.len(), 3);
    for text in quotes.iter().chain(&lists) {
        for line in text.lines() {
            assert!(!line.starts_with(['>', '-', ' ']), "{line:?} keeps syntax");
        }
    }
    for container in fixture
        .nodes(Scope::Blockquote)
        .into_iter()
        .chain(fixture.nodes(Scope::List))
    {
        let first = container.segments.first().ok_or("no segment")?;
        let last = container.segments.last().ok_or("no segment")?;
        assert_eq!(container.span, first.start..last.end);
    }
    Ok(())
}
