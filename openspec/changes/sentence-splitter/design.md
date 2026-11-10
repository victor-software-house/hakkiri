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

These are the failure classes the [pySBD paper][pysbd] reports for Punkt.

Alternative: Punkt alone. Lost: the Golden Rules score stays near Vale's.

### 3. Training text

The model is trained on three sources (operator, 2026-11-10):

- The Markdown of the owner's public repositories.
- The `.md` files of Common Pile's `github_archive`, kept only where the file's license is on the Blue Oak list ([Common Pile][common-pile]).
- Public-domain books from Common Pile's `project_gutenberg`.

The documentation sources teach terms such as `e.g.` and `etc.` as hakkiri's input uses them; the books give volume.  
A source list with each source's license is committed beside the model.

Alternatives:

- Books only. Lost: abbreviations skew literary.
- The owner's Markdown only. Lost: near the 100,000-word minimum that [nupunkt's guide][nupunkt-guide] gives, so rare abbreviations go unlearned.
- Wikipedia or Stack Exchange. Lost: CC BY-SA.

### 4. Training runs in a build box, the model is a file

The trainer is one CPU pass that counts tokens and runs Punkt's likelihood tests.  
A mise task fetches the sources, trains and writes `data/splitter/en.tsv` in an ephemeral build box.  
The model is a tab-separated text file, so a review reads its changes as lines.  
The binary embeds it with `include_str!` and parses it once per process.

Alternative: generate Rust source with the tables. Lost: a large generated file, slower to build.

### 5. Evaluation against gold boundaries, not against Vale

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
- [A file in `github_archive` carries a wrong license field] → the model holds counts, not text, and the source list names the filter.
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
[nupunkt-guide]: https://github.com/alea-institute/nupunkt/blob/main/docs/training-guide.md
[masc]: https://anc.org/data/masc/about/
