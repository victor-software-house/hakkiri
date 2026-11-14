## ADDED Requirements

### Requirement: Sentences split where a reader splits them

hakkiri SHALL split sentences by its own splitter where Vale 66bbb85's segmenter splits otherwise, in the places that `tests/sentence_differences.rs` pins.

#### Scenario: A numbered heading or item

- **WHEN** a Markdown file holds `## 1. Evaluation sets` or `- [x] 4. A patch changeset ships with the change.`
- **THEN** hakkiri reads the heading or item as one sentence, and Vale reads the `1.` or `4.` as a sentence of its own

#### Scenario: A version, a decimal or a task number before a capital

- **WHEN** a file holds `The team pins ctl-core 0.5.2. ctl-core 0.6.3 ships the new look.`
- **THEN** hakkiri gives two sentences, and Vale gives one, as it does after the decimal `4.75.` and the task number `1.3.`

#### Scenario: vs. between two versions

- **WHEN** a file holds `Rust 1.99 vs. Go 1.25 differ.`
- **THEN** hakkiri gives one sentence, and Vale ends the first after `vs.`
