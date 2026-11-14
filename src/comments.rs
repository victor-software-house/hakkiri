//! Comments of source files into nodes, through tree-sitter.
//!
//! Only comments enter the tree.
//! The markers of a comment are stripped, and its text is read as Markdown.
//! Every span the Markdown parser gives maps back to the bytes of the file.

use std::iter;
use std::ops::Range;
use std::sync::LazyLock;

use regex::Regex;
use tree_sitter::{
    Language as Grammar, Node as SyntaxNode, Parser, Query, QueryCursor, StreamingIterator,
};

use crate::error::Error;
use crate::language::Language;
use crate::markdown::{self, Flavor};
use crate::pattern::Pattern;
use crate::tree::{Node, Scope, remap_nodes};
use crate::{ignore, jsdoc};

/// The grammar that finds the comments of a file.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Dialect {
    Rust,
    TypeScript,
    Tsx,
    JavaScript,
    Go,
    Swift,
    Yaml,
    Toml,
    Shell,
}

impl Dialect {
    /// The grammar that reads the comments of `language`, or `None` when
    /// hakkiri reads the comments of no such language.
    pub(crate) const fn of(language: Language) -> Option<Self> {
        match language {
            Language::Rust => Some(Self::Rust),
            Language::TypeScript => Some(Self::TypeScript),
            Language::Tsx => Some(Self::Tsx),
            Language::JavaScript => Some(Self::JavaScript),
            Language::Go => Some(Self::Go),
            Language::Swift => Some(Self::Swift),
            Language::Yaml => Some(Self::Yaml),
            Language::Toml => Some(Self::Toml),
            Language::Shell => Some(Self::Shell),
            Language::Markdown | Language::Mdx | Language::Text => None,
        }
    }

    fn grammar(self) -> Grammar {
        match self {
            Self::Rust => tree_sitter_rust::LANGUAGE.into(),
            Self::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            Self::Tsx => tree_sitter_typescript::LANGUAGE_TSX.into(),
            Self::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
            Self::Go => tree_sitter_go::LANGUAGE.into(),
            Self::Swift => tree_sitter_swift::LANGUAGE.into(),
            Self::Yaml => tree_sitter_yaml::LANGUAGE.into(),
            Self::Toml => tree_sitter_toml_ng::LANGUAGE.into(),
            Self::Shell => tree_sitter_bash::LANGUAGE.into(),
        }
    }

    /// The query that captures each comment node.
    const fn query(self) -> &'static str {
        match self {
            Self::Rust => "[(line_comment) (block_comment)] @comment",
            Self::Swift => "[(comment) (multiline_comment)] @comment",
            Self::TypeScript
            | Self::Tsx
            | Self::JavaScript
            | Self::Go
            | Self::Yaml
            | Self::Toml
            | Self::Shell => "(comment) @comment",
        }
    }

    /// Whether a comment opens with `#`, and not with `//`.
    const fn hashes(self) -> bool {
        matches!(self, Self::Yaml | Self::Toml | Self::Shell)
    }

    /// Whether a line comment keeps the indentation of its text, after the one
    /// space that follows its marker.
    ///
    /// Go reads a line indented further as a code block, and so does Markdown
    /// for the `#` comments of data and shell files.
    /// Rust, TypeScript, JavaScript and Swift remove the indentation that all
    /// the lines share, as rustdoc does.
    const fn keeps_indent(self) -> bool {
        matches!(self, Self::Go | Self::Yaml | Self::Toml | Self::Shell)
    }

    /// Whether `///` opens a doc comment, as `///` does in Rust and Swift.
    const fn has_doc_lines(self) -> bool {
        matches!(self, Self::Rust | Self::Swift)
    }

    /// Whether the text of a `/** */` comment holds `JSDoc` tags.
    const fn has_jsdoc(self) -> bool {
        matches!(self, Self::TypeScript | Self::Tsx | Self::JavaScript)
    }

    /// The pattern of a comment addressed to a tool, such as a linter switch
    /// or a build constraint, which holds no prose.
    fn directive(self) -> Option<&'static Regex> {
        let patterns = &*DIRECTIVES;
        match self {
            Self::Rust => None,
            Self::TypeScript | Self::Tsx | Self::JavaScript => Some(&patterns.script),
            Self::Go => Some(&patterns.go),
            Self::Swift => Some(&patterns.swift),
            Self::Yaml => Some(&patterns.yaml),
            Self::Toml => Some(&patterns.toml),
            Self::Shell => Some(&patterns.shell),
        }
    }
}

