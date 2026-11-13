//! Markdown and MDX text into nodes, through markdown-rs.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::mem;
use std::ops::Range;
use std::slice::Iter;
use std::sync::LazyLock;

use markdown::mdast::{Node as Mdast, Strong, Text};
use markdown::unist::Position;
use markdown::{Constructs, MdxSignal, ParseOptions};
use regex::Regex;
use tree_sitter::{InputEdit, Node as SyntaxNode, Parser, Point, Tree};

use crate::error::Error;
use crate::tree::{Comment, Node, Scope};
use crate::{frontmatter, html, html_text, math};

/// Which constructs the parser reads.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Flavor {
    /// A `.md` file: GFM and front matter.
    Markdown,
    /// An `.mdx` file: MDX, GFM and front matter.
    Mdx,
    /// The text of a comment: GFM, with no front matter.
    Comment,
}

impl Flavor {
    fn options(self) -> ParseOptions {
        match self {
            Self::Markdown => ParseOptions {
                constructs: Constructs {
                    frontmatter: true,
                    ..Constructs::gfm()
                },
                ..ParseOptions::default()
            },
            Self::Comment => ParseOptions::gfm(),
            Self::Mdx => ParseOptions {
                constructs: Constructs {
                    frontmatter: true,
                    gfm_autolink_literal: true,
                    gfm_footnote_definition: true,
                    gfm_label_start_footnote: true,
                    gfm_strikethrough: true,
                    gfm_table: true,
                    gfm_task_list_item: true,
                    ..Constructs::mdx()
                },
                mdx_esm_parse: Some(Box::new(esm)),
                ..ParseOptions::default()
            },
        }
    }
}

/// Tells markdown-rs whether an import or export block is complete.
///
/// markdown-rs reads such a block only when it has a parser for it.
/// It ends the block at the first blank line the parser accepts.
/// The TSX grammar of tree-sitter parses the block, as acorn does in the MDX
/// compiler.
///
/// - No syntax error: the block ends here.
/// - The first error reaches the end of the block, such as a missing bracket.
///   The code is incomplete, so markdown-rs reads on past the blank line. At
///   the end of the file markdown-rs turns that into an error.
/// - Any other error is a syntax error, reported at its position.
fn esm(block: &str) -> MdxSignal {
    let source = Box::new("hakkiri".to_owned());
    let rule = Box::new("esm".to_owned());
    let Some(tree) = parse_tsx(block) else {
        return MdxSignal::Error(
            "Cannot parse an import or export".to_owned(),
            0,
            source,
            rule,
        );
    };
    let root = tree.root_node();
    if !root.has_error() {
        return MdxSignal::Ok;
    }
    let error = first_error(root);
    if error.end_byte() >= block.trim_end().len() {
        MdxSignal::Eof(
            "Unexpected end of file in an import or export".to_owned(),
            source,
            rule,
        )
    } else {
        MdxSignal::Error(
            "Unexpected syntax in an import or export".to_owned(),
            error.start_byte(),
            source,
            rule,
        )
    }
}

thread_local! {
    /// The TSX parser of this thread, with the last block it parsed.
    /// `None` when the grammar did not load.
    static TSX: RefCell<Option<Tsx>> = RefCell::new(Tsx::new());
}

/// A TSX parser that reparses a block that grew.
///
/// markdown-rs hands over the same block again with more lines appended, so
/// the tree of the last parse is edited and reused.
/// A block that does not extend the last one is parsed from scratch.
struct Tsx {
    parser: Parser,
    /// The last block parsed and its tree.
    last: Option<(String, Tree)>,
}

impl Tsx {
    fn new() -> Option<Self> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_typescript::LANGUAGE_TSX.into())
            .ok()?;
        Some(Self { parser, last: None })
    }

    fn parse(&mut self, block: &str) -> Option<Tree> {
        let old = match self.last.take() {
            Some((text, mut tree)) if block.starts_with(&text) => {
                let end = point(&text);
                tree.edit(&InputEdit {
                    start_byte: text.len(),
                    old_end_byte: text.len(),
                    new_end_byte: block.len(),
                    start_position: end,
                    old_end_position: end,
                    new_end_position: point(block),
                });
                Some(tree)
            }
            _ => None,
        };
        let tree = self.parser.parse(block, old.as_ref())?;
        self.last = Some((block.to_owned(), tree.clone()));
        Some(tree)
    }
}

/// The row and column, in bytes, where `text` ends.
fn point(text: &str) -> Point {
    Point {
        row: text.matches('\n').count(),
        column: text.len() - text.rfind('\n').map_or(0, |line_feed| line_feed + 1),
    }
}

/// The syntax tree of `block` in the TSX grammar.
fn parse_tsx(block: &str) -> Option<Tree> {
    TSX.with_borrow_mut(|tsx| tsx.as_mut()?.parse(block))
}

