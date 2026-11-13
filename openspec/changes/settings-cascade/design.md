# Design

## 1. Specificity

A glob's text is split at each `/` into parts.  
A literal character is one outside a `[…]` class and a `{…}` group that is not `*` or `?`; a backslash and the character after it count as one literal character.  
The specificity is four counts, compared in order, a larger count more specific, except the third:

1. The parts made of literal characters only, such as `docs` and `CHANGELOG.md`.
2. The parts that hold a literal character and something else, such as `*.md` and `a?md`.
3. The parts that are `**`, where fewer is more specific.
4. The literal characters of the whole glob.

| File | Globs, the winner first | Why |
|:--|:--|:--|
| `docs/a.mdx` | `docs/*`, then `*.mdx` | 1 literal part against none |
| `docs/api/x.md` | `docs/**/*.md`, then `*.md` | 1 literal part against none |
| `CHANGELOG.md` | `CHANGELOG.md`, then `*.md` | 1 literal part against none |
| `a.md` | `*.md`, then `*.{md,txt}` | 3 literal characters against 1 |
| `openspec/changes/x/proposal.md` | `openspec/changes/**/proposal.md`, then `**/openspec/changes/**/*.md` | 3 literal parts against 2 |
| `a.md` | `a.md`, then whichever of `a?md` and `*.md` is written later | 1 literal part; then 3 literal characters each |

The operator chose this measure on 2026-11-13 over one that counts literal characters first, because a folder narrows a glob by its place.  

## 2. Order

The globs are ranked once, when the config loads, by specificity, then by origin, then as written.  
Origin is the config file: each package in the order it loads, then the project, so the project wins a tie.  
A stable sort over the globs in load order gives that rank, so a glob's written place breaks the last tie.  
The top level of each config file ranks below every glob, in the same origin order.

`plan` folds the matching globs in rank order, a later one winning for the same key, as it folds them in written order today.  
Then it folds the forced values of the matching globs in the same order.  
`build_variants` walks the globs in the same rank, the unforced parameters of every glob and then the forced ones, so each set of parameters a file can get is read once.

## 3. Keys

| Key | What competes | Forced |
|:--|:--|:--|
| `styles` | the list, which replaces a lower one; an empty list drops the rule settings that rank below it | the forced list replaces the result, taken as written, with no `add_up` |
| a rule name | its switch (`on`, `off`, a level) and its level, as two keys | the same two keys |
| `params` | each parameter of each rule | a forced `params` mapping forces every parameter it holds |
| `code_span_size` | the number | the number |
| `vocab`, `block_ignores`, `token_ignores` | nothing: every matching list adds up, in rank order | the forced lists add up among themselves and replace the unforced ones |

The tag sits on the value of a key directly under a glob, or on the top-level `code_span_size`.  
A tag inside a value, such as on one parameter, is not read there: serde buffers an untagged enum, and a tag below the buffered node reaches the enum as the mapping `{tag: value}`.  
So `params: !override {Team.Long: {max: 30}}` forces `max`, and a tag on `max` alone fails the load as a mapping where a value belongs.

## 4. Reading the tag

The glob settings are read as `Ordered<Ordered<Tagged<Setting>>>`, and `code_span_size` as `Option<Tagged<u64>>`.  
`Tagged<T>` comes from ctl-core 0.7.5 (`ctl_core::input::Tagged`), which needs vsh-serde-saphyr 0.0.4: that release gives an untagged enum the plain value of a tagged node.  
A tag other than `!override` is a fault on the line of its key, and so is `!override` in a package's settings file.

## 5. Not changed

- `level`, the run options and `formats` are not per file, and keep their order: the project wins, then the later package.
- `add_up` still joins the `styles` lists of one glob text across config files, and an empty `vocab` list still drops the lists of the earlier globs of its text.
- A glob that names a vocabulary still runs that vocabulary's rules after its own settings.
