# Tasks

Every compile, lint and test step runs in the pull request's CI, not on the authoring machine.  
Training and measurement run in an ephemeral build box.

## 1. Evaluation sets

- [ ] 1.1 Label sentence boundaries by hand in paragraphs from the owner's public repositories, including the 89 measured hard cases.  
  Verify each paragraph's source and license are listed beside the set.
- [ ] 1.2 Add the Golden Rules list and a MASC sample with their gold boundaries, after reading each license file.  
  Verify the license of each set is named beside it.
- [ ] 1.3 Measure Vale 3.26.0's F1 on each set, and on UD English-EWT and GUM.  
  Commit the numbers for the committed sets.  
  Record the EWT and GUM numbers under this task, with the command that produced them.

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
