# Dictionary notice

`en_US-web.aff` and `en_US-web.dic` are the built-in English dictionary of [Vale][vale-data], byte for byte, at commit `66bbb85`.  
Their SHA-256 sums are `e3aa3f24f75ea49468f79f0aaf43e1095797f465d90ebd5a3e47690612e4b189` and `7e203a29114761b1c5d96d74d4c2011a9a202827cc64b1f1f0cee6a2fd377ba6`.  
hakkiri made no change to them.

The `.dic` file holds 81,558 words made from the English Speller Database (ESDB, previously SCOWL).  
The [copyright notice of the database][esdb-license] allows any use, as long as the notice appears in every copy, so it follows here.  
The official speller dictionaries need only the part before the `===` line.

```text
Copyright 2000-2026 by Kevin Atkinson

Permission to use, copy, modify, distribute, and sell any part of the English
Speller Database (ESDB, previously known as SCOWLv2), or word lists
created from it, is hereby granted without fee, provided that the above
copyright notice appears in all copies and that both the above copyright
notice and this notice appear in supporting documentation.  Kevin Atkinson
makes no representations about the suitability of this database for any
purpose.  It is provided "as is" without express or implied warranty.

ESDB is derived from many sources, most of which are in the Public Domain.
Data from the Corpus of Contemporary American English (COCA) was also used.

All data from COCA comes from 3-gram data that is not freely available;
however, the usage is within the rights given by the NDA that was signed when
purchasing the data.  More information on COCA is available at
https://www.english-corpora.org/coca/.

The primary source of words for ESDB comes from 12dicts and ENABLE2K.  Both
are in the Public Domain, but Alan Beale <biljir@pobox.com> deserves special
credit as he is the author of 12dicts and a major contributor to ENABLE2K.  In
addition, he gave me an incredible amount of feedback and created a number of
special lists in order to help improve the overall quality of ESDB.
```

The code in `src/dictionary` is a port of the `spell` package of Vale, which is derived from gospell.  
It is under the MIT license in [`vale-spell.LICENSE`](vale-spell.LICENSE).

[vale-data]: https://github.com/vale-cli/vale/tree/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/spell/data
[esdb-license]: https://github.com/en-wl/wordlist/blob/1e5b7d3a72f47a71da5d28686c1dd4b397178485/Copyright