/// The first `ERROR` or `MISSING` node of a tree that has one, in file order.
fn first_error(root: SyntaxNode<'_>) -> SyntaxNode<'_> {
    let mut cursor = root.walk();
    let mut node = root;
    loop {
        if node.is_error() || node.is_missing() {
            return node;
        }
        match node.children(&mut cursor).find(SyntaxNode::has_error) {
            Some(child) => node = child,
            None => return node,
        }
    }
}

/// What the parse of a Markdown or MDX text gives.
pub(crate) struct Built {
    /// The prose nodes.
    pub(crate) nodes: Vec<Node>,
    /// The fenced and indented code blocks.
    pub(crate) code_blocks: usize,
    /// The HTML comments of a Markdown file, in file order.
    pub(crate) comments: Vec<Comment>,
}

/// The tree of a whole Markdown or MDX file: one `raw` root over the prose
/// nodes, the number of its fenced and indented code blocks, and its HTML
/// comments.
pub(crate) fn document(
    source: &str,
    flavor: Flavor,
) -> Result<(Vec<Node>, usize, Vec<Comment>), Error> {
    let built = build(source, flavor)?;
    Ok((
        vec![Node::new(Scope::Raw, 0..source.len(), built.nodes)],
        built.code_blocks,
        built.comments,
    ))
}

/// The prose nodes of `source`, with spans in `source`.
pub(crate) fn blocks(source: &str, flavor: Flavor) -> Result<Vec<Node>, Error> {
    Ok(build(source, flavor)?.nodes)
}

/// The prose nodes of `source`, its number of code blocks, and the HTML
/// comments of a Markdown file.
pub(crate) fn build(source: &str, flavor: Flavor) -> Result<Built, Error> {
    let mut root = markdown::to_mdast(source, &flavor.options()).map_err(Error::Mdx)?;
    // Vale reads math as code, which markdown-rs reads by other rules, so the
    // text is parsed again with each math span marked as code.
    let masked;
    let mut spans = Vec::new();
    let source = if matches!(flavor, Flavor::Comment) {
        source
    } else {
        let math = math::find(&root, source, matches!(flavor, Flavor::Mdx));
        if math.is_empty() {
            source
        } else {
            masked = math::mask(source, &math);
            root = markdown::to_mdast(&masked, &flavor.options()).map_err(Error::Mdx)?;
            spans = math.inline;
            &masked
        }
    };
    let builder = Builder {
        source,
        code_blocks: Cell::new(0),
        addresses: addresses(&root),
        referenced: footnotes_referenced(&root),
    };
    let mut nodes = builder.visit(&root, Scope::TableCell).nodes;
    math::include_delimiters(&mut nodes, &spans);
    if !matches!(flavor, Flavor::Comment) {
        // JSON front matter is no construct of markdown-rs, so its values are
        // read from the start of the file, ahead of the body.
        let mut front = frontmatter::json_nodes(source);
        front.append(&mut nodes);
        nodes = front;
    }
    let comments = match flavor {
        Flavor::Markdown | Flavor::Mdx => comments_of(&root, source),
        Flavor::Comment => Vec::new(),
    };
    Ok(Built {
        nodes,
        code_blocks: builder.code_blocks.get(),
        comments,
    })
}

/// The comments below `root`, in file order: the HTML comments of an HTML
/// block and of inline HTML, and the `{/* … */}` comments of an MDX file, as a
/// flow expression or in a line of text. A code block and a code span hold
/// none.
fn comments_of(root: &Mdast, source: &str) -> Vec<Comment> {
    let mut found = Vec::new();
    let mut pending = vec![root];
    while let Some(node) = pending.pop() {
        if let Mdast::Html(_) = node
            && let Some(span) = span_of(node)
            && let Some(html) = source.get(span.clone())
        {
            for range in html::comments(html) {
                let written = html.get(range.clone()).unwrap_or_default();
                let inner = written.strip_prefix("<!--").unwrap_or(written);
                let inner = inner
                    .strip_suffix("-->")
                    .or_else(|| inner.strip_suffix("--!>"))
                    .unwrap_or(inner);
                found.push(Comment {
                    at: span.start + range.start,
                    text: inner.trim().to_owned(),
                });
            }
        }
        if let Mdast::MdxFlowExpression(_) | Mdast::MdxTextExpression(_) = node
            && let Some(span) = span_of(node)
            && let Some(written) = source.get(span.clone())
            && let Some(inner) = written
                .strip_prefix("{/*")
                .and_then(|rest| rest.strip_suffix("*/}"))
        {
            found.push(Comment {
                at: span.start,
                text: inner.trim().to_owned(),
            });
        }
        if let Some(children) = node.children() {
            pending.extend(children.iter().rev());
        }
    }
    found
}

/// Whether a block leaves the first place free: a thematic break, a link
/// definition, or an HTML block that resets nothing ([`crate::html`] explains
/// which).
fn leaves_first_place(node: &Mdast) -> bool {
    match node {
        Mdast::ThematicBreak(_) | Mdast::Definition(_) => true,
        Mdast::Html(html) => !html::resets_scope(&html.value),
        Mdast::Root(_)
        | Mdast::Blockquote(_)
        | Mdast::FootnoteDefinition(_)
        | Mdast::MdxJsxFlowElement(_)
        | Mdast::List(_)
        | Mdast::MdxjsEsm(_)
        | Mdast::Toml(_)
        | Mdast::Yaml(_)
        | Mdast::Break(_)
        | Mdast::InlineCode(_)
        | Mdast::InlineMath(_)
        | Mdast::Delete(_)
        | Mdast::Emphasis(_)
        | Mdast::MdxTextExpression(_)
        | Mdast::FootnoteReference(_)
        | Mdast::Image(_)
        | Mdast::ImageReference(_)
        | Mdast::MdxJsxTextElement(_)
        | Mdast::Link(_)
        | Mdast::LinkReference(_)
        | Mdast::Strong(_)
        | Mdast::Text(_)
        | Mdast::Code(_)
        | Mdast::Math(_)
        | Mdast::MdxFlowExpression(_)
        | Mdast::Heading(_)
        | Mdast::Table(_)
        | Mdast::TableRow(_)
        | Mdast::TableCell(_)
        | Mdast::ListItem(_)
        | Mdast::Paragraph(_) => false,
    }
}

/// What a visit of one mdast node gives its parent.
#[derive(Debug, Default)]
struct Visited {
    /// The prose nodes, in file order.
    nodes: Vec<Node>,
    /// MDX syntax inside the node that is not prose: JSX tags, expressions and
    /// ESM.
    cuts: Vec<Range<usize>>,
    /// The inline HTML inside the node, each tag or comment one span. They are
    /// cuts too: the text around them is read as Vale reads it.
    tags: Vec<Range<usize>>,
}

impl Visited {
    fn cut(span: Option<Range<usize>>) -> Self {
        Self {
            nodes: Vec::new(),
            cuts: span.into_iter().collect(),
            tags: Vec::new(),
        }
    }

    fn tag(span: Option<Range<usize>>) -> Self {
        Self {
            tags: span.clone().into_iter().collect(),
            ..Self::cut(span)
        }
    }

    fn append(mut self, other: Self) -> Self {
        self.nodes.extend(other.nodes);
        self.cuts.extend(other.cuts);
        self.tags.extend(other.tags);
        self
    }
}

struct Builder<'a> {
    source: &'a str,
    /// The fenced and indented code blocks seen so far.
    code_blocks: Cell<usize>,
    /// The address each link definition gives its label.
    addresses: HashMap<String, String>,
    /// The identifiers of the footnotes that some reference names.
    referenced: HashSet<String>,
}

/// The address of each link definition below `root`, by label.
/// The first definition of a label holds, as in `CommonMark`.
fn addresses(root: &Mdast) -> HashMap<String, String> {
    let mut found = HashMap::new();
    let mut pending = vec![root];
    while let Some(node) = pending.pop() {
        if let Mdast::Definition(definition) = node {
            found
                .entry(definition.identifier.clone())
                .or_insert_with(|| definition.url.clone());
        }
        if let Some(children) = node.children() {
            pending.extend(children.iter().rev());
        }
    }
    found
}

/// The bytes that goldmark writes into an `href` as they are, from
/// `urlEscapeTable` in goldmark 1.8.6, the version Vale 66bbb85 pins.
const HREF_SAFE: &[u8] =
    b"!#$&'()*+,-./0123456789:;=?@ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz~";

/// The `href` that goldmark writes for the link destination `destination`,
/// as its `util.URLEscape` does: a byte outside `HREF_SAFE` becomes `%` and
/// two upper-case hex digits, a space becomes `%20`, and a `%` before two hex
/// digits stays.
fn goldmark_href(destination: &str) -> String {
    let bytes = destination.as_bytes();
    let mut href = String::with_capacity(destination.len());
    let mut at = 0;
    while let Some(&byte) = bytes.get(at) {
        let escape = bytes
            .get(at + 1..at + 3)
            .filter(|hex| byte == b'%' && hex.iter().all(u8::is_ascii_hexdigit));
        if HREF_SAFE.contains(&byte) {
            href.push(char::from(byte));
            at += 1;
        } else if let Some(hex) = escape {
            href.push('%');
            href.extend(hex.iter().copied().map(char::from));
            at += 3;
        } else {
            href.push('%');
            for digit in [byte >> 4, byte & 0xf] {
                href.extend(
                    char::from_digit(u32::from(digit), 16).map(|hex| hex.to_ascii_uppercase()),
                );
            }
            at += 1;
        }
    }
    href
}

/// How a node of the mdast takes part in the tree.
enum Kind {
    /// A node of this scope over the nodes below it.
    Scoped(Scope),
    /// A table cell, of the scope the table gives its row.
    Cell,
    /// A JSX element: its tags are gaps in the text around it.
    Jsx,
    /// An inline HTML element, from its opening tag to its closing tag, of the
    /// scope Vale gives its tag. Its tags are gaps in its text, and the
    /// `title` nodes of its opening tag sit beside it, as they do beside a
    /// tag that opens no element.
    Raw {
        scope: Scope,
        open: Range<usize>,
        close: Range<usize>,
        titles: Vec<Node>,
    },
    /// No node of its own: the nodes below it go to its parent.
    Transparent,
}

/// The syntax an inline node holds that is not its text.
struct Markup {
    /// The span of the node's own text, markers aside.
    span: Range<usize>,
    /// What the node's own text leaves out.
    own: Vec<Range<usize>>,
    /// What the text around the node leaves out.
    up: Vec<Range<usize>>,
}

