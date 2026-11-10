# ProseStyle in the format

This file proves the format holds the package.  
It covers the 25 rules in [ProseStyle][prose-style-rules] and the habit candidates in the [frequency research][habit-research].  
No rule below uses a lookbehind to find code, a link, a fence or a comment; a scope holds that.  
A lookbehind that checks only the next character, such as the word before a dash, stays.

## 1. The 25 rules

Fields keep Vale's names (decision 2), so 20 rules carry over as they are.  
Five rules found structure with `raw` and lookbehind; decisions 8 and 9 make them three scope rules.

| Rule | Check | Scope | In hakkiri |
|:--|:--|:--|:--|
| `BareURL` | existence | `raw` | `scope: link`, section 2.1 |
| `BareURLInComment` | existence | `raw.rs` | merged into `BareURL` |
| `BareWorkItem` | existence | `raw` | `scope: text & ~link`, section 2.2 |
| `BareWorkItemInComment` | existence | `raw.rs` | merged into `BareWorkItem` |
| `ChattyHeadings` | existence | `heading` | as is |
| `Dashes` | existence | none | as is; its lookbehind checks one character |
| `Emoji` | existence | none | as is |
| `Exclamation` | existence | none | as is; its lookbehind checks one character |
| `Filler` | existence | none | as is |
| `FormalWords` | substitution | none | as is |
| `GenericHeaders` | existence | `table.header` | as is |
| `Jargon` | substitution | none | as is |
| `KindAsNoun` | existence | none | as is |
| `LeakedChat` | existence | none | as is |
| `Lingo` | substitution | none | as is |
| `ListItem` | metric | `list` | as is |
| `LongSentence` | occurrence | `sentence` | as is |
| `MadeUpWords` | existence | none | as is |
| `Paragraph` | metric | `paragraph` | as is |
| `Profanity` | existence | none | as is |
| `StaleWords` | existence | none | as is |
| `StockPhrases` | existence | none | as is |
| `StrawMan` | existence | none | as is |
| `TableRow` | existence | `raw` | `scope: table.row`, section 2.3 |
| `ThingsSpeak` | existence | none | as is |

`LongSentence` counts a code span as one word (decision 7), so a sentence with code spans counts more words than in Vale.

## 2. The rewritten rules

### 2.1 BareURL

```yaml
# A bare URL shows the reader an address in place of a name.
# GFM reads a bare URL or an autolink as a link whose text is the URL,
# in files and in comments alike, so a link whose text starts with a URL
# is bare. Link targets, code spans and fenced code are not link text.
extends: existence
message: "Bare URL '%s': put the link on the words that name it."
level: error
scope: link
nonword: true
tokens:
  - '^https?://\S+'
tests:
  - name: flags a bare URL and an autolink
    input: See https://example.com/x here, and <https://auto.example>.
    contains:
      - "'https://example.com/x'"
      - "'https://auto.example'"
  - name: leaves links, definitions and code alone
    input: |
      See [the page](https://ok.example) and `https://code.example`.

      [ref]: https://def.example
    want: ''
  - name: flags a bare URL in a comment and leaves a code span alone
    format: rs
    input: |
      //! Crate docs at https://crate.example here.
      // Run `curl https://span.example/x` first.
    contains:
      - "'https://crate.example'"
    absent:
      - span.example
```

### 2.2 BareWorkItem

Vale's `BareWorkItem` reads fenced code in its `raw` scope, so it is off for this sample.

<!-- vale ProseStyle.BareWorkItem = NO -->

```yaml
# A bare PR, MR, ticket or ledger code names nothing a reader knows.
# Say what it is, and link it. Link text is left out, and a code span
# reads as one word no token matches, so a code in either passes.
# Standard names such as SHA-256 and UTF-8 pass.
extends: existence
message: "'%s' is a bare code: say what it is, and link it."
level: warning
scope: text & ~link
nonword: true
tokens:
  - '(?<![\w/#&-])(?!(?:SHA|UTF|ISO|RFC|CVE|AES|TLS|HTTP|IPv|W)-)(?:#\d{2,5}|![1-9]\d{1,4}|[A-Z][A-Z0-9]{1,6}-\d{2,5})(?![\w-])'
