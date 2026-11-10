# Splitter model sources

`en.tsv` is the output of `mise run splitter:train` on the sources below.  
Two runs on these sources write byte-identical files.

## Corpus

| File | Documents | Tokens |
|:--|--:|--:|
| `repositories` | 1037 | 574192 |
| `stackv2` | 46146 | 10782267 |
| `peps` | 818 | 1625676 |
| `books` | 186 | 13162044 |

## Repositories

`repositories.tsv` lists the Markdown sources: public repositories of the owner, each at the commit below.  
A repository is listed when it is public, is not a fork, is not archived, and is at most 10 MB.  
A license of `none` means the repository declares no license.  
The corpus holds every tracked `.md` file except `CHANGELOG.md`, and the trainer reads the text of its paragraphs.

| Repository | Commit | License |
|:--|:--|:--|
| victor-software-house/.github | `bbe605d642915d0dc2c913724dfea9b21611c9d3` | none |
| victor-software-house/actions | `a333d6f8cb6a3dfd55088fec5a22924262017212` | none |
| victor-software-house/anti-slop | `08e6b53e8b1d86e65a81ff82aeb122ddc0ad6af7` | MIT |
| victor-software-house/bun-release | `993254e382a155553af292066e52410ab191bcb9` | MIT |
| victor-software-house/ctl-core | `1d6746b33cb27ca7a9c235a218dbdacf7162293c` | MIT |
| victor-software-house/exa-cli | `2a83afa232ec01fc64651af9c304f15cdacf6b28` | MIT |
| victor-software-house/fnox-export | `a9b5aa79dd5520b2943a12cb43a9b91c7b5321e4` | MIT |
| victor-software-house/forkctl | `b6894a38f175c71675f97eeadce99442a282fecc` | MIT |
| victor-software-house/frontend-design-system | `38a02e6383001cae493f30f80d035992d113e3a7` | none |
| victor-software-house/hakkiri | `63e35c93b074a0a7ee8e515f1182a4f367437213` | AGPL-3.0 |
| victor-software-house/herdr-restore-notice | `7900828e51ddca7eee454051e2f17e995afbfb1e` | MIT |
| victor-software-house/herdr-stash | `5618e89585cd92fef9215cc5630cce67dfb9c23b` | MIT |
| victor-software-house/history-search | `a8dc8cc284852b3cc9c479d7451363944ef996e0` | none |
| victor-software-house/homebrew-forks | `8a56d8649294cbd15ab8c5e57addcd06f376c7d2` | none |
| victor-software-house/hunk-history | `9843a303eb5b0153d7c19c90d391275cd4d7fefb` | MIT |
| victor-software-house/is-node-vulnerable | `04ad3caba41a7d8bd81f2e65270cb12247c5f3d1` | MIT |
| victor-software-house/mintlify-docs | `978d7abc56d15702adbb944af99dbcb31c19c0de` | MIT |
| victor-software-house/mise-env-fnox | `48f5938f66dc585951e3790aa2ee1cf1f12da362` | MIT |
| victor-software-house/nishiki-sources | `c1f7e967eeeffa3b15627c10f21ae404345f0e26` | none |
| victor-software-house/opencode-cursor-inference | `a7f132f78f657026cee08cac3693305e0505fd85` | MIT |
| victor-software-house/pi-agent-browser | `43cccc85ee109cbc4405f58ebb518ebc3d0fc639` | MIT |
| victor-software-house/pi-curated-themes | `ac8e0c8e890a8ee6ae926c6a195f16b9f0033bbb` | MIT |
| victor-software-house/pi-cursor | `0155c9f7ec0abb7437442cbde198f444ee05589c` | MIT |
| victor-software-house/pi-ghostty-theme-sync | `e608cdaf2076c29cb5474d70e9eada54a00bb802` | none |
| victor-software-house/pi-ghostty-themes | `701a0a864b0da1113ac9323a55920bb6b3ad2e1b` | MIT |
| victor-software-house/pi-multicodex | `f80c63c8ff00ae29c1b8c88821c65b4d4ef9da81` | MIT |
| victor-software-house/pi-openai-proxy | `deccaac16fb83cc0e2fffd661009877eeb9ceee3` | MIT |
| victor-software-house/pi-prompt-composer | `f4a1163df113c931317e0e6e9ca2645eb4bf77fb` | MIT |
| victor-software-house/pi-provider-utils | `01d25fb9557fadd2e5c35e5300514b0a63ba4d93` | MIT |
| victor-software-house/pi-skills-manager | `c716fdba84895749145bd6069732c9212b5fc697` | MIT |
| victor-software-house/pi-term | `dbc891069250de406cb9be2d03b4c7e5eb718114` | MIT |
| victor-software-house/prose-style | `ee9f207279864e305d7372aef4a6c5dacc3eb846` | none |
| victor-software-house/qctl | `802ac0d3b0c4b478433f944ba0a89c3e665d0b40` | MIT |
| victor-software-house/quickbar | `a10018786ef7b1538c2c6703e2f0ca376fab6dab` | MIT |
| victor-software-house/rawr | `40abc93a3ef4bb293540bfce6320a69d4ebbc5d9` | MIT |
| victor-software-house/reviewbench | `aab24a579bad5f93fe9c90ee56f4c5c6c39bbe0e` | MIT |
| victor-software-house/rgbctl | `09b5f71eedc88b8b9910618cee64f1f0f7ccacbc` | MIT |
| victor-software-house/shrub | `86ef0f4f5c26556880d1517eb105984b83c92b04` | MIT |
| victor-software-house/swift-libdeflate | `8a7462b30000f1344308e62cd30f78cb4890846f` | MIT |
| victor-software-house/swift-ngtcp2 | `29b0a48d6b8fdd75d88373f539fc8204c4ab7a5e` | MIT |
| victor-software-house/swift-simdjson | `b2b75acaaece1dab57b767b4583568ac673507d6` | Apache-2.0 |
| victor-software-house/swift-zstd | `552001d52e803b638ddd9c8bd8be93b1d1f8de6c` | MIT |
| victor-software-house/task-tracker-plugin | `4e0b2df87a2fad9cfd79388c1435861cc9b6f85b` | none |
| victor-software-house/unixd | `cd5be7d3eb4e5106650ee7c13b519e45ca529819` | MIT |
| victor-software-house/verctl | `98efe628ea3b26a9e89d0ab81cee032b79260a66` | MIT |
| victor-software-house/victor-software-house.github.io | `b75f81af159fec0398b6e50e95293493c69d711c` | none |
| victor-software-house/victor.workspace | `fb883b543540aa210353b0af180dd5690c17e6c1` | none |
| victor-software-house/yamled | `0796bf22ddf53f120378d216c9f99c0beb8e340d` | MIT |

