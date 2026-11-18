## REMOVED Requirements

### Requirement: The Markdown parser is a known gap

**Reason**: The probe measured the differences, so the cases are known.

**Migration**: The ADDED section has two requirements to read.  
They are "Markdown reads differ from Vale's converter in these ways" and "The alt text of an inline img is a known gap".

### Requirement: HTML-block text differs from Vale in detail

**Reason**: An inline HTML element now has the scope Vale gives its tag.  
So the scenario "Inline HTML in a paragraph has no element scope" is no longer true.  
OpenSpec cannot drop one scenario of a modified requirement.  
So the requirement is removed, and added again below without that scenario.

**Migration**: Read the ADDED requirement "HTML-block text differs from Vale in these details", which has the same body.  
Its other scenarios are unchanged.

## ADDED Requirements

### Requirement: HTML-block text differs from Vale in these details

hakkiri SHALL read the text of an HTML block, and the text around inline HTML, as each scenario below shows.  
Vale 3.26.0 reads it in another way.

#### Scenario: A reference is unescaped once

- **WHEN** a paragraph contains `&amp;lt;` or `&amp;#32;`, or a code span contains `&amp;`
- **THEN** hakkiri reads `&lt;`, `&#32;` and `&amp;`.  
  Vale unescapes the text of each token twice, so it reads `<`, a space and `&` ([html 5][html-5])

#### Scenario: Vale loses a match beside a reference

- **WHEN** a text contains a reference that Vale cannot find in the file once it is unescaped, and a rule matches a word of it.  
  Two examples are `&#109;arker` and a final `&nbsp;`.
- **THEN** hakkiri reports the match through its byte table.  
  Vale finds the words of the text in the file, so it gives no alert for a word it cannot find ([html 5][html-5])
- **AND** Vale places another alert on the wrong line or column

#### Scenario: Vale loses the text around a blank alt

- **WHEN** a paragraph contains `one marker <img src="x" alt=" "> end`
- **THEN** hakkiri reads the paragraph as `one marker end`, and Vale gives no alert for that text.  
  Vale does give one when the alt text contains a word ([html 6][html-6])

#### Scenario: A code element is a code span

- **WHEN** an HTML block contains `a <code>x</code> b`
- **THEN** hakkiri reads a code span of one word, and reports no match over it.  
  Vale masks the text with asterisks, and reports a match over them ([html 3][html-3])

#### Scenario: A block of masked text only

- **WHEN** an HTML block contains `<p><kbd>x</kbd></p>`
- **THEN** hakkiri reads `*`, and Vale reports nothing, since it cannot place text that is not in the file ([html 3][html-3])

#### Scenario: A script that never ends

- **WHEN** an HTML block contains `<script><!-- <script </script> marker`, and the next block contains `last marker`
- **THEN** hakkiri reads the next block, since it tokenizes each HTML block alone.  
  Vale tokenizes the whole file as one stream, so the open script hides the rest of it ([html 1][html-1])

### Requirement: Markdown reads differ from Vale's converter in these ways

hakkiri SHALL read Markdown as each scenario below shows, where Vale 3.26.0 reads it in another way.

#### Scenario: An inline tag between letters

- **WHEN** a paragraph contains `one*two*three`, `one~~two~~three` or `one[two](u)three`
- **THEN** hakkiri reads the word `onetwothree`, and Vale reads `one` and `twothree`.  
  Its converter puts a space before the text of an opening tag, and none after a closing tag ([markdown-probe 8][markdown-probe-8])

#### Scenario: A word contains an escape

- **WHEN** a paragraph contains `well\-known` or `snake\_case`
- **THEN** hakkiri reads `well-known` and `snake_case`, and alerts where they stand.  
  Vale cannot find the converted word in the file, so it gives no alert for it ([markdown-probe 4][markdown-probe-4])

#### Scenario: A paragraph line starts with a number

- **WHEN** a line that continues a paragraph starts with `1986.` and a space
- **THEN** hakkiri reads `1986` as a word of the paragraph.  
  Vale masks every line start that has digits and a dot, so it reads no such word ([markdown-probe 8][markdown-probe-8])

#### Scenario: A reference that nothing defines

- **WHEN** a paragraph contains `[a label][nope]` and no definition has the label `nope`
- **THEN** hakkiri reads the text as written, so `nope` is a word.  
  Vale masks the second label, as it does for a defined reference ([markdown-probe 8][markdown-probe-8])

#### Scenario: A footnote reference

- **WHEN** a paragraph contains `A word[^one] here.`
- **THEN** hakkiri reads `A word here.`  
  Vale reads the number `1` as a word of a link, and places it at the first `1` of the line ([markdown-probe 6][markdown-probe-6])

#### Scenario: The last word of a footnote definition

- **WHEN** a definition ends with `the last.` and a rule matches each run of non-space characters
- **THEN** hakkiri reports `last.`, and Vale gives no alert.  
  Its text has the back reference arrow after the full stop, so the word with the arrow is not in the file ([markdown-probe 6][markdown-probe-6])

#### Scenario: A tag that is never closed

- **WHEN** a paragraph contains `Text <b>one open.` and the next contains `two </b>stray.`
- **THEN** hakkiri scopes no text of the second paragraph as strong, and Vale scopes `two` as strong ([markdown-probe 8][markdown-probe-8])

#### Scenario: An unclosed tag that swallows another

- **WHEN** a paragraph contains `Text <unclosed one tag and <b>two bold.` and the next contains `three </b>stray.`
- **THEN** hakkiri finishes the file, and Vale panics with a slice out of range ([markdown-probe 8][markdown-probe-8])

#### Scenario: A link that is its own address

- **WHEN** a paragraph contains `Read <https://b.example> first.`, `Read https://b.example first.` or `See [https://b.example](https://b.example) now.`
- **THEN** hakkiri's `link` scope reads `https://b.example`, so ProseStyle's `BareURL` can flag it, and Vale's reads no word ([markdown-probe 2][markdown-probe-2])

### Requirement: The alt text of an inline img is a known gap

hakkiri SHALL differ from Vale 3.26.0 as the scenario below shows, until the queue row in it closes.

#### Scenario: An img tag in a line of text

- **WHEN** a paragraph contains `Build <a href="u"><img src="b.svg" alt="the badge"></a> now.`
- **THEN** hakkiri reads no word of `the badge`.  
  Vale reads them as words of the paragraph, and not of the link (row [HKR-027][ledger], [markdown-probe 9][markdown-probe-9])

[html-1]: ../../../2026-11-10-html-block-text/design.md#1-one-walk-follows-vales
[html-3]: ../../../2026-11-10-html-block-text/design.md#3-masked-text-reads-as-asterisks
[html-5]: ../../../2026-11-10-html-block-text/design.md#5-a-character-reference-reads-once
[html-6]: ../../../2026-11-10-html-block-text/design.md#6-captions-data-and-descriptions-have-scopes
[ledger]: ../../../../../../tasks.yaml
[markdown-probe-2]: ../../design.md#2-a-link-keeps-the-address-for-the-link-scope
[markdown-probe-4]: ../../design.md#4-a-backslash-escape-loses-the-backslash
[markdown-probe-6]: ../../design.md#6-a-footnote-reads-as-in-vales-html
[markdown-probe-8]: ../../design.md#8-what-hakkiri-keeps-from-the-earlier-reading
[markdown-probe-9]: ../../design.md#9-known-gap-the-alt-text-of-an-inline-img
