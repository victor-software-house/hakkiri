# Design

## Context

The [document-tree design][tree] gives every node a scope and a span.  
A span is one range, so it cannot skip the markers inside a node.  
The text of a paragraph in a block quote, over two line comments, or with an HTML tag in it holds bytes that are syntax.

## Goals / Non-Goals

**Goals:**

- A node reads its text from ranges that hold no syntax.
- One rule for Markdown files and for comments.

**Non-Goals:**

- Sentences and words, which come with word counting in `HKR-004`.
- Scopes and the tree shape, which stay in the document-tree change.

## Decisions

### 1. Segments hold the text, and the span covers them

A segment is a byte range of the file.  
The text of a node is its segments in order.  
A line feed joins two segments with a line break between them.  
One space joins two cells of a table row, and any other gap joins nothing.

The span runs from the start of the first segment to the end of the last.  
An alert takes its position from the span.

Alternative: one span plus a list of ranges to remove. Lost: a reader must apply the removals to get the text, where segments are the text.

### 2. A Markdown line is one segment

Each line of a prose node is one segment, without its line ending.  
The indentation and the `>` markers of the containers around the node stay out of it.  
A line of a paragraph never starts with `>` unless a block quote put it there.  
So every `>` that opens a continuation line is a marker.

Alternative: keep one segment for the whole paragraph. Lost: the markers sit inside it, so the text would hold them.

### 3. Syntax that is not prose is a gap

Fenced code, raw HTML, link definitions, front matter, MDX expressions and the tags of JSX elements are gaps.  
Where one sits inside a prose node, it separates two of its segments, so one paragraph stays one node.

Alternative: split the paragraph into one node per piece of text. Lost: a check that reads a sentence would see it cut in two.

### 4. Containers and headings read as their content

A block quote, list item and table row have the segments of the nodes inside them.  
A heading leaves out its opening `#` run with the space, its closing `#` run, and a setext underline.  
A container with no prose inside it gives no node.

Alternative: keep the markers in the container, because it owns them. Lost: the text of a container would hold syntax, against decision 1.

### 5. Comments strip markers into the same segments

The stripped comment text keeps a map to the file.  
A node of the stripped text maps back through it, one segment for each line.  
A comment marker and the indentation hakkiri strips are gaps.  
Adjacent line comments of one marker join into one text, and a paragraph over them has one segment for each comment line.

Alternative: map only the start and end of a node. Lost: the markers between lines would stay in its text.

## Risks / Trade-offs

- [A tab or a carriage return can sit at a line end] → a segment ends before a carriage return.  
  The common indent counts spaces and tabs.
- [A heading loses its `#`, so its span starts later] → the span is the covering range of the segments, as decision 1 says.

[tree]: ../2026-11-10-document-tree/design.md
