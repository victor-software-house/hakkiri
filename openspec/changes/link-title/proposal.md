# Proposal

Queue row: [`HKR-054`][ledger].

## Motivation

Vale reads the title of a Markdown link only through the `raw` scope.
No other scope holds it.
A probe of Vale at `66bbb85` put one rule on each of `text`, `paragraph`, `sentence`, `link`, `alt`, `heading`, `table`, `code` and `raw`.
It covered a link, an image, a single-quoted and a parenthesised title, two reference definitions, a heading and a table cell.
Only `raw` gave an alert at a title.
ProseStyle's `BareURL` rule reads `raw`, so Vale flags the URL in `[t](u "see https://x")`.
hakkiri's package moves that rule onto the scopes it can read cheaply, and no scope holds a title, so the URL goes unseen.

## Changes

- A scope `title` selects the text of the title of an inline link, an inline image and a link reference definition.
- No prose scope reads a title, as in Vale: `text`, `paragraph`, `link`, `alt` and `sentence` give no block of one.
- `title & link` and `title & image` select the title of a link and of an image, by the element that holds it, and a new scope `image` selects the alt text.
- The `title` attribute of a tag in an HTML block is a title, and in a line of text only `raw` reads it, as in Vale.
- The scope has its own name, because Vale has no scope for a title, and the dotted names of Vale keep their meaning.

## Capabilities

### Modified Capabilities

- `scope-selection`: a scope `title` selects link, image and definition titles.

## Impact

- New: the variant `Scope::Title` and its name `title`, and the nodes that carry it.
- Changed: the Markdown builder, `tests/select.rs`, the `scope-selection` spec and a patch changeset.
- Not changed: the alerts of any rule that names no `title` scope.

[ledger]: ../../../tasks.yaml