/// What the builder does with a node before it looks below it.
enum Step {
    /// The node has no prose below it. This is what it gives its parent.
    Done(Visited),
    /// The nodes below the node come first.
    Open(Kind),
}

/// A node whose children the builder is still visiting.
struct Frame<'n> {
    node: &'n Mdast,
    kind: Kind,
    /// The scope of a table cell found below the node.
    cell: Scope,
    children: Iter<'n, Mdast>,
    index: usize,
    inner: Visited,
    /// Whether the node is the first thing inside the node that gives it a
    /// place in the tree, and each node between the two is first in its own
    /// parent.
    first: bool,
    /// Whether a child that takes the first place has been visited.
    taken: bool,
    /// How many cells the header of the table around a row has, or none.
    columns: usize,
    /// The index among the children of `node` of the first child this frame
    /// visits: not the first for an inline HTML element.
    base: usize,
}

impl<'n> Frame<'n> {
    fn new(node: &'n Mdast, kind: Kind, cell: Scope, first: bool) -> Self {
        Self {
            node,
            kind,
            cell,
            children: node.children().map_or(&[][..], Vec::as_slice).iter(),
            index: 0,
            inner: Visited::default(),
            first,
            taken: false,
            columns: 0,
            base: 0,
        }
    }

    /// The next child, the scope of a table cell below it, and whether it is
    /// the first thing inside the node that holds it in the tree.
    ///
    /// The parser lists every child, a fenced block, a table, an empty list
    /// item and an HTML block included, and each takes the first place.
    /// A thematic break, a link definition, and an HTML block that ends no
    /// run of tags leave the place free: Vale takes a block's scope from the
    /// tags opened since the last reset.
    /// A node with no place of its own passes the place of its first child up.
    fn next_child(&mut self) -> Option<(&'n Mdast, Scope, bool)> {
        let child = self.children.next()?;
        let cell = match (self.node, self.index) {
            (Mdast::Table(_), 0) => Scope::TableHeader,
            (Mdast::Table(_), _) => Scope::TableCell,
            _ => self.cell,
        };
        let first = !self.taken
            && match self.kind {
                Kind::Scoped(_) | Kind::Cell | Kind::Raw { .. } => true,
                Kind::Jsx | Kind::Transparent => self.first,
            };
        self.taken |= !leaves_first_place(child);
        self.index += 1;
        Some((child, cell, first))
    }
}

