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
- [`masc.fixture`][masc-set] holds 284 sentences in 71 records from [MASC 3.0.0][masc].
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

- [x] 2.1 Build Punkt's decision over a model: candidate end marks, abbreviations, pairs, sentence starters and the capitalization table.  
  Verify the plain, question and abbreviation scenarios of the [sentence-splitting spec][spec] pass as unit tests in CI.
  - 2026-11-10 (UTC), `two_plain_sentences`, `a_question_inside_a_paragraph`, `a_title_before_a_name` and `a_latin_abbreviation_in_the_middle` in `src/splitter/tests.rs` run the embedded model.  
    [Verify][run-splitter] passed on [PR 14][pr14] at `cac88f4`.
- [x] 2.2 Add the override rules for tokens with no space, closing marks, brackets, ellipses and list numbers.  
  Verify the remaining scenarios pass as unit tests in CI.
  - 2026-11-10 (UTC), one test per remaining scenario passed in the same [Verify][run-splitter] run.  
    The tests for the list number and for a chain of initials cover the two rules added after the design.

## 3. Model

- [x] 3.1 Build the trainer and mise tasks that fetch the four sources, filter `stackv2` by license, and write `data/splitter/en.tsv` with a source list.  
  Verify two runs in a build box write byte-identical files, and record the run time and token counts under this task.
  - 2026-11-10 (UTC), two runs of `mise run splitter:train` in a 4x8 build box wrote byte-identical `en.tsv` and `SOURCES.md`, and the fetched corpus files were identical too.  
    The SHA-256 of `en.tsv` is `b4ec6f04…` and of `SOURCES.md` is `dd147b99…` in both runs.
  - Each run took about 2 minutes: the fetch about 75 seconds, reading the corpus about 22 seconds, training 5.8 seconds.
  - The corpus holds 26,079,695 tokens.  
    A token is a whitespace-separated word, counted after the Markdown sources are cut to the text of their paragraphs.
  - The owner's 48 public repositories give 1,037 documents and 574,192 tokens.
  - The first 512 MiB of `stackv2` shard 129 gives 45,930 Markdown documents and 10,717,783 tokens.
  - The Python Enhancement Proposals give 818 documents and 1,625,676 tokens.
  - The first 32 MiB of `project_gutenberg` shard 0 gives 186 documents and 13,162,044 tokens.
  - The model is 570,625 bytes in 24,598 lines: 2,629 abbreviations, 1,181 collocations, 787 sentence starters and 20,000 capitalization entries cut from 815,218.  
    The data comes from 1,257,144 word types, 1,222,335 tokens that end in a period.
  - `stackv2` spreads its Markdown over many of its 764 shards, mixed with code.  
    Shard 129 holds 45,915 `.md` and `.markdown` records, the densest of the shards counted, and a prefix of it is read.
  - `github_archive` holds issue and pull request threads, not `.md` files, so it is not a source.
- [x] 3.2 Embed the model and parse it once per process.  
  Verify a test reads the embedded model in CI.
  - 2026-11-10 (UTC), `the_embedded_model_is_read_and_is_in_file_order` reads the embedded model, finds `dr` in it, and checks that the model writes back the same text as the file.  
    [Verify][run-splitter] passed on [PR 14][pr14] at `cac88f4`.

## 4. Accuracy and speed

- [ ] 4.1 Add the evaluation test.  
  Verify it passes in CI, and record hakkiri's and Vale's F1 per set under this task.
- [x] 4.2 Split a 50,000-character paragraph and a 60,000-line file.  
  Verify both finish in CI with no panic, and record the time per paragraph.
  - 2026-11-10 (UTC), `a_50000_character_paragraph_splits`, `a_50000_character_run_with_no_space_is_one_sentence` and `a_60000_line_input_splits` passed in [Verify][run-splitter] on [PR 14][pr14] at `cac88f4`.
  - In the build box, a 50,000-character paragraph of 2,845 sentences took 8 ms in a debug build and 0.7 ms in a release build.  
    A 60,000-line input of 120,000 sentences took 290 ms and 20 ms.  
    The cost is linear in the length of the text.

[spec]: specs/sentence-splitting/spec.md
[run-splitter]: https://github.com/victor-software-house/hakkiri/actions/runs/42817055813
[pr14]: https://github.com/victor-software-house/hakkiri/pull/14
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
