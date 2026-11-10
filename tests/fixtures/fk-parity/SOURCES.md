# Flesch-Kincaid parity set

Vale's readability counts for the case inputs of [ProseStyle][prose-style], for the parity test of the [rule engine][change].  
The values are in [`vale-fk.tsv`](vale-fk.tsv), one row per input.

## Format

| Column | Holds |
|:--|:--|
| `id` | The rule file or `package` cases file, then the case's position in its `tests:` list, as `KindAsNoun-02`. The input is `inputs/<id>.<format>.fixture` |
| `vale` | `3.26.0` |
| `prose_style` | `ee9f207`, the commit the inputs come from |
| `format` | `md`, `txt` or `rs`, the extension Vale read the input as |
| `status` | `measured`, or `no-words` where Vale printed no value |
| `flesch_kincaid` | Vale's `flesch_kincaid`, to six decimals |
| `words`, `sentences`, `syllables` | Vale's counts |

A `no-words` row has empty value columns.  
Vale returns no metrics for a document with no counted words, and it printed none for any `rs` input or for a table.

## Inputs

- Source: [prose-style at `ee9f207`][prose-style-commit], public, with no license file; the owner is the same as hakkiri's.
- A case is each entry of `tests:` in a rule file under `ProseStyle/styles/ProseStyle/`, and each entry of `test/package.test.yml`.
- That is 87 cases.
  21 hold a code span or fence and are left out, since a code span counts one word on purpose.
  The other 66 are in [`inputs/`](inputs), copied as the `input` text of the case, byte for byte.
- A case with `format: rs` or `.rs` is a Rust file, `.txt` a text file, and every other case Markdown.
- 51 inputs have values, and 15 are `no-words`: 10 `rs`, 4 Markdown tables and 1 Markdown line that holds only a URL.

## Rerun

Vale 3.26.0 ran once in a throwaway Linux box, from the official release `vale_3.26.0_Linux_64-bit.tar.gz` (SHA-256 `22aa69a556558e4a9dfc63c17030777383d8d3c48dcdfad655fa98e839d666ed`).  
The measuring files are not committed.

1. Write four `metric` rules in a style named `Metrics`, one per value.
   Each has `formula` set to `words`, `sentences`, `syllables` or `flesch_kincaid * 1000000`, `condition: "> -1000000000"` and `message: "<name>=%s"`.
   Vale prints one `%.2f` value per message, and the factor keeps six decimals of the score.
2. Set `[*.{md,txt,rs}]` to `BasedOnStyles = Metrics` in a `.vale.ini`.
3. Run `vale --output=JSON inputs` and read each file's four messages.
   Divide the score by 1,000,000.

[prose-style]: https://github.com/victor-software-house/prose-style
[prose-style-commit]: https://github.com/victor-software-house/prose-style/tree/ee9f207279864e305d7372aef4a6c5dacc3eb846
[change]: ../../../openspec/changes/rule-engine/design.md
