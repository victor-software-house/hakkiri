# Spec Delta

## ADDED Requirements

### Requirement: A Markdown tree counts code blocks

The tree of a Markdown or MDX file SHALL hold the number of its fenced and indented code blocks.  
The tree of a source file SHALL hold 0.

#### Scenario: Two fences and an indented block

- **WHEN** a Markdown file holds two fenced code blocks and one block indented by four spaces
- **THEN** the tree holds a code block count of 3