## Datasets

[Common Pile](https://arxiv.org/abs/2506.05209) datasets on Hugging Face, each at a pinned revision.  
`datasets.tsv` lists the files read, and how many bytes of each; 0 means the whole file.

| Dataset | Revision | File | Bytes | License |
|:--|:--|:--|--:|:--|
| common-pile/stackv2 | `d0e3266fce12d25de28f2576ffb7272c18b0148f` | `documents/00129_stackv2.jsonl.gz` | 536870912 | Blue Oak Council list, per repository |
| common-pile/python_enhancement_proposals | `f932757e3eba16475c893e1418918c77f14a790d` | `v0/documents/00000_peps.jsonl.gz` | 0 | Public Domain |
| common-pile/project_gutenberg | `01dc90a5002f8977c7fb03a372c14bca29c65cf1` | `v0/documents/00000_pg.jsonl.gz` | 33554432 | Public Domain |

### Filters

- `common-pile/stackv2` ([dataset card](https://huggingface.co/datasets/common-pile/stackv2)) holds code files from GitHub repositories whose detected licenses are all on the Blue Oak list.  
  A record is kept when its `metadata.path` ends in `.md` or `.markdown`, in any case, and every license in its `metadata.license`, a comma-separated list, is one of the 225 ids in `licenses.txt`.  
  That file is the [Blue Oak Council list](https://blueoakcouncil.org/list), version 16.
- `common-pile/python_enhancement_proposals` ([dataset card](https://huggingface.co/datasets/common-pile/python_enhancement_proposals)) holds the Python Enhancement Proposals in the public domain, as plain text.  
  A record is kept when `metadata.license` is `Public Domain`.
- `common-pile/project_gutenberg` ([dataset card](https://huggingface.co/datasets/common-pile/project_gutenberg)) holds books.  
  A record is kept when `metadata.license` is `Public Domain` and `metadata.language` is `en`.

## Result

| Count | Value |
|:--|--:|
| Tokens read | 26144179 |
| Word types | 1262092 |
| Tokens that end in a period | 1225698 |
| Possible sentence ends | 1295408 |
| Abbreviations | 2655 |
| Collocations | 1183 |
| Sentence starters | 793 |
| Capitalization entries seen | 817997 |
| Capitalization entries kept | 20000 |