/// The directive patterns of the dialects, each matched against a comment as
/// written, marker included.
///
/// Vale drops the comments of Go, TypeScript and JavaScript that these two
/// patterns match.
/// The others name the tools of the languages hakkiri reads that Vale does
/// not read.
struct Directives {
    go: Regex,
    script: Regex,
    swift: Regex,
    yaml: Regex,
    toml: Regex,
    shell: Regex,
}

#[expect(
    clippy::expect_used,
    reason = "The patterns are constants that compile."
)]
static DIRECTIVES: LazyLock<Directives> = LazyLock::new(|| {
    let compile = |pattern: &str| Regex::new(pattern).expect("a directive pattern compiles");
    Directives {
        go: compile(r"^//(?:go:|line |export |extern |nolint\b|lint:ignore|[a-z0-9]+:[a-z0-9])"),
        script: compile(
            r"^//[ \t]*(?:eslint|@ts-|prettier-ignore|biome-ignore|istanbul\b|c8\b|v8\b|@flow\b|@jsx|jshint|jslint)|^/\*[ \t]*(?:eslint|globals?\b|jshint|jslint|istanbul\b|c8\b|@flow\b|@jsx|webpackChunkName|[@#]__PURE__)",
        ),
        swift: compile(r"^//[ \t]*(?:swiftlint:|swiftformat:|sourcery:)|^/\*[ \t]*swiftlint:"),
        yaml: compile(r"^#[ \t]*(?:yaml-language-server:|yamllint[ \t])"),
        toml: compile(r"^#:schema\b"),
        shell: compile(r"^#[ \t]*shellcheck[ \t]"),
    }
});

/// How a comment opens.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Marker {
    /// `//`
    Line,
    /// `#`
    Hash,
    /// Rust `///`
    OuterDocLine,
    /// Rust `//!`
    InnerDocLine,
    /// `/*`
    Block,
    /// `/**`, and Rust `/*!`
    DocBlock,
}

impl Marker {
    fn of(dialect: Dialect, text: &str) -> Option<Self> {
        let is_rust = dialect == Dialect::Rust;
        if dialect.hashes() {
            text.starts_with('#').then_some(Self::Hash)
        } else if let Some(after) = text.strip_prefix("//") {
            Some(if is_rust && after.starts_with('!') {
                Self::InnerDocLine
            } else if dialect.has_doc_lines() && after.starts_with('/') && !after.starts_with("//")
            {
                Self::OuterDocLine
            } else {
                Self::Line
            })
        } else {
            let rest = text.strip_prefix("/*")?;
            // `/**/` and `/***` are plain comments.
            let javadoc =
                rest.starts_with('*') && !rest.starts_with("*/") && !rest.starts_with("**");
            Some(if javadoc || (is_rust && rest.starts_with('!')) {
                Self::DocBlock
            } else {
                Self::Block
            })
        }
    }

    const fn scope(self) -> Scope {
        match self {
            Self::Line | Self::Hash => Scope::CommentLine,
            Self::OuterDocLine | Self::InnerDocLine => Scope::CommentDocLine,
            Self::Block => Scope::CommentBlock,
            Self::DocBlock => Scope::CommentDocBlock,
        }
    }

    /// The bytes of the opening marker.
    const fn opener(self) -> usize {
        match self {
            Self::Hash => 1,
            Self::Line | Self::Block => 2,
            Self::OuterDocLine | Self::InnerDocLine | Self::DocBlock => 3,
        }
    }

