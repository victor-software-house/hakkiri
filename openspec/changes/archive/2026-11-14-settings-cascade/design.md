# Design

## 1. Specificity

hakkiri splits the text of a glob at each `/` into parts.
A literal character is one outside a `[…]` class that is not `*` or `?`.
A backslash and the character after it count as one literal character.

The specificity is four counts, compared in order.
For each count but the third, more is more specific:

1. The parts made of literal characters only, such as `docs` and `CHANGELOG.md`.
2. The parts that have a literal character and something else, such as `*.md` and `a?md`.
3. The parts that are `**`, where fewer is more specific.
4. The literal characters of the whole glob.

| File | Globs, the winner first | Why |
|:--|:--|:--|
| `docs/a.mdx` | `docs/*`, then `*.mdx` | 1 literal part against none |
| `docs/api/x.md` | `docs/**/*.md`, then `*.md` | 1 literal part against none |
| `CHANGELOG.md` | `CHANGELOG.md`, then `*.md` | 1 literal part against none |
| `a.md` | `a*.md`, then `*.md` | 4 literal characters against 3 |
| `openspec/changes/x/proposal.md` | `openspec/changes/**/proposal.md`, then `**/openspec/changes/**/*.md` | 3 literal parts against 2 |
| `a.md` | `a.md`, then whichever of `a?md` and `*.md` is written later | 1 literal part; then 3 literal characters each |
| `docs/a.md` | `{*.md,docs/a.md}`, then `docs/*.md` | the alternative `docs/a.md` has 2 literal parts against 1 |
| `docs/b.md` | `docs/*.md`, then `{*.md,docs/a.md}` | the alternative `*.md` has no literal part |

On 2026-11-13 the operator chose this measure over one that counts literal characters first, because a folder narrows a glob by its place.

A glob with `{…}` groups is read as its alternatives, one for each choice in each group.
For `{*.md,docs/a.md}` they are `*.md` and `docs/a.md`.
Each alternative has its own specificity.
For one file, the glob ranks as its most specific alternative that matches that file (row `HKR-072`).

So a skip list such as `{CHANGELOG.md,notes/**}` ranks as `CHANGELOG.md` for the changelog and as `notes/**` for a note.
It then beats a package's `*.{md,mdx,rs}`.
A glob may expand to 256 alternatives at most.
One with more is a fault, and its author must split it.

The operator chose this on 2026-11-13 over two measures that are fixed when the config loads.
They are the most specific alternative and the least specific one.
The first ranks `{*.md,docs/a.md}` as `docs/a.md` for every Markdown file.
The second ranks `{CHANGELOG.md,notes/**}` as `notes/**` for the changelog.

## 2. Order

hakkiri ranks the alternatives of every glob once, when the config loads.
It ranks them by specificity, then by origin, then as written.
Origin is the config file: each package in the order it loads, then the project.
So the project wins a tie.

A sort by those three keys over the alternatives in load order gives that rank.
A glob's written place breaks the last tie.
All the alternatives compile into one `GlobSet`, so one pass over a path finds every alternative that matches it.

Each glob that matches takes the highest rank among its matching alternatives.
The matching globs sort by that rank.
The top level of each config file ranks below every glob, in the same origin order.

`plan` folds the matching globs in rank order, and a later one wins for the same key.
Before this change it folded them in written order in the same way.
Then it folds the forced values of the matching globs in the same order.

`build_variants` walks the globs in the rank of the alternatives, once for each alternative of a glob.
It walks the unforced parameters of every glob first and then the forced ones.
The place a glob takes for any file is then among the choices.
So each set of parameters that a file can get is read once.

## 3. Keys

| Key | What competes | Forced |
|:--|:--|:--|
| `styles` | the list; it replaces a lower one, and an empty list drops the rule settings that rank below it | the forced list replaces the result, taken as written, with no `add_up` |
| a rule name | its switch (`on`, `off`, a level) and its level, as two keys | the same two keys |
| `params` | each parameter of each rule | a forced `params` mapping forces every parameter in it |
| `code_span_size` | the number | the number |
| `vocab`, `block_ignores`, `token_ignores` | nothing: every matching list adds up, in rank order | the forced lists add up among themselves and replace the unforced ones |

The tag sits on the value of a key directly under a glob, or on the top-level `code_span_size`.
A tag inside a value, such as on one parameter, is not read there.

serde buffers an untagged enum, and a tag below the buffered node reaches the enum as the mapping `{tag: value}`.
So `params: !override {Team.Long: {max: 30}}` forces `max`.
A tag on `max` alone fails the load, because a mapping stands where a value belongs.

## 4. Reading the tag

The glob settings are read as `Ordered<Ordered<Tagged<Setting>>>`, and `code_span_size` as `Option<Tagged<u64>>`.
`Tagged<T>` comes from ctl-core 0.7.5 (`ctl_core::input::Tagged`), which needs vsh-serde-saphyr 0.0.4.
That release gives an untagged enum the plain value of a tagged node.

A tag other than `!override` is a fault on the line of its key, and so is `!override` in a package's settings file.

## 5. Not changed

- `level`, the run options and `formats` are not per file, and keep their order: the project wins, then the later package.
- `add_up` still joins the `styles` lists of one glob text across config files.
  An empty `vocab` list still drops the lists of the earlier globs of its text.
- A glob that sets a vocabulary still runs that vocabulary's rules after its own settings.
