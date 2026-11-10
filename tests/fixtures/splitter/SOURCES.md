# Splitter evaluation sets

Four sets of paragraphs with hand-checked sentence boundaries, for the [sentence splitter][change].  
Vale's score on each set is in [`vale-f1.tsv`](vale-f1.tsv).

## Format

A set is one `.fixture` file of records.  
A record starts with one `# <source>` line, then one sentence per line, then a blank line.  
The text of a record is its sentences joined by one space.  
The gold boundaries are the end of every sentence but the last, as byte offsets in that text.

Inline code spans are replaced by the one-letter word `x`, as hakkiri's engine reads them.  
Links keep their text, and emphasis marks are gone.

## Sets

| Set | Records | Sentences | Source | License |
|:--|--:|--:|:--|:--|
| [`docs.fixture`](docs.fixture) | 298 | 905 | Markdown of public repositories of the owner | The owner's own text, each repository's `LICENSE` where it has one |
| [`hard.fixture`](hard.fixture) | 88 | 165 | Written for this test | The owner's own text |
| [`golden-rules.fixture`](golden-rules.fixture) | 48 | 80 | The English Golden Rules, as pySBD ships them | MIT, see [`golden-rules.LICENSE`](golden-rules.LICENSE) |
| [`masc.fixture`](masc.fixture) | 71 | 284 | MASC 3.0.0 | No license or other restrictions |

### docs

- Paragraphs and list items from the `.md` and `.mdx` files of 22 public repositories under `victor-software-house`.
- The `# <source>` line names the repository and the file.
- Each repository was checked as public before use.
- A repository whose `LICENSE` names another copyright holder is left out, and so is every fork.
- A seeded draw picked the paragraphs, with a cap per repository, from those of 40 to 600 characters.
- Paragraphs with a private repository name, a local path, an email address or mostly code were left out.
- A plain draft of the boundaries was corrected paragraph by paragraph.  
  Vale's output was not used.
- Where a sentence starts in lower case, such as a command or a crate name, the boundary still counts.

### hard

- Short cases that Punkt is known to miss.
- They cover abbreviations, initials, versions, file names, decimals, ellipses, quotes, brackets and list numbers.
- A case with two spaces after a full stop is left out, since a record joins sentences with one space.

### golden-rules

- 48 of the 52 English Golden Rules of [pragmatic_segmenter][pragmatic], as [pySBD][pysbd] lists them in `tests/lang/test_english.py` (commit `5905f13b`).
- The `# <source>` line gives the rule's position in that list.
- Three rules with a line break inside a sentence are not in pySBD's list.  
  Nor is the rule with no space after a full stop.
- Whitespace is collapsed to single spaces.
- Both projects are under the MIT license.

### masc

- Four consecutive sentences from each of 71 written texts of [MASC][masc] 3.0.0.
- The archive is `MASC-3.0.0.zip`, with SHA-256 `0b300d96016233c7c79b43559b477a6bb0ed16bb416d5c517eccb108a6228493`.
- The gold sentences are the corpus's own `-s.xml` regions.  
  The corpus project made them with GATE's ANNIE splitter, a rule-based splitter, and then validated them by hand.  
  They may favour rule-based splitters.
- The MASC site states that MASC is distributed without license or other restrictions.
- A seeded draw took up to seven texts from each written genre.
- The `wsj` folder of the newspaper genre, the `spam` folder of the email genre, spoken data and Twitter are not used.
- A window is skipped when a sentence has under four words, a non-ASCII symbol, or no `.`, `?` or `!` before its closing quotes.
- A window is also skipped when two of its sentence regions overlap or leave text between them.
- A record with a phone number is skipped.
- Records with a misspelling that `lint:typos` flags are left out, since the corpus text is verbatim.

## Vale's score

- [`vale-f1.tsv`](vale-f1.tsv) holds Vale 3.26.0's boundary F1 on each set.
- Vale v3.26.0 [pins][vale-gomod] prose v3.2.2 and sentences.v1 v1.0.7, and [calls][vale-prose] `segment.New()`.  
  The table uses those versions.
- Vale at [66bbb85][vale-main], which the designs read, pins prose `b33e60d`.  
  The four sets score the same on both pins.
- The measuring program is not committed.  
  [tasks.md][tasks] describes it, with the F1 definition.

## Not committed

UD English-EWT and GUM are measured only for the design evidence, and nothing from them is stored here.

[change]: ../../../openspec/changes/sentence-splitter/proposal.md
[pragmatic]: https://github.com/diasks2/pragmatic_segmenter
[pysbd]: https://github.com/nipunsadvilkar/pySBD
[masc]: https://anc.org/data/masc/about/
[vale-gomod]: https://github.com/errata-ai/vale/blob/v3.26.0/go.mod
[vale-prose]: https://github.com/errata-ai/vale/blob/v3.26.0/internal/nlp/prose.go
[vale-main]: https://github.com/errata-ai/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/go.mod
[tasks]: ../../../openspec/changes/sentence-splitter/tasks.md