impl Builder<'_> {
    /// Visits `root` and everything below it.
    ///
    /// The visit keeps its own stack of frames, so the depth of the document
    /// costs no call stack.
    fn visit(&self, root: &Mdast, cell: Scope) -> Visited {
        let mut stack = Vec::new();
        match self.step(root) {
            Step::Done(visited) => return visited,
            Step::Open(kind) => stack.push(Frame::new(root, kind, cell, true)),
        }
        loop {
            let Some(top) = stack.last_mut() else {
                return Visited::default();
            };
            if let Some((child, cell, first)) = top.next_child() {
                let step = if let Mdast::Html(html) = child
                    && Self::holds_blocks(top.node)
                {
                    let then = self.flush_after(top.node, top.index);
                    Step::Done(self.html_block(child, &html.value, first, then))
                } else if matches!(top.node, Mdast::TableRow(_)) && top.index > top.columns {
                    // A cell past the last of the header is no part of the row.
                    Step::Done(Visited::cut(span_of(child)))
                } else if !Self::holds_blocks(top.node)
                    && let Some((scope, close, open_span, close_span)) =
                        Self::raw_pair(top.node, top.base + top.index - 1)
                {
                    // The siblings through the closing tag are the element's.
                    let at = top.base + top.index - 1;
                    let siblings = top.node.children().map_or(&[][..], Vec::as_slice);
                    let inside = siblings.get(at + 1..close).unwrap_or_default();
                    top.children.nth(close - at - 1);
                    top.index = close + 1 - top.base;
                    let titles = match siblings.get(at) {
                        Some(Mdast::Html(html)) => {
                            html_text::tag_titles(self.source, &open_span, &html.value)
                        }
                        _ => Vec::new(),
                    };
                    let kind = Kind::Raw {
                        scope,
                        open: open_span,
                        close: close_span,
                        titles,
                    };
                    let mut frame = Frame::new(top.node, kind, cell, first);
                    frame.children = inside.iter();
                    frame.base = at + 1;
                    frame.columns = top.columns;
                    stack.push(frame);
                    continue;
                } else {
                    self.step(child)
                };
                match step {
                    Step::Done(visited) => {
                        top.inner = mem::take(&mut top.inner).append(visited);
                    }
                    Step::Open(kind) => {
                        let columns = if let Mdast::Table(table) = top.node {
                            table
                                .children
                                .first()
                                .map_or(0, |row| row.children().map_or(0, Vec::len))
                        } else {
                            top.columns
                        };
                        let mut frame = Frame::new(child, kind, cell, first);
                        frame.columns = columns;
                        stack.push(frame);
                    }
                }
                continue;
            }
            let Some(frame) = stack.pop() else {
                return Visited::default();
            };
            let visited = self.finish(frame);
            match stack.last_mut() {
                Some(parent) => {
                    parent.inner = mem::take(&mut parent.inner).append(visited);
                }
                None => return visited,
            }
        }
    }

    /// The byte where Vale's walk flushes the text it holds when it reaches
    /// `block`, a child of a container, or `None` when it flushes nothing.
    ///
    /// A block that opens an element that is not inline or void flushes the
    /// text before it: a paragraph, a list, a quote, a code block, a table, a
    /// heading, and an HTML block that starts such a tag.
    /// A thematic break renders a void tag, and a definition renders nothing.
    /// The MDX blocks and display math are not probed, and flush nothing.
    fn flush_at(&self, block: &Mdast) -> Option<usize> {
        let span = span_of(block)?;
        match block {
            Mdast::Html(html) => html_text::flushes_at(self.source, &span, &html.value),
            Mdast::ThematicBreak(_)
            | Mdast::Definition(_)
            | Mdast::FootnoteDefinition(_)
            | Mdast::Yaml(_)
            | Mdast::Toml(_)
            | Mdast::Math(_)
            | Mdast::MdxJsxFlowElement(_)
            | Mdast::MdxFlowExpression(_)
            | Mdast::MdxjsEsm(_) => None,
            Mdast::Root(_)
            | Mdast::Blockquote(_)
            | Mdast::List(_)
            | Mdast::ListItem(_)
            | Mdast::Paragraph(_)
            | Mdast::Heading(_)
            | Mdast::Code(_)
            | Mdast::Table(_)
            | Mdast::TableRow(_)
            | Mdast::TableCell(_)
            | Mdast::Break(_)
            | Mdast::InlineCode(_)
            | Mdast::InlineMath(_)
            | Mdast::Delete(_)
            | Mdast::Emphasis(_)
            | Mdast::MdxTextExpression(_)
            | Mdast::FootnoteReference(_)
            | Mdast::Image(_)
            | Mdast::ImageReference(_)
            | Mdast::MdxJsxTextElement(_)
            | Mdast::Link(_)
            | Mdast::LinkReference(_)
            | Mdast::Strong(_)
            | Mdast::Text(_) => Some(span.start),
        }
    }

    /// The byte where the first flush comes among the children of `parent`
    /// from the index `after` on: the start of the next block that flushes,
    /// or the end of the parent, whose end tag flushes.
    fn flush_after(&self, parent: &Mdast, after: usize) -> usize {
        parent
            .children()
            .and_then(|children| children.get(after..))
            .and_then(|rest| rest.iter().find_map(|block| self.flush_at(block)))
            .or_else(|| span_of(parent).map(|span| span.end))
            .unwrap_or(self.source.len())
    }

    /// Whether the children of `node` are blocks, so that an HTML node among
    /// them is an HTML block and not an inline tag.
    const fn holds_blocks(node: &Mdast) -> bool {
        matches!(
            node,
            Mdast::Root(_)
                | Mdast::Blockquote(_)
                | Mdast::ListItem(_)
                | Mdast::FootnoteDefinition(_)
                | Mdast::MdxJsxFlowElement(_)
        )
    }

    /// An HTML block: a gap in the nodes around it, and the nodes for the text
    /// Vale reads in it.
    ///
    /// `then` is the byte where the next flush comes after the block.
    fn html_block(&self, node: &Mdast, value: &str, first: bool, then: usize) -> Visited {
        let span = span_of(node);
        let mut visited = Visited::cut(span.clone());
        if let Some(span) = span
            && let Some(nodes) = html_text::nodes(self.source, &span, value, first, then)
        {
            visited.nodes = nodes;
        }
        visited
    }

    /// The element of the inline HTML tag at `at` among the children of
    /// `parent`: the scope Vale gives its tag, the index of the closing tag
    /// among the siblings, and the spans of the two tags.
    ///
    /// Vale scopes `b` and `strong` as strong text, `i` and `em` as emphasis,
    /// `code` and `tt` as code, and `a` as a link, as it does the elements
    /// its Markdown converter writes. Only a closing tag among the siblings
    /// ends the element.
    fn raw_pair(parent: &Mdast, at: usize) -> Option<(Scope, usize, Range<usize>, Range<usize>)> {
        let children = parent.children()?;
        let Mdast::Html(opening) = children.get(at)? else {
            return None;
        };
        let name = raw_tag(&opening.value, false)?;
        let scope = match name.as_str() {
            "b" | "strong" => Scope::Strong,
            "i" | "em" => Scope::Emphasis,
            "code" | "tt" => Scope::Code,
            "a" => Scope::Link,
            _ => return None,
        };
        let mut depth = 0_usize;
        for (index, sibling) in children.iter().enumerate().skip(at + 1) {
            let Mdast::Html(html) = sibling else {
                continue;
            };
            if raw_tag(&html.value, false).as_deref() == Some(name.as_str()) {
                depth += 1;
            } else if raw_tag(&html.value, true).as_deref() == Some(name.as_str()) {
                if depth == 0 {
                    return Some((scope, index, span_of(children.get(at)?)?, span_of(sibling)?));
                }
                depth -= 1;
            }
        }
        None
    }

    /// A node that holds the span of an inline HTML element.
    ///
    /// Its first and last children are empty text at the end of its opening
    /// tag and at the start of its closing tag, so that the tags are its
    /// markers.
    fn carrier(span: Range<usize>, inner_start: usize, inner_end: usize) -> Mdast {
        let position = |from: usize, to: usize| Position::new(1, 1, from, 1, 1, to);
        let marker = |offset: usize| {
            Mdast::Text(Text {
                value: String::new(),
                position: Some(position(offset, offset)),
            })
        };
        Mdast::Strong(Strong {
            children: vec![marker(inner_start), marker(inner_end)],
            position: Some(position(span.start, span.end)),
        })
    }

    /// A text node: its escapes lose their backslash, and an `ftp://` address
    /// is a gap, as Vale's converter makes a link of it.
    fn text(&self, node: &Mdast) -> Visited {
        let Some(span) = span_of(node) else {
            return Visited::default();
        };
        let Some(text) = self.source.get(span.clone()) else {
            return Visited::default();
        };
        let before = self
            .source
            .get(..span.start)
            .and_then(|head| head.chars().next_back());
        let mut cuts = ftp_addresses(text, before)
            .into_iter()
            .map(|found| span.start + found.start..span.start + found.end)
            .collect::<Vec<_>>();
        let bytes = text.as_bytes();
        let mut at = 0;
        while let Some(&byte) = bytes.get(at) {
            if byte == b'\\' && bytes.get(at + 1).is_some_and(u8::is_ascii_punctuation) {
                let start = span.start + at;
                if !cuts.iter().any(|cut| cut.contains(&start)) {
                    cuts.push(start..start + 1);
                }
                at += 2;
            } else {
                at += 1;
            }
        }
        cuts.sort_by_key(|cut| cut.start);
        Visited {
            cuts,
            ..Visited::default()
        }
    }

    /// A hard break written with a backslash loses the backslash.
    fn hard_break(&self, node: &Mdast) -> Visited {
        let span =
            span_of(node).filter(|span| self.source.as_bytes().get(span.start) == Some(&b'\\'));
        Visited::cut(span.map(|span| span.start..span.start + 1))
    }

    fn step(&self, node: &Mdast) -> Step {
        match node {
            Mdast::Paragraph(_) => Step::Open(Kind::Scoped(Scope::Paragraph)),
            Mdast::Heading(heading) => {
                Step::Open(Scope::heading(heading.depth).map_or(Kind::Transparent, Kind::Scoped))
            }
            Mdast::Blockquote(_) => Step::Open(Kind::Scoped(Scope::Blockquote)),
            // A reference is a number that Vale places elsewhere, and a
            // definition that nothing references is not rendered.
            Mdast::FootnoteReference(_) => Step::Done(Visited::cut(span_of(node))),
            Mdast::FootnoteDefinition(definition)
                if !self.referenced.contains(&definition.identifier) =>
            {
                Step::Done(Visited::cut(span_of(node)))
            }
            Mdast::ListItem(_) | Mdast::FootnoteDefinition(_) => {
                Step::Open(Kind::Scoped(Scope::List))
            }
            Mdast::TableRow(_) => Step::Open(Kind::Scoped(Scope::TableRow)),
            Mdast::TableCell(_) => Step::Open(Kind::Cell),
            Mdast::Link(_) if !self.is_linked(node) => Step::Open(Kind::Transparent),
            Mdast::Link(_) | Mdast::LinkReference(_) => Step::Open(Kind::Scoped(Scope::Link)),
            Mdast::InlineCode(_) => Step::Open(Kind::Scoped(Scope::Code)),
            Mdast::Strong(_) => Step::Open(Kind::Scoped(Scope::Strong)),
            Mdast::Emphasis(_) => Step::Open(Kind::Scoped(Scope::Emphasis)),
            Mdast::Image(_) | Mdast::ImageReference(_) => Step::Done(self.alt(node)),
            Mdast::MdxJsxFlowElement(_) | Mdast::MdxJsxTextElement(_) => Step::Open(Kind::Jsx),
            // Syntax that is not prose is a gap in the text around it.
            Mdast::Code(_) => {
                self.code_blocks.set(self.code_blocks.get() + 1);
                Step::Done(Visited::cut(span_of(node)))
            }
            Mdast::Html(html) => {
                let span = span_of(node);
                let nodes = span.as_ref().map_or_else(Vec::new, |span| {
                    html_text::tag_titles(self.source, span, &html.value)
                });
                Step::Done(Visited {
                    nodes,
                    ..Visited::tag(span)
                })
            }
            Mdast::Yaml(_) | Mdast::Toml(_) => {
                let span = span_of(node);
                let toml = matches!(node, Mdast::Toml(_));
                let nodes = span
                    .as_ref()
                    .map_or_default(|span| frontmatter::nodes(self.source, span, toml));
                Step::Done(Visited {
                    nodes,
                    ..Visited::cut(span)
                })
            }
            Mdast::MdxFlowExpression(_) | Mdast::MdxTextExpression(_) => {
                Step::Done(self.expression(node))
            }
            Mdast::MdxjsEsm(_) | Mdast::InlineMath(_) | Mdast::Math(_) => {
                Step::Done(Visited::cut(span_of(node)))
            }
            Mdast::Definition(definition) => {
                Step::Done(self.definition(node, definition.title.is_some()))
            }
            Mdast::Root(_) | Mdast::List(_) | Mdast::Delete(_) | Mdast::Table(_) => {
                Step::Open(Kind::Transparent)
            }
            Mdast::Break(_) => Step::Done(self.hard_break(node)),
            Mdast::Text(_) => Step::Done(self.text(node)),
            Mdast::ThematicBreak(_) => Step::Done(Visited::default()),
        }
    }

    /// What a node gives its parent once the nodes below it are visited.
    fn finish(&self, frame: Frame<'_>) -> Visited {
        match frame.kind {
            Kind::Scoped(scope) => {
                let mut visited = self.scoped(frame.node, scope, frame.inner, frame.first);
                // The title of a tag in the text of a link is that tag's, so it
                // sits beside the link, and `title & link` leaves it out.
                let mut nested = Vec::new();
                if scope == Scope::Link
                    && let Some(link) = visited
                        .nodes
                        .last_mut()
                        .filter(|node| node.scope == Scope::Link)
                {
                    let (titles, rest) = mem::take(&mut link.children)
                        .into_iter()
                        .partition(|child| child.scope == Scope::Title);
                    link.children = rest;
                    nested = titles;
                }
                if let Mdast::Link(link) = frame.node
                    && link.title.is_some()
                    && let Some(span) = span_of(frame.node)
                {
                    let from = frame
                        .node
                        .children()
                        .and_then(|kids| kids.last())
                        .and_then(span_of)
                        .map_or(span.start, |last| last.end);
                    if let Some(title) = self.title(&span, from, true) {
                        match visited.nodes.last_mut() {
                            Some(node) if node.scope == Scope::Link => node.children.push(title),
                            _ => visited.nodes.push(title),
                        }
                    }
                }
                visited.nodes.extend(nested);
                visited
            }
            Kind::Cell => self.scoped(frame.node, frame.cell, frame.inner, frame.first),
            Kind::Raw {
                scope,
                open,
                close,
                titles,
            } => {
                let inner = Visited::tag(Some(open.clone()))
                    .append(frame.inner)
                    .append(Visited::tag(Some(close.clone())));
                let tags = inner.tags.clone();
                let carrier = Self::carrier(open.start..close.end, open.end, close.start);
                let mut visited = self.scoped(&carrier, scope, inner, frame.first);
                visited.tags = tags;
                visited.nodes.splice(0..0, titles);
                visited
            }
            Kind::Jsx => self.jsx(frame.node, frame.inner),
            Kind::Transparent => Self::strikethrough(frame.node, frame.inner),
        }
    }

    /// Makes each paragraph of the tight list item `node` close where the
    /// next flush comes.
    ///
    /// A tight item has no `p` around its text, so Vale reads the text at the
    /// next tag that opens a block inside the item, such as a nested list, or
    /// when the item closes.
    /// A comment indented under the text reaches the text unless a block
    /// opens before it.
    fn close_tight_item(&self, node: &Mdast, children: &mut [Node], element_end: usize) {
        if let Mdast::ListItem(item) = node
            && !item.spread
        {
            let blocks = node.children().map_or(&[][..], Vec::as_slice);
            for child in children {
                if child.scope != Scope::Paragraph {
                    continue;
                }
                let position = blocks.iter().position(|block| {
                    matches!(block, Mdast::Paragraph(_))
                        && span_of(block).is_some_and(|span| {
                            span.start <= child.span.start && child.span.start < span.end
                        })
                });
                child.closes = position
                    .and_then(|index| blocks.get(index + 1..))
                    .and_then(|rest| rest.iter().find_map(|block| self.flush_at(block)))
                    .unwrap_or(element_end);
            }
        }
    }

    /// A node of `scope` over `node`, holding the nodes in `inner`.
    ///
    /// Syntax that is not prose and sits inside the node is a gap between its
    /// segments.
    /// A table row, a block quote and a list item have the segments of the
    /// nodes inside them.
    /// No marker, bullet or indentation is in their text.
    /// A container with no prose inside it gives no node.
    fn scoped(&self, node: &Mdast, scope: Scope, inner: Visited, first: bool) -> Visited {
        let Some(span) = span_of(node).map(|span| self.trimmed(scope, span)) else {
            return inner;
        };
        let Markup { span, own, up } = self.markup(node, scope, span);
        let element_end = span_of(node).map_or(span.end, |element| element.end);
        let Visited {
            nodes: mut children,
            mut cuts,
            tags,
        } = inner;
        self.close_tight_item(node, &mut children, element_end);
        cuts.extend(own);
        let mut segments = match scope {
            // A container reads as the content inside it.
            Scope::TableRow | Scope::Blockquote | Scope::List => children
                .iter()
                .flat_map(|child| child.segments.clone())
                .collect(),
            Scope::Heading1
            | Scope::Heading2
            | Scope::Heading3
            | Scope::Heading4
            | Scope::Heading5
            | Scope::Heading6
            | Scope::Heading0
            | Scope::Heading7
            | Scope::Heading8
            | Scope::Heading9 => {
                let atx = self
                    .source
                    .get(span.clone())
                    .is_some_and(|text| !text.contains('\n'));
                self.heading_text(self.lines_of(&span, &cuts), atx)
            }
            Scope::Paragraph
            | Scope::TableHeader
            | Scope::TableCell
            | Scope::TableCaption
            | Scope::FigureCaption
            | Scope::Link
            | Scope::Code
            | Scope::Strong
            | Scope::Emphasis
            | Scope::Alt
            | Scope::Title
            | Scope::Image
            | Scope::Description
            | Scope::Meta
            | Scope::FrontMatter
            | Scope::Raw
            | Scope::CommentLine
            | Scope::CommentBlock
            | Scope::CommentDocLine
            | Scope::CommentDocBlock => self.lines_of(&span, &cuts),
        };
        let inline_html = Self::reads_inline_html(scope);
        let flushes = if inline_html {
            html::flush_tags(self.source, &tags)
        } else {
            Vec::new()
        };
        let reads = if inline_html {
            let (kept, reads) = html::inline_reads(self.source, span.start, segments, &tags);
            segments = kept;
            reads
        } else if matches!(scope, Scope::TableRow | Scope::Blockquote | Scope::List) {
            children
                .iter()
                .flat_map(|child| (0..child.segments.len()).map(|index| child.reading(index)))
                .collect()
        } else {
            Vec::new()
        };
        // A node with no text gives way to the nodes below it.
        let nodes = if segments.is_empty() {
            children
        } else {
            Node::from_segments(scope, segments, children)
                .map(|node| Node {
                    first,
                    closes: element_end,
                    flushes,
                    ..node.with_reads(reads)
                })
                .into_iter()
                .collect()
        };
        cuts.extend(up);
        // The tags of an inline element belong to the paragraph around it.
        let tags = if matches!(scope, Scope::Link | Scope::Strong | Scope::Emphasis) {
            tags
        } else {
            Vec::new()
        };
        Visited { nodes, cuts, tags }
    }

    /// Whether the text of a node of `scope` holds the inline HTML that joins
    /// and flushes it.
    const fn reads_inline_html(scope: Scope) -> bool {
        matches!(
            scope,
            Scope::Paragraph
                | Scope::Heading1
                | Scope::Heading2
                | Scope::Heading3
                | Scope::Heading4
                | Scope::Heading5
                | Scope::Heading6
                | Scope::Heading0
                | Scope::Heading7
                | Scope::Heading8
                | Scope::Heading9
                | Scope::TableHeader
                | Scope::TableCell
        )
    }

    /// The syntax of an inline node that is not its text.
    ///
    /// Strong text, emphasis and a link written with brackets lose the
    /// markers around what they hold, in their own text and in the text
    /// around them.
    /// A link whose text is its address loses the whole link in the text
    /// around it.
    /// A code span loses its backticks the same way, and keeps its content.
    /// An autolink and a bare URL keep their address as their own text, and
    /// the text around them loses the whole of it.
    /// A link that is not its own address, such as a bare email address,
    /// stays in the text around it, with the angle brackets cut if it has
    /// them.
    fn markup(&self, node: &Mdast, scope: Scope, span: Range<usize>) -> Markup {
        match scope {
            Scope::Strong | Scope::Emphasis => {
                let cuts = Self::wrapper(node, &span);
                Markup {
                    span,
                    own: cuts.clone(),
                    up: cuts,
                }
            }
            Scope::Link => {
                let text = self.source.get(span.clone()).unwrap_or_default();
                if text.starts_with('[') || matches!(node, Mdast::Strong(_)) {
                    let cuts = Self::wrapper(node, &span);
                    let address = self.is_address(node, text, false);
                    let up = if address {
                        vec![span.clone()]
                    } else {
                        cuts.clone()
                    };
                    return Markup {
                        span,
                        own: cuts,
                        up,
                    };
                }
                let angle = text.starts_with('<') && text.ends_with('>') && text.len() >= 2;
                let inner = if angle {
                    span.start + 1..span.end - 1
                } else {
                    span.clone()
                };
                if self.is_address(node, text, angle) {
                    return Markup {
                        span: inner,
                        own: Vec::new(),
                        up: vec![span],
                    };
                }
                let cuts = if angle {
                    vec![span.start..inner.start, inner.end..span.end]
                } else {
                    Vec::new()
                };
                Markup {
                    span: inner,
                    own: cuts.clone(),
                    up: cuts,
                }
            }
            Scope::Code => {
                let content = self.code_content(&span);
                let cuts = vec![span.start..content.start, content.end..span.end];
                Markup {
                    span: content,
                    own: cuts.clone(),
                    up: cuts,
                }
            }
            Scope::Paragraph
            | Scope::Heading1
            | Scope::Heading2
            | Scope::Heading3
            | Scope::Heading4
            | Scope::Heading5
            | Scope::Heading6
            | Scope::Heading0
            | Scope::Heading7
            | Scope::Heading8
            | Scope::Heading9
            | Scope::Blockquote
            | Scope::List
            | Scope::TableRow
            | Scope::TableHeader
            | Scope::TableCell
            | Scope::TableCaption
            | Scope::FigureCaption
            | Scope::Alt
            | Scope::Title
            | Scope::Image
            | Scope::Description
            | Scope::Meta
            | Scope::FrontMatter
            | Scope::Raw
            | Scope::CommentLine
            | Scope::CommentBlock
            | Scope::CommentDocLine
            | Scope::CommentDocBlock => Markup {
                span,
                own: Vec::new(),
                up: Vec::new(),
            },
        }
    }

    /// Whether a link is one that Vale's converter makes.
    ///
    /// A link written with brackets or angle brackets is.
    /// A bare address is when it starts with a lower-case `http://`,
    /// `https://` or `www.`, or is an email address.
    /// markdown-rs also reads an address with an upper-case scheme or a
    /// `mailto:` prefix as a link, which Vale's converter leaves as text.
    fn is_linked(&self, node: &Mdast) -> bool {
        let text = span_of(node)
            .and_then(|span| self.source.get(span))
            .unwrap_or_default();
        if text.starts_with(['[', '<']) {
            return true;
        }
        text.starts_with("http://")
            || text.starts_with("https://")
            || text.starts_with("www.")
            || text.split_once('@').is_some_and(|(name, _)| {
                !name.is_empty()
                    && name.bytes().all(|c| {
                        c.is_ascii_alphanumeric() || matches!(c, b'.' | b'_' | b'+' | b'-')
                    })
            })
    }

    /// Whether the link `node`, written as `text`, is its own address, which
    /// Vale's converter keeps out of the text.
    ///
    /// Vale drops the text of a link when it equals the link's `href`.
    /// goldmark writes that `href` percent-encoded, as [`goldmark_href`]
    /// does, so the text is compared with it and not with the destination.
    /// Thus `[a.md](a.md)` and `[a%20b](a%20b)` are their own address, and
    /// `[ä.md](ä.md)` and `[a b](<a b>)` are not.
    /// A link with brackets is when its text repeats that address, whatever
    /// the destination is, and Vale reads it as a URL.
    /// An autolink is when that address is what is written, which holds for
    /// a web address in angle brackets or written bare with a lower-case
    /// scheme.
    /// So is another address in angle brackets that is its own destination,
    /// such as `<mailto:x>`, which goldmark and Vale drop from the text.
    /// A bare email address, a bare `www.` address and a bare `mailto:`
    /// address are not: the converter writes a different destination for the
    /// first two and reads the last as text, so their words stay in the text.
    fn is_address(&self, node: &Mdast, text: &str, angle: bool) -> bool {
        let destination = if let Mdast::Link(link) = node {
            link.url.as_str()
        } else if let Mdast::LinkReference(reference) = node
            && let Some(address) = self.addresses.get(&reference.identifier)
        {
            address.as_str()
        } else {
            return false;
        };
        let destination = goldmark_href(destination);
        if text.starts_with('[') {
            return matches!(
                node.children().map(Vec::as_slice),
                Some([Mdast::Text(shown)]) if shown.value == destination
            );
        }
        let shown = if angle {
            text.get(1..text.len() - 1).unwrap_or_default()
        } else {
            text
        };
        shown == destination
            && (angle || shown.starts_with("http://") || shown.starts_with("https://"))
    }

    /// The markers before the first child of `node` and after its last.
    ///
    /// A node with no child is cut whole.
    fn wrapper(node: &Mdast, span: &Range<usize>) -> Vec<Range<usize>> {
        let children = node.children().map_or(&[][..], Vec::as_slice);
        match (
            children.first().and_then(span_of),
            children.last().and_then(span_of),
        ) {
            (Some(first), Some(last)) => {
                vec![span.start..first.start, last.end..span.end]
            }
            _ => vec![span.clone()],
        }
    }

    /// A strikethrough loses its tildes, and gives the nodes inside it to its
    /// parent.
    fn strikethrough(node: &Mdast, mut inner: Visited) -> Visited {
        if let (Mdast::Delete(_), Some(span)) = (node, span_of(node)) {
            inner.cuts.extend(Self::wrapper(node, &span));
        }
        inner
    }

    /// The content of the code span at `span`: between its backtick runs,
    /// without the one space a span pads its content with when both ends have
    /// one.
    fn code_content(&self, span: &Range<usize>) -> Range<usize> {
        let text = self.source.get(span.clone()).unwrap_or_default();
        if text.starts_with('<')
            && let (Some(open), Some(close)) = (text.find('>'), text.rfind('<'))
            && open < close
        {
            return span.start + open + 1..span.start + close;
        }
        let ticks = text.len() - text.trim_start_matches('`').len();
        if text.len() < ticks * 2 {
            return span.clone();
        }
        let start = span.start + ticks;
        let end = span.end - ticks;
        let content = self.source.get(start..end).unwrap_or_default();
        let padding = |c: char| c == ' ' || c == '\n';
        let padded = content.len() >= 2
            && content.starts_with(padding)
            && content.ends_with(padding)
            && !content.chars().all(padding);
        if padded {
            start + 1..end - 1
        } else {
            start..end
        }
    }

    /// The lines of `span` that no cut covers, one range each.
    fn lines_of(&self, span: &Range<usize>, cuts: &[Range<usize>]) -> Vec<Range<usize>> {
        self.segments(span, cuts)
            .into_iter()
            .flat_map(|part| self.lines(part))
            .collect()
    }

    /// The lines of a heading without its syntax.
    ///
    /// An ATX heading loses its opening `#` run with the space after it, and
    /// its closing `#` run.
    /// A setext heading loses its underline.
    fn heading_text(&self, mut lines: Vec<Range<usize>>, atx: bool) -> Vec<Range<usize>> {
        let text_of = |line: &Range<usize>| self.source.get(line.clone()).unwrap_or_default();
        if atx {
            if let Some(first) = lines.first_mut() {
                let text = text_of(first);
                let rest = text.trim_start_matches('#');
                first.start += text.len() - rest.trim_start().len();
            }
            if let Some(last) = lines.last_mut() {
                let text = text_of(last);
                let body = text.trim_end_matches('#');
                if body.len() < text.len() && (body.is_empty() || body.ends_with([' ', '\t'])) {
                    last.end = last.start + body.trim_end().len();
                }
            }
        } else if lines.last().is_some_and(|last| {
            let text = text_of(last).trim();
            !text.is_empty() && (text.chars().all(|c| c == '=') || text.chars().all(|c| c == '-'))
        }) {
            lines.pop();
        }
        lines.retain(|line| !line.is_empty());
        lines
    }

    /// `span` without the line ending and padding that markdown-rs leaves at
    /// its end.
    ///
    /// A table cell also loses its pipes and padding.
    /// markdown-rs starts a cell at its leading pipe and ends it after the
    /// padding or the closing pipe.
    fn trimmed(&self, scope: Scope, span: Range<usize>) -> Range<usize> {
        let Some(text) = self.source.get(span.clone()) else {
            return span;
        };
        let text = text.trim_end();
        if !matches!(scope, Scope::TableHeader | Scope::TableCell) {
            return span.start..span.start + text.len();
        }
        let unpadded = text.trim_start();
        let pipe = usize::from(unpadded.starts_with('|'));
        let body = unpadded.get(pipe..).unwrap_or_default();
        let start = span.start
            + (text.len() - unpadded.len())
            + pipe
            + (body.len() - body.trim_start().len());
        let mut body = body.trim();
        if let Some(before) = body
            .strip_suffix('|')
            .filter(|before| !before.ends_with('\\'))
        {
            body = before.trim_end();
        }
        start..start + body.len()
    }

    /// The parts of `span` that no cut covers, without the whitespace at
    /// either end of the text.
    ///
    /// Returns no part when only whitespace is left.
    fn segments(&self, span: &Range<usize>, cuts: &[Range<usize>]) -> Vec<Range<usize>> {
        let mut inside: Vec<&Range<usize>> = cuts
            .iter()
            .filter(|cut| cut.start < span.end && span.start < cut.end)
            .collect();
        inside.sort_by_key(|cut| cut.start);
        let mut parts = Vec::new();
        let mut at = span.start;
        for cut in inside {
            if at < cut.start {
                parts.push(at..cut.start);
            }
            at = at.max(cut.end);
        }
        if at < span.end {
            parts.push(at..span.end);
        }
        let text_of = |part: &Range<usize>| self.source.get(part.clone()).unwrap_or_default();
        if parts.iter().all(|part| text_of(part).trim().is_empty()) {
            return Vec::new();
        }
        if let Some(first) = parts.first_mut() {
            first.start += text_of(first).len() - text_of(first).trim_start().len();
        }
        if let Some(last) = parts.last_mut() {
            last.end = last.start + text_of(last).trim_end().len();
        }
        parts.retain(|part| !part.is_empty());
        parts
    }

    /// One range for each line of `part`, without its line ending.
    ///
    /// A line after the first also loses the indentation and the block quote
    /// markers that open it: they belong to the containers around the node.
    /// A line that is left empty gives no range.
    fn lines(&self, part: Range<usize>) -> Vec<Range<usize>> {
        let Some(text) = self.source.get(part.clone()) else {
            return vec![part];
        };
        let count = text.split('\n').count();
        let mut lines = Vec::new();
        let mut offset = 0;
        for (index, line) in text.split('\n').enumerate() {
            let start = part.start + offset;
            offset += line.len() + 1;
            let line = if index + 1 < count {
                line.strip_suffix('\r').unwrap_or(line)
            } else {
                line
            };
            let opening = if index == 0 { 0 } else { opening_length(line) };
            lines.push(start + opening..start + line.len());
        }
        lines.retain(|line| !line.is_empty());
        lines
    }

    /// A JSX element: its children stay prose and its tags are gaps.
    fn jsx(&self, node: &Mdast, mut visited: Visited) -> Visited {
        if let Some(span) = span_of(node) {
            let children = node.children().map_or(&[][..], Vec::as_slice);
            let first = children.first().and_then(span_of);
            let last = children.last().and_then(span_of);
            // The `title` attribute of the opening tag is a title.
            let opening = span.start..first.as_ref().map_or(span.end, |first| first.start);
            if let Some(tag) = self.source.get(opening.clone()) {
                let titles = html_text::tag_titles(self.source, &opening, tag);
                visited.nodes.splice(0..0, titles);
            }
            if let (Some(first), Some(last)) = (first, last) {
                visited.cuts.push(span.start..first.start);
                visited.cuts.push(last.end..span.end);
            } else {
                if self.reads_as_code(node, &span) {
                    visited.nodes.push(Self::code(span.clone()));
                }
                visited.cuts.push(span);
            }
        }
        visited
    }

    /// Whether Vale reads the JSX element `node`, which holds no node, as
    /// code: a flow element, and a text element written as `<Tag />`.
    /// An open and a close tag with nothing between them, inside a line of
    /// text, are no code.
    fn reads_as_code(&self, node: &Mdast, span: &Range<usize>) -> bool {
        matches!(node, Mdast::MdxJsxFlowElement(_))
            || self
                .source
                .get(span.clone())
                .is_some_and(|text| text.ends_with("/>"))
    }

    /// An `{…}` expression, which Vale reads as code, and not a `{/* … */}`
    /// comment, which it turns into an HTML comment.
    fn expression(&self, node: &Mdast) -> Visited {
        let span = span_of(node);
        let mut visited = Visited::cut(span.clone());
        if let Some(span) = span
            && !self
                .source
                .get(span.clone())
                .is_some_and(|text| text.starts_with("{/*") && text.ends_with("*/}"))
        {
            visited.nodes.push(Self::code(span));
        }
        visited
    }

    /// A node of code over `span`, as written.
    fn code(span: Range<usize>) -> Node {
        Node {
            closes: span.end,
            ..Node::new(Scope::Code, span, Vec::new())
        }
    }

    /// The `alt` node of an image or an image reference: the label between
    /// `![` and its closing bracket.
    fn alt(&self, node: &Mdast) -> Visited {
        let span = span_of(node).and_then(|span| {
            let label = self.source.get(span.clone())?.strip_prefix("![")?;
            let length = label_length(label).filter(|length| *length > 0)?;
            let start = span.start + "![".len();
            Some(start..start + length)
        });
        let mut nodes: Vec<Node> = span
            .clone()
            .map(|span| Node::new(Scope::Alt, span, Vec::new()))
            .into_iter()
            .collect();
        if matches!(node, Mdast::Image(_) | Mdast::ImageReference(_))
            && let Some(whole) = span_of(node)
        {
            // The image holds its title, so `title & image` selects it. An
            // image with no alt text holds an empty span, which reads as
            // nothing.
            let has_title = matches!(node, Mdast::Image(image) if image.title.is_some());
            let titles: Vec<Node> = has_title
                .then(|| self.title(&whole, whole.start, true))
                .flatten()
                .into_iter()
                .collect();
            let label = span.unwrap_or_else(|| {
                let start = whole.start + "![".len();
                start..start
            });
            nodes.push(Node::new(Scope::Image, label, titles));
        }
        Visited {
            nodes,
            cuts: span_of(node).into_iter().collect(),
            tags: Vec::new(),
        }
    }

    /// A link reference definition: a gap in the text, and the `title` node of
    /// its title.
    fn definition(&self, node: &Mdast, titled: bool) -> Visited {
        let span = span_of(node);
        let mut visited = Visited::cut(span.clone());
        if titled
            && let Some(span) = span
            && let Some(title) = self.title(&span, span.start, false)
        {
            visited.nodes.push(title);
        }
        visited
    }

    /// The `title` node of the title that ends the link, image or definition
    /// at `span`, looked for after the byte `from`.
    fn title(&self, span: &Range<usize>, from: usize, paren: bool) -> Option<Node> {
        title_range(self.source, from, span.end, paren)
            .map(|range| Node::new(Scope::Title, range, Vec::new()))
    }
}

