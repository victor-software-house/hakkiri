## ADDED Requirements

### Requirement: The comments of a fenced code block are read

hakkiri SHALL read the comments of a fenced code block when it reads the comments of the block's language, as the `document-tree` spec sets.  
Vale 66bbb85 reads no text inside a fence.

#### Scenario: A Rust fence in Markdown

- **WHEN** a Markdown file contains a fence opened with ```` ```rust ```` around `// A very good start.`, and a rule flags `very`
- **THEN** hakkiri reports `very` in the comment, and Vale reports nothing ([decision 5][language-model-5])

### Requirement: A scope part names a Linguist language

hakkiri SHALL read the last piece of a scope part as a language of the GitHub Linguist table that langdex contains.  
Vale 66bbb85 compares it with the file's extension.  
Before that, its own table folds some extensions together, such as `.tsx`, `.mts` and `.cts` into `.ts`.

Linguist keeps TSX (TypeScript with JavaScript XML) apart from TypeScript.  
So a scope that gives `ts` does not reach a `.tsx` file.

#### Scenario: A TSX file

- **WHEN** a rule has `scope: text.comment.ts` and `tokens: [marker]`, and `a.tsx` contains `// A marker.`
- **THEN** Vale reports one alert, and hakkiri reports none ([decision 4][language-model-4])

### Requirement: A file with no extension is read by its interpreter

hakkiri SHALL read a file whose path gives no language as the language of its first-line interpreter.  
Vale 66bbb85 finds no format for such a file and lints nothing in it.

#### Scenario: A script

- **WHEN** `bin/tool` has no extension, starts with `#!/usr/bin/env node`, contains `// A very good start.`, and a rule flags `very`
- **THEN** hakkiri reads it as JavaScript and reports `very`, and Vale reports nothing ([decision 2][language-model-2])

[language-model-2]: ../../design.md#2-the-language-of-a-file
[language-model-4]: ../../design.md#4-a-scope-part-names-a-language
[language-model-5]: ../../design.md#5-fenced-code-is-parsed-with-the-grammar-of-the-fence-language