tests:
  - name: flags PR, ticket and MR codes
    input: 'See PR #123, EVO-9673, and !294 for details.'
    contains:
      - "'#123'"
      - "'EVO-9673'"
      - "'!294'"
  - name: leaves codes in links and code spans, and standard names
    input: Use `QCTL-006`, [PR 40](https://example.com/pull/40), SHA-256 and UTF-8.
    want: ''
  - name: flags a code in a comment
    format: rs
    input: // Fixed in EVO-9673.
    contains: "'EVO-9673'"
```

<!-- vale ProseStyle.BareWorkItem = YES -->

### 2.3 TableRow

```yaml
# A table row over 250 characters of cell text: make it a list or a section.
extends: metric
message: "This table row is over 250 characters: make it a list or a section."
level: warning
scope: table.row
formula: characters
condition: '> 250'
tests:
  - name: leaves a short row alone
    input: |
      | Name | Note |
      |:--|:--|
      | a | short |
    want: ''
```

The long-row case keeps ProseStyle's input, a row of 260 characters, which is too wide to show here.

## 3. The habit rules

Each candidate in the research becomes one rule.  
A rate rule is a metric on `occurrences` per 1,000 words over the whole file (decision 4).  
Its threshold sits between the highest human rate and the agent rate the research measured.  
The prose-style change that adds these rules sets the final numbers.

| Candidate | Rule | Check | Research rate per 1,000 words |
|:--|:--|:--|:--|
| Semicolons | `Semicolons` | metric, `> 3` | agents 8.05, people at most 1.20 |
| Colons inside a sentence | `InnerColons` | metric, `> 5` | agents 9.47, people at most 3.27 |
| ", not the" contrasts | `NotTheContrast` | existence | agents 0.69, people at most 0.12 |
| Three-item lists | `ThreeItemLists` | metric, `> 2` | agents 3.21, people at most 0.93 |
| "so the" and "so it" | `SoThe` | existence | agents 1.54, people at most 0.38 |
| Verbs that make things act | `ThingsAct` | existence | see the research's word table |
| Stock adjectives and adverbs | `StockWords` | existence | see the research's word table |
| Four stock phrases | `StockPhrases` | existence, added tokens | see the research's phrase table |

### 3.1 Semicolons

```yaml
extends: metric
message: "%s semicolons per 1,000 words, over 3: split the sentences."
level: warning
nonword: true
tokens:
  - ';'
formula: occurrences / words * 1000
condition: '> 3'
tests:
  - name: flags a file thick with semicolons
    input: One; two; three four five six seven eight nine ten.
    contains: "200.00 semicolons per 1,000 words"
  - name: leaves a file with few semicolons alone
    input: One two three four five six seven eight nine ten.
    want: ''
```

### 3.2 InnerColons

```yaml
extends: metric
message: "%s colons inside sentences per 1,000 words, over 5: make two sentences or a list."
level: warning
nonword: true
tokens:
  - '\w:\s+[a-z]'
formula: occurrences / words * 1000
condition: '> 5'
tests:
  - name: flags colons inside sentences
    input: 'The fix is plain: it reads the file. The cause is old: it ran twice.'
    contains: ProseStyle.InnerColons
  - name: leaves a label before a list alone
    input: |
      Steps:

      - Read the file.
    want: ''
```

### 3.3 NotTheContrast

```yaml
extends: existence
message: "'%s' sets up a contrast: state what it is."
level: warning
ignorecase: true
tokens:
  - ', not (?:the|a|an)\b'
tests:
  - name: flags the contrast
    input: It reads the file, not the cache.
    contains: "', not the'"
  - name: leaves a plain negative alone
    input: It does not read the cache.
    want: ''
```

### 3.4 ThreeItemLists

```yaml
extends: metric
message: "%s three-item lists per 1,000 words, over 2: name only what matters."
level: suggestion
tokens:
  - '\b\w+, \w+,? (?:and|or) \w+\b'
formula: occurrences / words * 1000
condition: '> 2'
tests:
  - name: flags a file of triples
    input: It reads, parses and writes. It is fast, small and safe.
    contains: ProseStyle.ThreeItemLists
  - name: leaves a pair alone
    input: It reads and writes the file in one pass over its lines.
    want: ''
```

### 3.5 SoThe

```yaml
extends: existence
message: "'%s' joins a cause and a result: make two sentences."
level: warning
ignorecase: true
tokens:
  - ', so (?:the|it)\b'
tests:
  - name: flags the join
    input: The cache was stale, so the test failed.
    contains: "', so the'"
  - name: leaves a sentence opener alone
    input: So the test failed.
    want: ''
```

### 3.6 ThingsAct, StockWords and the phrases

These are word lists, so they are `existence` rules like `Filler`.  
`ThingsAct` lists owns, refuses, survives, carries, stays, waits, keeps, sits and lives after a noun for a thing, as `ThingsSpeak` does for speech.  
`StockWords` lists durable, authoritative, deliberate, silently and verbatim.  
`StockPhrases` gains "byte identical", "byte for byte", "source of truth" and "keeps its own".

[prose-style-rules]: https://github.com/victor-software-house/prose-style/tree/ee9f207279864e305d7372aef4a6c5dacc3eb846/ProseStyle/styles/ProseStyle
[habit-research]: https://github.com/victor-software-house/prose-style/blob/ee9f207279864e305d7372aef4a6c5dacc3eb846/openspec/changes/frequency-habit-rules/research.md
