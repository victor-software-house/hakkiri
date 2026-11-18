# Spec Delta

## ADDED Requirements

### Requirement: A Markdown tree counts code blocks

The tree of a Markdown or Markdown with components (MDX) file SHALL have the number of its fenced and indented code blocks.  
The tree of a source file SHALL have 0.

#### Scenario: Two fences and an indented block

- **WHEN** a Markdown file has two fenced code blocks and one block indented by four spaces
- **THEN** the tree has a code block count of 3
