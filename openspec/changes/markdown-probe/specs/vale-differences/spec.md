## REMOVED Requirements

### Requirement: The Markdown parser is a known gap

**Reason**: The probe measured the differences, so the cases are known.

**Migration**: Read the requirements "Markdown reads differ from Vale's converter in these ways" and "The alt text of an inline img is a known gap" under the ADDED section.

## ADDED Requirements

### Requirement: Markdown reads differ from Vale's converter in these ways

hakkiri SHALL read Markdown as each scenario below says, where Vale 3.26.0 reads it otherwise.

#### Scenario: An inline tag between letters

- **WHEN** a paragraph holds `one*two*three`, `one~~two~~three` or `one[two](u)three`
- **THEN** hakkiri reads the word `onetwothree`, and Vale reads `one` and `twothree`, since its converter puts a space before the text of an opening tag and none after a closing tag ([markdown-probe 8][markdown-probe-8])

#### Scenario: A word holds an escape

- **WHEN** a paragraph holds `well\-known` or `snake\_case`
- **THEN** hakkiri reads `well-known` and `snake_case` and alerts where they stand, and Vale cannot find the converted word in the file, so it gives no alert for it ([markdown-probe 4][markdown-probe-4])

#### Scenario: A paragraph line starts with a number

- **WHEN** a line that continues a paragraph starts with `1986. `
- **THEN** hakkiri reads `1986` as a word of the paragraph, and Vale masks every line start that holds digits and a dot, so it reads no such word ([markdown-probe 8][markdown-probe-8])

#### Scenario: A reference that nothing defines

- **WHEN** a paragraph holds `[a label][nope]` and no definition names `nope`
- **THEN** hakkiri reads the text as written, so `nope` is a word, and Vale masks the second label as it does for a defined reference ([markdown-probe 8][markdown-probe-8])

#### Scenario: A footnote reference

- **WHEN** a paragraph holds `A word[^one] here.`
- **THEN** hakkiri reads `A word here.`, and Vale reads the number `1` as a word of a link and places it at the first `1` of the line ([markdown-probe 6][markdown-probe-6])

#### Scenario: The last word of a footnote definition

- **WHEN** a definition ends with `the last.` and a rule matches each run of non-space characters
- **THEN** hakkiri reports `last.`, and Vale gives no alert, since its text holds the back reference arrow after the full stop and the word with the arrow is not in the file ([markdown-probe 6][markdown-probe-6])

#### Scenario: A tag that is never closed

- **WHEN** a paragraph holds `Text <b>one open.` and the next holds `two </b>stray.`
- **THEN** hakkiri scopes no text of the second paragraph as strong, and Vale scopes `two` as strong ([markdown-probe 8][markdown-probe-8])

#### Scenario: An unclosed tag that swallows another

- **WHEN** a paragraph holds `Text <unclosed one tag and <b>two bold.` and the next holds `three </b>stray.`
- **THEN** hakkiri finishes the file, and Vale panics with a slice out of range ([markdown-probe 8][markdown-probe-8])

#### Scenario: A link that is its own address

- **WHEN** a paragraph holds `Read <https://b.example> first.`, `Read https://b.example first.` or `See [https://b.example](https://b.example) now.`
- **THEN** hakkiri's `link` scope reads `https://b.example`, so ProseStyle's `BareURL` can flag it, and Vale's reads no word ([markdown-probe 2][markdown-probe-2])

### Requirement: The alt text of an inline img is a known gap

hakkiri SHALL differ from Vale 3.26.0 as the scenario below says, until the queue row it names closes.

#### Scenario: An img tag in a line of text

- **WHEN** a paragraph holds `Build <a href="u"><img src="b.svg" alt="the badge"></a> now.`
- **THEN** hakkiri reads no word of `the badge`, and Vale reads them as words of the paragraph and not of the link (row [HKR-027][ledger], [markdown-probe 9][markdown-probe-9])

[ledger]: ../../../../../tasks.yaml
[markdown-probe-2]: ../../design.md#2-a-link-keeps-its-address-for-the-link-scope
[markdown-probe-4]: ../../design.md#4-a-backslash-escape-loses-its-backslash
[markdown-probe-6]: ../../design.md#6-a-footnote-reads-as-vale-renders-it
[markdown-probe-8]: ../../design.md#8-what-hakkiri-keeps-from-the-reading-it-had
[markdown-probe-9]: ../../design.md#9-known-gap-the-alt-text-of-an-inline-img