/// The bytes between the delimiters of the title that ends `source[from..to]`,
/// when the text ends with one.
///
/// A title is written `"…"`, `'…'` or `(…)` after white space, and an inline
/// link or image closes with a `)` after it, which `paren` says to strip.
/// A backslash escapes a delimiter inside the title.
fn title_range(source: &str, from: usize, to: usize, paren: bool) -> Option<Range<usize>> {
    let mut text = source.get(from..to)?;
    if paren {
        text = text.strip_suffix(')')?;
    }
    let text = text.trim_end();
    let bytes = text.as_bytes();
    let close = text.len().checked_sub(1)?;
    let open = match bytes.get(close)? {
        b'"' => b'"',
        b'\'' => b'\'',
        b')' => b'(',
        _ => return None,
    };
    for at in (0..close).rev() {
        // Backslashes are counted only before a delimiter, so a long run of
        // them costs its length once.
        if bytes.get(at) != Some(&open) {
            continue;
        }
        let slashes = bytes
            .get(..at)?
            .iter()
            .rev()
            .take_while(|byte| **byte == b'\\')
            .count();
        if slashes % 2 == 0 {
            let spaced = at
                .checked_sub(1)
                .and_then(|before| bytes.get(before))
                .is_some_and(u8::is_ascii_whitespace);
            return (spaced && at + 1 < close).then_some(from + at + 1..from + close);
        }
    }
    None
}

