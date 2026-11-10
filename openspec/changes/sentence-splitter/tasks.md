# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.  
Training and measurement run in an ephemeral build box.

## 1. Evaluation sets

- [x] 1.1 Label sentence boundaries by hand in paragraphs from the owner's public repositories, including the 89 measured hard cases.  
  Verify each paragraph's source and license are listed beside the set.
- [x] 1.2 Add the Golden Rules list and a MASC sample with their gold boundaries, after reading each license file.  
  Verify the license of each set is named beside it.
- [x] 1.3 Measure Vale 3.26.0's F1 on each set, and on UD English-EWT and GUM.  
  Commit the numbers for the committed sets.  
  Record the EWT and GUM numbers under this task, with the command that produced them.

### Evidence for 1.1

- 2026-11-10 UTC: [`docs.fixture`][docs-set] holds 298 paragraphs, each headed by its repository and file.
- [`hard.fixture`][hard-set] holds 88 of the 89 cases, written for this test.
- The case with two spaces after a full stop is left out, since a record joins sentences with one space.
- Sources and licenses are in [`SOURCES.md`][sources].
- Not verified: one reader labelled the boundaries so far, and the review of this pull request is the second.

### Evidence for 1.2

- 2026-11-10 UTC: [`golden-rules.fixture`][golden-set] holds 48 rules from [pySBD][pysbd].
- [`masc.fixture`][masc-set] holds 292 sentences in 73 records from [MASC 3.0.0][masc].
- pySBD and pragmatic_segmenter are MIT, with both notices in [`golden-rules.LICENSE`][golden-license].
- The MASC site states that MASC has no license or other restrictions.
- MASC's gold is the corpus's own sentence regions, made with GATE's ANNIE splitter and validated by hand.

### Evidence for 1.3

- 2026-11-10 UTC: [`vale-f1.tsv`][vale-f1] holds Vale's F1 on the four committed sets.
- Vale v3.26.0 [pins][vale-gomod] prose v3.2.2 and sentences.v1 v1.0.7.  
  The committed numbers use these.
- Vale at [66bbb85][vale-main], which the designs read, pins prose `b33e60d`.  
  The four committed sets and EWT score the same on both pins.  
  GUM gives 0.9549 on prose v3.2.2 and 0.9555 on `b33e60d`.
- The measuring program is a small Go program.  
  It is not committed, since it would make the repository declare Go for one measurement.
- The program builds the segmenter with `segment.New()`.  
  It calls `Segment` on the text of each record, as Vale does.
- A record's text is its sentences joined by one space.  
  The gold boundaries are the end offsets of every sentence but the last.  
  The predicted ones are the end offsets of every segment but the last.
- Precision is the correct boundaries over the predicted ones, and recall is the correct ones over the gold ones.  
  F1 is their harmonic mean, summed over a whole set and not averaged per record.
- A UD record is one `# newdoc` or `# newpar` block, with its `# text` lines joined by one space.
- The program read the test files of [EWT r2.18][ewt] and [GUM r2.18][gum] in an ephemeral 4x8 build box, since removed.
- The EWT file has SHA-256 `fa024f43dc5da3c5ac02563bc9bd0e974f46cbb1560823976a8f342a37dc494a`.
- The GUM file has SHA-256 `cd96a285e7339f401f4803dd0f4f61109c51692e3ef6f9186a1f977544080b69`.
- Neither file nor anything derived from it is committed.
- The scores on prose v3.2.2:

  | Set | Boundaries | Precision | Recall | F1 |
  |:--|--:|--:|--:|--:|
  | UD English-EWT test | 1223 | 0.9750 | 0.7980 | 0.8777 |
  | UD English-GUM test | 901 | 0.9586 | 0.9512 | 0.9549 |

- The design cites 90.8 F1 for Punkt on EWT from the [SaT paper][sat].  
  This run scores 87.8 on its own paragraph grouping, so the two numbers measure different things.

## 2. Splitter

- [ ] 2.1 Build Punkt's decision over a model: candidate end marks, abbreviations, pairs, sentence starters and the capitalization table.  
  Verify the plain, question and abbreviation scenarios of the [sentence-splitting spec][spec] pass as unit tests in CI.
- [ ] 2.2 Add the override rules for tokens with no space, closing marks, brackets and ellipses.  
  Verify the remaining scenarios pass as unit tests in CI.

## 3. Model

- [ ] 3.1 Build the trainer and a mise task that fetches the three sources, filters `github_archive` by license, and writes `data/splitter/en.tsv` with a source list.  
  Verify two runs in a build box write byte-identical files, and record the run time and token counts under this task.
- [ ] 3.2 Embed the model and parse it once per process.  
  Verify a test reads the embedded model in CI.

## 4. Accuracy and speed

- [ ] 4.1 Add the evaluation test.  
  Verify it passes in CI, and record hakkiri's and Vale's F1 per set under this task.
- [ ] 4.2 Split a 50,000-character paragraph and a 60,000-line file.  
  Verify both finish in CI with no panic, and record the time per paragraph.

[spec]: specs/sentence-splitting/spec.md
[docs-set]: ../../../tests/fixtures/splitter/docs.fixture
[hard-set]: ../../../tests/fixtures/splitter/hard.fixture
[golden-set]: ../../../tests/fixtures/splitter/golden-rules.fixture
[masc-set]: ../../../tests/fixtures/splitter/masc.fixture
[golden-license]: ../../../tests/fixtures/splitter/golden-rules.LICENSE
[sources]: ../../../tests/fixtures/splitter/SOURCES.md
[vale-f1]: ../../../tests/fixtures/splitter/vale-f1.tsv
[pysbd]: https://github.com/nipunsadvilkar/pySBD/blob/5905f13be4fc95f407b98392e0ec303617a33d86/tests/lang/test_english.py
[masc]: https://anc.org/data/masc/about/
[vale-gomod]: https://github.com/errata-ai/vale/blob/v3.26.0/go.mod
[vale-main]: https://github.com/errata-ai/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/go.mod
[ewt]: https://github.com/UniversalDependencies/UD_English-EWT/blob/b7711cce01cdd4f5fcc0a8199b8a50d951b16c0c/en_ewt-ud-test.conllu
[gum]: https://github.com/UniversalDependencies/UD_English-GUM/blob/b58e74bc22d17220c9198864c253a50a897bf27f/en_gum-ud-test.conllu
[sat]: https://arxiv.org/abs/2406.16678
