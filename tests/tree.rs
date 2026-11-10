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

use hakkiri::{Language, Node, Scope, Tree, parse};

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
        lines.push(format!(
            "{}{} {}..{} {:?}",
            "  ".repeat(depth),
            node.scope,
            node.span.start,
            node.span.end,
            self.text(node),
        ));
        for child in &node.children {
            self.render_node(child, depth + 1, lines);
        }
    }

    fn text(&self, node: &Node) -> &str {
        self.source
            .get(node.span.clone())
            .unwrap_or("<span outside the file>")
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
    fn prose(&self) -> Vec<&str> {
        fn collect<'a>(fixture: &'a Fixture, node: &'a Node, found: &mut Vec<&'a str>) {
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

    fn texts(&self, scope: Scope) -> Vec<&str> {
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
    snapshot_tsx: "view.tsx",
    snapshot_mjs: "module.mjs",
}

#[test]
fn a_paragraph_in_a_markdown_file() -> TestResult {
    let fixture = load("heading-paragraph.md")?;
    let heading = fixture.nodes(Scope::Heading1);
    let paragraph = fixture.nodes(Scope::Paragraph);
    assert_eq!(heading.len(), 1);
    assert_eq!(heading[0].span, 0..7);
    assert_eq!(paragraph.len(), 1);
    assert_eq!(paragraph[0].span, 9..18);
    Ok(())
}

#[test]
fn a_nested_list() -> TestResult {
    let fixture = load("nested-list.md")?;
    let lists = fixture.nodes(Scope::List);
    let one = lists.first().ok_or("no list node")?;
    assert!(fixture.text(one).starts_with("- One"));
    let two = one
        .children
        .iter()
        .find(|child| child.scope == Scope::List)
        .ok_or("no nested list node")?;
    assert!(fixture.text(two).starts_with("- Two"));
    Ok(())
}

#[test]
fn a_table() -> TestResult {
    let fixture = load("table.md")?;
    let rows = fixture.nodes(Scope::TableRow);
    assert_eq!(rows.len(), 3);
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
    assert_eq!(fixture.texts(Scope::Alt), ["alt *text*"]);
    Ok(())
}

#[test]
fn a_jsx_element_with_text() -> TestResult {
    let fixture = load("mdx.mdx")?;
    let paragraphs = fixture.texts(Scope::Paragraph);
    assert!(paragraphs.contains(&"Read this."));
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
    assert!(paragraphs.contains(&"A paragraph inside."));
    assert_eq!(fixture.texts(Scope::List), ["- A list item"]);
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
    let paragraphs: Vec<_> = comments[0]
        .children
        .iter()
        .map(|child| (child.scope, fixture.text(child)))
        .collect();
    assert_eq!(
        paragraphs,
        [
            (Scope::Paragraph, "First line"),
            (Scope::Paragraph, "Second paragraph.")
        ]
    );
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