/// The lower-case name of the tag `html`, which is one inline HTML tag, when
/// it opens an element (`closing` false) or closes one (`closing` true).
///
/// A tag with attributes opens an element, and a tag that ends in `/>` does
/// not.
fn raw_tag(html: &str, closing: bool) -> Option<String> {
    let rest = html.strip_prefix('<')?;
    let rest = if closing {
        rest.strip_prefix('/')?
    } else if rest.starts_with('/') || html.ends_with("/>") {
        return None;
    } else {
        rest
    };
    let name: String = rest
        .chars()
        .take_while(char::is_ascii_alphanumeric)
        .collect();
    let after = rest.get(name.len()..)?;
    let valid = if closing {
        after.trim() == ">"
    } else {
        after.starts_with(['>', ' ', '\t', '\n']) && html.ends_with('>')
    };
    (!name.is_empty() && valid).then(|| name.to_ascii_lowercase())
}

/// The identifiers of the footnotes that a reference in `root` names.
fn footnotes_referenced(root: &Mdast) -> HashSet<String> {
    let mut found = HashSet::new();
    let mut pending = vec![root];
    while let Some(node) = pending.pop() {
        if let Mdast::FootnoteReference(reference) = node {
            found.insert(reference.identifier.clone());
        }
        pending.extend(node.children().into_iter().flatten());
    }
    found
}

