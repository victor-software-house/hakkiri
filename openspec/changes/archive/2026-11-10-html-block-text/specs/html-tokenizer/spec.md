## ADDED Requirements

### Requirement: A vertical tab is not whitespace inside a tag

hakkiri SHALL end a tag name and an attribute only at a space, tab, line feed, form feed or carriage return, as the tokenizer of `golang.org/x/net/html` does.  
The trimming of the text between tags SHALL trim a vertical tab too, as Vale does.

#### Scenario: A vertical tab in a tag

- **WHEN** an HTML block holds `<span`, a vertical tab and `class="code">marker</span>`
- **THEN** the tag name holds the vertical tab and `class="code"`, so no attribute is read, and `marker` reads as text

#### Scenario: A vertical tab around a text

- **WHEN** an HTML block holds `<p>`, a vertical tab, `marker` and a vertical tab before `</p>`
- **THEN** the paragraph reads `marker`
