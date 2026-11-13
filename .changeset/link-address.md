---
hakkiri: patch
"verctl:group": fixed
---

Count no word for a Markdown link whose text is its address, as Vale counts none.
The text is compared with the address as goldmark writes it, percent-encoded, so `[ä.md](ä.md)` and `[a b](<a b>)` keep their words, as in Vale.