/// Where `text` holds an `ftp://` address that Vale's converter links.
///
/// Its linkify extension reads an address that starts at a line start or after
/// a space or one of `*`, `_`, `~` and `(`, and ends at the last byte its
/// pattern takes, less the closing punctuation.
/// `before` is the character in the file before `text`.
fn ftp_addresses(text: &str, before: Option<char>) -> Vec<Range<usize>> {
    static FTP: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(
            r#"^ftp://[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-z]+(?::\d+)?(?:[/#?][-a-zA-Z0-9@:%_+.~#$!?&/=\(\);,'">\^{}\[\]`]*)?"#,
        )
        .unwrap_or_else(|_| unreachable!("the pattern is a literal"))
    });
    let mut found = Vec::new();
    let mut from = 0;
    while let Some(offset) = text.get(from..).and_then(|rest| rest.find("ftp://")) {
        let start = from + offset;
        from = start + 1;
        let previous = text
            .get(..start)
            .and_then(|head| head.chars().next_back())
            .or(before);
        if previous.is_some_and(|c| !c.is_whitespace() && !matches!(c, '*' | '_' | '~' | '(')) {
            continue;
        }
        let Some(hit) = text.get(start..).and_then(|rest| FTP.find(rest)) else {
            continue;
        };
        let end = start + trimmed_address(hit.as_str());
        found.push(start..end);
        from = end.max(from);
    }
    found
}

