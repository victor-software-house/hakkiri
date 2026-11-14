# Design

## Context

hakkiri reads Markdown with markdown-rs and cuts the syntax that is not prose out of the source text.
Vale reads the HTML that goldmark writes from the same file, and finds each word of that HTML again in the file.
The first reads constructs as the CommonMark and GFM specs define them.
The second also keeps what its converter and its search for a word do to a construct.

## Goals / Non-Goals

Goals:

- Measure where the two readings differ, over every construct the repositories use.
- Make hakkiri give each alert that Vale gives, and list each other difference with its cause.

Non-goals:

- MDX and front matter, which other changes probed.
- Reading `img` text in inline HTML, which a known gap lists.

## Decisions

### 1. One input holds one construct, with words no other line uses

Each input is a file of its own, and each word in it is spelt once, except a few short ones.
Vale places a repeated word at another copy of it in some nested blocks, and a probe with natural prose showed rows that came from that and not from a construct.
The probe style has a rule for each scope a construct reaches, and one rule that matches each run of non-space characters, so that a joined or split word shows.
Alternative: lint the repositories' own documents.
It lost because each document repeats words, and a difference could not be traced to one construct.

### 2. A link keeps the address for the `link` scope

Vale masks the destination of a link out of the text where it first stands, so a link whose text is its destination gives the `link` scope no word.
That holds for an autolink, for a bare URL that goldmark links, and for `[https://a.example](https://a.example)`.
hakkiri keeps that address in the `link` node and keeps it a gap in the text around it, as it did before the probe.
ProseStyle v0.0.16's `BareURL` rule reads the `link` scope to find a bare URL without lookbehinds, so following Vale here would leave the rule with no text to flag.
goldmark links a bare address that starts with a lower-case `http://`, `https://`, `ftp://` or `www.` and an email address, and markdown-rs also links `HTTP://` and `mailto:` text, which hakkiri reads as text.
An email address, a `www.` address and `<me@a.example>` keep their text, since the destination the converter writes, `mailto:me@a.example`, is not their text.
The `vale-differences` spec lists the kept address.

### 3. An `ftp://` address is a gap

markdown-rs links no `ftp://` address, and goldmark does, so the words of one were prose.
The other forms the probe covered, with punctuation at the end, in parentheses, with underscores and with a query, agree.
hakkiri finds an `ftp://` address with goldmark's pattern and trims its end as goldmark does.
Alternative: read bare addresses with a port of the whole of goldmark's linkify parser.
It lost because the probe found no other difference to pay for it.

### 4. A backslash escape loses the backslash

markdown-rs reads `\*` as `*`, and hakkiri kept the backslash in the text, so a word held it: `well\-known`.
Vale reads the character, cannot find a word that holds it in the file, and gives no alert for that word.
hakkiri cuts the backslash, as it cuts any other marker, so a rule sees `well-known` and the alert lands on its place in the file.
A hard break written with a backslash loses it the same way.
Alternative: keep the text as written.
It lost because a rule then matched a backslash that no reader sees.

### 5. A table row has the cells of the header

The GFM spec ignores a cell past the last of the header, and goldmark does.
markdown-rs lists it, so its words were prose.
hakkiri cuts a cell past the header's last one.
Alternative: keep the cell.
It lost because the spec and Vale both leave it out.

### 6. A footnote reads as in Vale's HTML

Vale renders a reference as a number that it places at the first matching digit of the line, and a definition as a list item at the end of the document, for each definition that a reference names.
hakkiri reads a reference as a gap, since the number is no prose, a definition that no reference names as a gap, and one that a reference names as a `list` node where it stands.
Alternative: read each reference as its number.
It lost because Vale gives that number an alert at a place that is not the reference, so no reading could match it.
The order of the definitions does not change an alert, so hakkiri keeps them where they stand.

### 7. An inline HTML element has the scope of the tag

Vale scopes `b` and `strong` as `strong`, `i` and `em` as `emphasis`, `code` and `tt` as `code`, and `a` as `link`, whether the converter or the file wrote the tag.
markdown-rs gives one HTML node for each tag, so hakkiri pairs an opening tag with the closing tag among the same siblings and reads the nodes between them as the element.
A tag with no closing tag among its siblings is a gap, as it was.
Alternative: find the pairs in the HTML of the paragraph.
It lost because the paragraph's HTML is not parsed anywhere in the tree builder.

### 8. What hakkiri keeps from the earlier reading

The probe found nine places where Vale's own reading loses or invents text.
The `vale-differences` spec lists them, each with the input, so that none is read as a gap to close.

### 9. Known gap: the alt text of an inline `img`

Vale reads the alt text of an `img` tag in a line of text as prose, and not as part of a link around it.
hakkiri cuts the tag whole.
Reading it needs the alt text outside the `link` node and inside the paragraph, which the tag cuts do not give yet.
Row `HKR-075` takes it.
