# Design

## Context

The motivation is in [proposal.md][proposal].  
Vale splits each paragraph, heading and table cell with prose's `segment` package ([segment.go][p-segment]).  
That package runs neurosnap's Go port of Punkt ([sentences][neurosnap]) with NLTK's English model, converted to JSON.

The model holds four tables, learned without labels from about 469,000 tokens of 1980s Wall Street Journal text:

| Table | Entries | Example |
|:--|:--|:--|
| Abbreviations | 156 | `calif`, `corp`, `dr` |
| Pairs a period does not split | 37 | `b smith` |
| Words that start sentences | 39 | `but`, `yet` |
| How a word was capitalized, at a sentence start and inside one | 20,366 | `crickets` |

The last table is 95% of the 403 KB file.  
No license covers the model ([nltk_data#188][nltk-188]).

That model scores 90.8 F1 on UD English-EWT ([SaT paper][sat]).  
On the Golden Rules, a list of hard cases, it scores 56% to 72% ([pySBD paper][pysbd]).

## Goals / Non-Goals

**Goals:**

- Split at least as well as Vale on every evaluation set, and better on the cases Punkt is known to miss.
- Linear time in the text, well under a millisecond per paragraph, with no runtime beyond the binary.
- A model hakkiri may ship, rebuilt by one committed command.

**Non-Goals:**

- Languages other than English.
- A neural splitter: each needs an ML runtime ([SaT paper][sat]).

## Decisions

### 1. Own Punkt, from the paper

hakkiri implements Punkt as [Kiss and Strunk][punkt] describe it, with NLTK's code as a reference ([punkt.py][nltk-punkt], Apache-2.0).  
The splitter marks each `.`, `?` and `!` that may end a sentence, then decides with the model's four tables (operator, 2026-11-10).

Alternatives:

- `sentencex`: 34 crates enter the build, `clap` among them, and it agreed with Vale on 91.3% of our measured cases. Lost: the weight, and rules we cannot train.
- A port of Vale's Go code and model. Lost: the model has no license.
- `punkt`, a Rust port with no release since 2019, and `nupunkt-rs`, whose model is trained on legal text and UD treebanks. Lost: an old base and a model we cannot ship.

### 2. Override rules after Punkt

A small set of rules runs after Punkt:

- A closing bracket or quote after the end mark stays with the sentence.
- No sentence ends inside brackets that close later in the same node.
- A token with no space inside, such as `v1.2.3`, `README.md` or `3.50`, never holds a boundary.
- An ellipsis followed by a lower-case word does not end a sentence.
- A number or a letter with a period, at the start of a sentence or after a colon, numbers a list item and ends nothing: `1. Install it.`

These are the failure classes the [pySBD paper][pysbd] reports for Punkt.

Two more changes sit inside the Punkt step: a run of initials, such as `J. R. R. Tolkien`, goes on, and a run of lone periods, such as `. . .`, reads as one ellipsis.

Alternative: Punkt alone. Lost: the Golden Rules score stays near Vale's.

### 3. Training text

The model is trained on four sources (operator, 2026-11-10):

- The Markdown of the owner's public repositories: public, neither a fork nor archived, at most 10 MB, each pinned by commit.
- The Markdown files of Common Pile's `stackv2`, kept where the record's `path` ends in `.md` or `.markdown` and every license in its `metadata.license` is on the Blue Oak list ([dataset card][stackv2], [Common Pile][common-pile]).  
  Markdown spreads over many of its 764 shards, mixed with code; the task reads a pinned prefix of one shard that is dense with it.
- The Python Enhancement Proposals of Common Pile's `python_enhancement_proposals`, in the public domain ([dataset card][peps]).
- Public-domain books from Common Pile's `project_gutenberg`: a pinned prefix of one shard.

The documentation sources teach terms such as `e.g.` and `etc.` as hakkiri's input uses them; the books give volume.  
A source list with each source's license is committed beside the model, in `data/splitter/SOURCES.md`.  
Every dataset is pinned by revision, so a rerun reads the same input.

Alternatives:

- Books only. Lost: abbreviations skew literary.
- The owner's Markdown only. Lost: near the 100,000-word minimum that [nupunkt's guide][nupunkt-guide] gives, so rare abbreviations go unlearned.
- Wikipedia or Stack Exchange. Lost: CC BY-SA.

### 4. Training runs in a build box, the model is a file

The trainer is three CPU passes over the text, in the library module `splitter::train`: it counts tokens, marks candidate sentence ends, and runs Punkt's likelihood tests.  
The example `examples/train_splitter.rs` reads the corpus files and writes the model and `data/splitter/SOURCES.md`.  
`mise run splitter:train` runs it after `mise run splitter:fetch`, which reads the pins in `data/splitter/` and writes the corpus to `target/splitter-corpus/`.  
`mise run splitter:pin` refreshes the repository pins.  
The tasks run in an ephemeral build box.  
The capitalization table keeps the 20,000 most frequent words; a word outside it reads as unknown.  
The model is a tab-separated text file, so a review reads its changes as lines.  
The binary embeds it with `include_str!` and parses it once per process.

Alternative: generate Rust source with the tables. Lost: a large generated file, slower to build.

### 5. Evaluation against gold boundaries instead of Vale

The committed evaluation set holds:

- Paragraphs from the owner's public repositories, labelled by hand, including the 89 hard cases measured for this choice.
- The Golden Rules list, under the MIT license of [pySBD][pysbd-repo].
- A sample of [MASC][masc], which carries no license limits.

Vale's F1 on each set is measured once and committed.  
A test computes hakkiri's F1 and fails when it is lower on any set.

UD English-EWT and GUM are measured too, only in the design evidence.  
They are CC BY-SA and CC BY-NC-SA, so nothing from them is committed.

Alternative: agreement with Vale as the score. Lost: Vale's own mistakes would count as right.

## Risks / Trade-offs

- [hakkiri splits some text apart from Vale] → `LongSentence` and Flesch-Kincaid move on those inputs. The rule-engine change states its tolerance, and `HKR-008` reviews the alert diff.
- [A file in `stackv2` carries a wrong license field] → the model holds counts, not text, and the source list names the filter.
- [Hand labels carry mistakes] → each label change is a reviewed diff, and the Golden Rules and MASC sets are labelled by others.

[proposal]: proposal.md
[p-segment]: https://github.com/jdkato/prose/blob/b33e60dbff0492289b227f3681120ac057d8131b/segment/segment.go#L58-L80
[neurosnap]: https://github.com/neurosnap/sentences
[nltk-188]: https://github.com/nltk/nltk_data/issues/188
[nltk-punkt]: https://github.com/nltk/nltk/blob/375d5002d9c36f7776acb9057f012a8dafa60622/nltk/tokenize/punkt.py
[punkt]: https://aclanthology.org/J06-4003.pdf
[sat]: https://arxiv.org/abs/2406.16678
[pysbd]: https://aclanthology.org/2020.nlposs-1.15.pdf
[pysbd-repo]: https://github.com/nipunsadvilkar/pySBD
[common-pile]: https://arxiv.org/abs/2506.05209
[stackv2]: https://huggingface.co/datasets/common-pile/stackv2
[peps]: https://huggingface.co/datasets/common-pile/python_enhancement_proposals
[nupunkt-guide]: https://github.com/alea-institute/nupunkt/blob/1e0b9d9ab77e7ce7f8473e7cce7e6bf4723a4583/docs/training-guide.md
[masc]: https://anc.org/data/masc/about/
