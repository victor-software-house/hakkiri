# Proposal

Queue row: `HKR-015` in [tasks.yaml][ledger].

## Motivation

The `sentence` scope, `LongSentence` and every metric that counts sentences need a sentence splitter.  
Vale uses Punkt with NLTK's English model, trained on Wall Street Journal text from the LDC.  
No license covers that model ([nltk_data#188][nltk-188]), so hakkiri cannot ship it.

The splitter is also where hakkiri can beat Vale.  
Punkt is known to fail on brackets, lists and abbreviation chains ([pySBD paper][pysbd]).  
A Punkt model trained afresh splits better than the one NLTK ships ([Gillick 2009][gillick]).  
The [rule-engine change][engine] reads sentences through this splitter.

## Changes

- An own implementation of Punkt, with override rules for the cases Punkt is known to miss.
- An English model trained by hakkiri's own trainer on documentation Markdown and public-domain books, embedded in the binary.
- An evaluation set committed with gold sentence boundaries, and a test that hakkiri splits it at least as well as Vale.

## Capabilities

### New Capabilities

- `sentence-splitting`: where a sentence ends in the text of a prose node, and how well the splitter must do.

### Modified Capabilities

None.

## Impact

- No new dependency; the model is a data file in the repository.
- New modules for the splitter, the model reader and the trainer, and a mise task that trains the model in a build box.
- `HKR-004` cuts sentences with this splitter, so it waits for this row.

[ledger]: ../../../../tasks.yaml
[engine]: ../2026-11-10-rule-engine/proposal.md
[nltk-188]: https://github.com/nltk/nltk_data/issues/188
[pysbd]: https://aclanthology.org/2020.nlposs-1.15.pdf
[gillick]: https://aclanthology.org/N09-2061.pdf