    const fn is_line(self) -> bool {
        matches!(
            self,
            Self::Line | Self::Hash | Self::OuterDocLine | Self::InnerDocLine
        )
    }

    /// Whether rustdoc reads a run of these across blank lines as one text.
    const fn is_doc_line(self) -> bool {
        matches!(self, Self::OuterDocLine | Self::InnerDocLine)
    }
}

/// One comment, or several adjacent line comments of the same marker.
#[derive(Debug)]
struct Group {
    marker: Marker,
    /// The file bytes of each comment, markers included.
    comments: Vec<Range<usize>>,
    /// The name a doc comment of the group opens with, from the declaration
    /// the last of its comments precedes.
    name: Option<String>,
}

/// One comment of a file, as `find` reads it.
struct Found {
    range: Range<usize>,
    marker: Marker,
    /// The name of the declaration the comment precedes, where the dialect has
    /// doc comments that open with it.
    name: Option<String>,
}

/// The `raw` roots of a source file, one per comment group, and the file bytes
/// of the text that a doc comment marks as no prose.
///
/// The Markdown of each comment is rewritten under `blocks` and `tokens`, as
/// [`crate::ignore`] explains for a file.
pub(crate) fn roots(
    dialect: Dialect,
    source: &str,
    blocks: &[&Pattern],
    tokens: &[&Pattern],
) -> Result<(Vec<Node>, Vec<Range<usize>>), Error> {
    let mut nodes = Vec::new();
    let mut masks = Vec::new();
    for group in groups(source, &find(dialect, source)?) {
        let (node, group_masks) = root(dialect, source, &group, blocks, tokens)?;
        nodes.push(node);
        masks.extend(group_masks);
    }
    Ok((nodes, masks))
}

/// Each comment in file order.
fn find(dialect: Dialect, source: &str) -> Result<Vec<Found>, Error> {
    let grammar = dialect.grammar();
    let mut parser = Parser::new();
    parser.set_language(&grammar)?;
    let tree = parser.parse(source, None).ok_or(Error::Syntax)?;
    let query = Query::new(&grammar, dialect.query())?;
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
    let mut found = Vec::new();
    while let Some(found_match) = matches.next() {
        for node in found_match.nodes_for_capture_index(0) {
            let range = node.byte_range();
            let text = source.get(range.clone()).unwrap_or_default();
            // A line comment ends before its line ending.
            let end = range.start + text.trim_end_matches(['\n', '\r']).len();
            if let Some(marker) = Marker::of(dialect, text)
                && !is_directive(dialect, text, range.start)
            {
                found.push(Found {
                    range: range.start..end,
                    marker,
                    name: declared_name(dialect, node, source),
                });
            }
        }
    }
    found.sort_by_key(|comment| comment.range.start);
    Ok(found)
}

/// The name of the declaration that the comment `node` precedes, which a Go doc
/// comment opens with: `Package` and the name for a package clause.
fn declared_name(dialect: Dialect, node: SyntaxNode<'_>, source: &str) -> Option<String> {
    if dialect != Dialect::Go {
        return None;
    }
    let mut next = node.next_named_sibling()?;
    // A comment that opens a block is a sibling of its list of statements.
    if next.kind() == "statement_list" {
        next = next.named_child(0)?;
    }
    // Go reads a comment as the doc of a declaration only when it starts its
    // own line and ends on the line before the declaration.
    if !starts_line(source, node.start_byte())
        || node.end_position().row + 1 != next.start_position().row
    {
        return None;
    }
    let text = |name: SyntaxNode<'_>| source.get(name.byte_range()).map(str::to_owned);
    match next.kind() {
        "package_clause" => text(next.named_child(0)?).map(|name| format!("Package {name}")),
        "function_declaration"
        | "method_declaration"
        | "type_spec"
        | "var_spec"
        | "const_spec"
        | "field_declaration"
        | "method_spec"
        | "method_elem" => text(next.child_by_field_name("name")?),
        "type_declaration" | "var_declaration" | "const_declaration" => {
            let mut cursor = next.walk();
            let name = next
                .named_children(&mut cursor)
                .find_map(|spec| spec.child_by_field_name("name"))?;
            text(name)
        }
        _ => None,
    }
}

