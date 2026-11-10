# Tagger model notice

`en.bin` is the English part-of-speech model of [prose][prose-model], byte for byte, at commit `b33e60d`.  
Its SHA-256 is `031ef22b05a03ea31ebc154bffd396e8497360546671d48f7b921f20ef4c9345`.  
hakkiri made no change to it.

| | |
|:--|:--|
| Format | prose's flat tagger format, version 2, Penn Treebank tags, the `runes` feature template |
| Training data | UD_English-EWT at `4a4d77f` (2026-05-06), from the [Universal Dependencies][ud] project |
| License of the model | [CC BY-SA 4.0][cc-by-sa] ([model list][prose-models]) |

The model stays a separate data file, so the share-alike license covers this file and any change to it, and not the source code of hakkiri.

The code in `src/tagger` is a port of the `tag`, `tokenize` and `internal/perceptron` packages of prose, which are under the MIT license in [`prose.LICENSE`](prose.LICENSE).

[prose-model]: https://github.com/jdkato/prose/blob/b33e60dbff0492289b227f3681120ac057d8131b/tag/aptagmodel/en.bin
[prose-models]: https://github.com/jdkato/prose/blob/b33e60dbff0492289b227f3681120ac057d8131b/tag/lang/MODELS.md
[ud]: https://universaldependencies.org/
[cc-by-sa]: https://creativecommons.org/licenses/by-sa/4.0/