/// The length of the address `matched` that Vale's converter keeps: closing
/// punctuation, an unbalanced `)` and a trailing character reference are not
/// part of it.
fn trimmed_address(matched: &str) -> usize {
    let bytes = matched.as_bytes();
    let mut end = bytes.len().saturating_sub(1);
    while end > 0
        && matches!(
            bytes.get(end),
            Some(b'?' | b'!' | b'.' | b',' | b':' | b'*' | b'_' | b'~')
        )
    {
        end -= 1;
    }
    end += 1;
    let head = bytes.get(..end).unwrap_or_default();
    let text = matched.get(..end).unwrap_or_default();
    match head.last() {
        Some(b')') => {
            let closing = text.matches(')').count();
            let opening = text.matches('(').count();
            end - closing.saturating_sub(opening)
        }
        Some(b';') => {
            let name = head.get(..end - 1).unwrap_or_default();
            let letters = name
                .iter()
                .rev()
                .take_while(|c| c.is_ascii_alphanumeric())
                .count();
            let amp = end - 2 - letters;
            if letters > 0 && head.get(amp) == Some(&b'&') {
                amp
            } else {
                end
            }
        }
        _ => end,
    }
}

fn span_of(node: &Mdast) -> Option<Range<usize>> {
    node.position()
        .map(|position| position.start.offset..position.end.offset)
}

/// The byte length of the indentation and block quote markers that open
/// `line`, a line of text inside a paragraph.
///
/// A line of a paragraph never starts with `>` unless a block quote around
/// the paragraph put it there, because a `>` would end the paragraph.
fn opening_length(line: &str) -> usize {
    let mut at = 0;
    loop {
        let rest = line.get(at..).unwrap_or_default();
        at += rest.len() - rest.trim_start_matches([' ', '\t']).len();
        if line.get(at..).is_some_and(|rest| rest.starts_with('>')) {
            at += 1;
        } else {
            return at;
        }
    }
}

/// The byte length of a link label that starts `label`, up to its closing
/// bracket.
fn label_length(label: &str) -> Option<usize> {
    let mut depth = 0_usize;
    let mut escaped = false;
    for (index, byte) in label.bytes().enumerate() {
        if escaped {
            escaped = false;
            continue;
        }
        match byte {
            b'\\' => escaped = true,
            b'[' => depth += 1,
            b']' if depth == 0 => return Some(index),
            b']' => depth -= 1,
            _ => {}
        }
    }
    None
}