/// Whether the comment `text` is addressed to a tool: a directive of the
/// dialect, or the `#!` line that opens a script.
/// `offset` is where the comment starts in the file.
fn is_directive(dialect: Dialect, text: &str, offset: usize) -> bool {
    (dialect.hashes() && offset == 0 && text.starts_with("#!"))
        || dialect
            .directive()
            .is_some_and(|directive| directive.is_match(text))
}

/// Joins each run of adjacent line comments that share a marker, as rustdoc
/// does. Doc line comments join across blank lines too, since rustdoc reads
/// the lines of one item's doc comments as one text, so a fence opened before
/// a blank line stays open after it.
fn groups(source: &str, comments: &[Found]) -> Vec<Group> {
    let mut groups: Vec<Group> = Vec::new();
    for Found {
        range,
        marker,
        name,
    } in comments
    {
        let joins = groups.last().is_some_and(|group| {
            group.marker == *marker
                && marker.is_line()
                && group.comments.last().is_some_and(|previous| {
                    let gap = source.get(previous.end..range.start).unwrap_or_default();
                    gap.trim().is_empty()
                        && (marker.is_doc_line() || gap.matches('\n').count() == 1)
                        && starts_line(source, previous.start)
                })
        });
        match groups.last_mut() {
            Some(group) if joins => {
                group.comments.push(range.clone());
                if name.is_some() {
                    group.name.clone_from(name);
                }
            }
            _ => groups.push(Group {
                marker: *marker,
                comments: vec![range.clone()],
                name: name.clone(),
            }),
        }
    }
    groups
}

/// Whether only whitespace sits before `offset` on its line.
fn starts_line(source: &str, offset: usize) -> bool {
    source
        .get(..offset)
        .and_then(|before| before.rsplit('\n').next())
        .is_some_and(|line| line.trim().is_empty())
}

/// The `raw` root of one group, the comment node over the Markdown of its text,
/// and the file ranges of the text that it marks as no prose.
fn root(
    dialect: Dialect,
    source: &str,
    group: &Group,
    blocks: &[&Pattern],
    tokens: &[&Pattern],
) -> Result<(Node, Vec<Range<usize>>), Error> {
    let stripped = strip(
        source,
        &lines(source, group),
        dialect.keeps_indent() && group.marker.is_line(),
    );
    let masks = masks(dialect, &stripped.text, group.name.as_deref())
        .into_iter()
        .filter_map(|mask| {
            let lines = stripped.segments(mask);
            Some(lines.first()?.start..lines.last()?.end)
        })
        .collect();
    let segments = if dialect.has_jsdoc() && group.marker == Marker::DocBlock {
        jsdoc::prose(&stripped.text)
    } else {
        iter::once(0..stripped.text.len()).collect()
    };
    let mut children = Vec::new();
    for segment in segments {
        let text = stripped.text.get(segment.clone()).unwrap_or_default();
        if blocks.is_empty() && tokens.is_empty() {
            let mut nodes = markdown::blocks(text, Flavor::Comment)?;
            for node in &mut nodes {
                node.map_segments(&|span| {
                    stripped.segments(span.start + segment.start..span.end + segment.start)
                });
            }
            children.extend(nodes);
        } else {
            let changed = ignore::rewrite(text, blocks, tokens).map_err(Error::Ignore)?;
            let mut nodes = markdown::blocks(changed.text(), Flavor::Comment)?;
            remap_nodes(
                &mut nodes,
                &|span| {
                    changed
                        .origin(span)
                        .into_iter()
                        .flat_map(|place| {
                            stripped
                                .segments(place.start + segment.start..place.end + segment.start)
                        })
                        .collect()
                },
                &Some,
            );
            children.extend(nodes);
        }
    }
    let start = group.comments.first().map_or(0, |comment| comment.start);
    let end = group.comments.last().map_or(0, |comment| comment.end);
    let comment = Node {
        scope: group.marker.scope(),
        span: start..end,
        segments: group.comments.clone(),
        reads: Vec::new(),
        children,
        first: false,
        closes: end,
        flushes: Vec::new(),
        label: None,
        language: None,
    };
    let node = Node {
        scope: Scope::Raw,
        span: start..end,
        segments: group.comments.clone(),
        reads: Vec::new(),
        children: vec![comment],
        first: false,
        closes: end,
        flushes: Vec::new(),
        label: None,
        language: None,
    };
    Ok((node, masks))
}

