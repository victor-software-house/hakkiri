## ADDED Requirements

### Requirement: A bare address ends where GFM ends it

hakkiri SHALL end a bare web address at whitespace or `<`, as GFM and its Markdown parser do.  
Vale ends one at the first byte that goldmark 1.8.6's linkify pattern leaves out, which holds no letter outside ASCII, and reads the rest as text.

#### Scenario: A bare address with a letter outside ASCII

- **WHEN** a Markdown file holds `Visit https://b.example/ä for special thanks.`
- **THEN** hakkiri counts 8 words, the words of the whole address among them, and Vale 66bbb85 counts 5, since its link ends before `ä` ([link-address][link-address])

[link-address]: ../../proposal.md