/// The byte ranges of the text of a doc comment that hold no prose.
///
/// A Go doc comment opens with the name of its declaration, and links to
/// others in brackets, such as `[Name]` and `[pkg.Name]`.
///
/// Rustdoc links to an item in brackets too, such as `[Name]`,
/// `[crate::Name]` and ``[`Name`]``.
/// `JSDoc` links in an inline tag, such as `{@link Name}`.
///
/// A bracket is a link unless a `(`, a `[` or a `:` follows it, which make it
/// a link with a target of its own, or a link definition.
fn masks(dialect: Dialect, text: &str, name: Option<&str>) -> Vec<Range<usize>> {
    let link = match dialect {
        Dialect::Go => &GO_LINK,
        Dialect::Rust => &RUST_LINK,
        Dialect::TypeScript | Dialect::Tsx | Dialect::JavaScript => &JSDOC_LINK,
        Dialect::Swift | Dialect::Yaml | Dialect::Toml | Dialect::Shell => return Vec::new(),
    };
    let mut masks = Vec::new();
    if let Some(name) = name {
        let at = text.len() - text.trim_start_matches(' ').len();
        if let Some(after) = text.get(at..).and_then(|rest| rest.strip_prefix(name))
            && !after.starts_with(|next: char| next.is_alphanumeric() || next == '_')
        {
            masks.push(at..at + name.len());
        }
    }
    masks.extend(
        link.captures_iter(text)
            .filter_map(|link| link.get(1).map(|bracket| bracket.range())),
    );
    masks
}

#[expect(
    clippy::expect_used,
    reason = "The patterns are constants that compile."
)]
static GO_LINK: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(\[\*?[A-Za-z_][A-Za-z0-9_.]*\])(?:[^(\[:]|$)")
        .expect("the Go link pattern compiles")
});

#[expect(
    clippy::expect_used,
    reason = "The patterns are constants that compile."
)]
static RUST_LINK: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(\[`[^`\n]+`\]|\[[A-Za-z_][A-Za-z0-9_:]*\])(?:[^(\[:]|$)")
        .expect("the Rust link pattern compiles")
});

/// The file bytes of each line of comment text, with the markers and the `*` of
/// each line removed.
///
/// A plain comment also loses the run of `/`, `#` or `*` that follows its
/// opener, so a banner such as `//////`, `######` or `/*****` leaves no
/// punctuation in the text.
fn lines(source: &str, group: &Group) -> Vec<Range<usize>> {
    if group.marker.is_line() {
        return group
            .comments
            .iter()
            .map(|comment| {
                let start = comment.start + group.marker.opener();
                let after = source.get(start..comment.end).unwrap_or_default();
                let banner = match group.marker {
                    Marker::Line => leading_run(after, '/'),
                    Marker::Hash => leading_run(after, '#'),
                    Marker::OuterDocLine
                    | Marker::InnerDocLine
                    | Marker::Block
                    | Marker::DocBlock => 0,
                };
                start + banner..comment.end
            })
            .map(|line| trim_carriage_return(source, line))
            .collect();
    }
    let Some(comment) = group.comments.first() else {
        return Vec::new();
    };
    let mut start = comment.start + group.marker.opener();
    let text = source.get(comment.clone()).unwrap_or_default();
    let mut end = if text.len() >= group.marker.opener() + 2 && text.ends_with("*/") {
        comment.end - 2
    } else {
        comment.end
    };
    if group.marker == Marker::Block {
        start += leading_run(source.get(start..end).unwrap_or_default(), '*');
        let body = source.get(start..end).unwrap_or_default();
        end -= body.len() - body.trim_end_matches('*').len();
    }
    let mut lines = Vec::new();
    let mut at = start;
    for line in source.get(start..end).unwrap_or_default().split('\n') {
        lines.push(trim_carriage_return(source, at..at + line.len()));
        at += line.len() + 1;
    }
    // rustdoc removes the `*` that starts the lines of a comment only when
    // every line has one.
    let starred = lines.iter().skip(1).all(|line| {
        let text = source.get(line.clone()).unwrap_or_default().trim_start();
        text.is_empty() || text.starts_with('*')
    });
    if starred {
        for line in lines.iter_mut().skip(1) {
            let text = source.get(line.clone()).unwrap_or_default();
            if text.trim_start().starts_with('*') {
                line.start += text.len() - text.trim_start().len() + 1;
            }
        }
    }
    lines
}

/// The bytes of the run of `c` that starts `text`.
fn leading_run(text: &str, c: char) -> usize {
    text.len() - text.trim_start_matches(c).len()
}

fn trim_carriage_return(source: &str, line: Range<usize>) -> Range<usize> {
    let text = source.get(line.clone()).unwrap_or_default();
    line.start..line.start + text.trim_end_matches('\r').len()
}

/// Comment text with markers stripped, and the file byte each of its bytes came
/// from.
struct Stripped {
    text: String,
    /// `map[i]` is the file byte of `text` byte `i`; one more entry ends the
    /// last line.
    map: Vec<usize>,
}

impl Stripped {
    /// The file ranges of a span of `text`, one for each line of it.
    ///
    /// A line with no text gives an empty range, so the ranges joined by a line
    /// feed read as the span.
    fn segments(&self, span: Range<usize>) -> Vec<Range<usize>> {
        let at = |index: usize| self.map.get(index).copied().unwrap_or_default();
        let mut start = span.start;
        self.text
            .get(span)
            .unwrap_or_default()
            .split('\n')
            .map(|line| {
                let end = start + line.len();
                let first = at(start);
                let last = if line.is_empty() {
                    first
                } else {
                    at(end - 1) + 1
                };
                start = end + 1;
                first..last
            })
            .collect()
    }
}

/// Joins `lines` into one text and removes the indentation they all share, as
/// rustdoc does, or with `one_space` only the first space of each line.
fn strip(source: &str, lines: &[Range<usize>], one_space: bool) -> Stripped {
    let text_of = |line: &Range<usize>| source.get(line.clone()).unwrap_or_default();
    let indent = lines
        .iter()
        .map(text_of)
        .filter(|text| !text.trim().is_empty())
        .map(|text| text.len() - text.trim_start_matches([' ', '\t']).len())
        .min()
        .unwrap_or_default();
    let mut text = String::new();
    let mut map = Vec::new();
    let mut previous_end = lines.first().map_or(0, |line| line.start);
    for (index, line) in lines.iter().enumerate() {
        if index > 0 {
            text.push('\n');
            map.push(previous_end);
        }
        let start = if text_of(line).trim().is_empty() {
            line.end
        } else if one_space {
            line.start + usize::from(text_of(line).starts_with(' '))
        } else {
            line.start + indent
        };
        // The padding before a closing `*/` is not text.
        let end = if index + 1 == lines.len() {
            start
                + source
                    .get(start..line.end)
                    .unwrap_or_default()
                    .trim_end()
                    .len()
        } else {
            line.end
        };
        text.push_str(source.get(start..end).unwrap_or_default());
        map.extend(start..end);
        previous_end = end;
    }
    map.push(previous_end);
    Stripped { text, map }
}

#[expect(
    clippy::expect_used,
    reason = "The patterns are constants that compile."
)]
static JSDOC_LINK: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(\{@(?:link|linkcode|linkplain|tutorial)\b[^}]*\})")
        .expect("the JSDoc link pattern compiles")
});
