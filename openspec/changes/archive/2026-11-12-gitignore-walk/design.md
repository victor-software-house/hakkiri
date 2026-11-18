# Design

## Context

[`check-command`][check] walks a folder with `fs::read_dir`, skipping `node_modules`, `.git` and the package folders.
Vale at [66bbb85][v-walk] does the same with its default ignored folders and reads no git file.
`--fail-level` is a flag and a variable, and the config contains `level` only.

## Goals / Non-Goals

Goals:

- A walk skips the files git ignores, with git's own rules and precedence.
- A given file always lints, and a warning marks it when no glob selects it.
- Each run option is set in the config, a flag or a variable, in ctl's order.

Non-goals:

- A `.ignore` file, or any ignore file that is not git's.
- Hidden files: a walk reads dot files and dot folders as before, except `.git`.
- Reading the config's options from a package's settings.

## Decisions

### 1. The walk is the ignore crate's

The `ignore` crate is ripgrep's walker.
It reads `.gitignore` in every folder and in each parent up to the repository root, with git's precedence and negation.
The crate also reads `.git/info/exclude` and the global excludes file.
It applies them inside a git repository only, as ripgrep does: a `.gitignore` in a folder with no `.git` above it changes nothing.

hakkiri turns off its `.ignore` and hidden-file filters, and sorts each folder by path as before.
It keeps its own skips for `node_modules`, `.git`, the package folders and folders off the way to the project root.
The crate was authorized for the walk (operator, 2026-11-11), and its latest release is 0.4.34.

Alternative: read the `.gitignore` files with the crate's `gitignore` module and keep `read_dir`.
It lost because git defines the global excludes file, the parent folders and the precedence between the three.
The crate already contains that part.

### 2. A given file is not filtered

A file given on the command line is linted whether git ignores it or not, because the caller chose that file.
ripgrep does the same.
The walk root is not tested either, so `hakkiri check build` lints inside an ignored folder.
The entries below it follow the same rules as any walk.

Alternative: filter a given file too.
It lost because a caller who gives a file expects a result for it, and a skip with no message looks like a clean file.

### 3. A given file that no glob selects draws a warning

Today `plan` returns nothing for such a file and the run prints nothing.
A warning on standard error gives the file, and `on_unmatched` can make it an error with exit 2 or turn it off.

No warning goes to a file that a glob selects but whose styles contain no rule.
The config chose to run nothing there, as an empty `styles` list does for a changelog.

A file found by a walk draws nothing either.
Since a walk reads what the globs select, so most files it meets are not selected.
A file given twice warns once.

Alternative: a failure by default.
It lost because a pre-commit hook passes every staged file, and most of them are not prose.

### 4. Each option reads flag, variable, config, default

`--fail-level` and `--on-unmatched` take no default in the parser.
So an absent flag and variable leave the config's value to apply, and the default applies last.
`--gitignore` takes an optional boolean, as in `--gitignore=false` and the variable `HAKKIRI_GITIGNORE=false`.
`--no-gitignore` is the same as `--gitignore=false`.

ctl-core leaves a `--no-*` flag with no variable.
It drops a variable that a `--x` or `--no-x` flag on the command line overrules, so `--no-gitignore` wins over `HAKKIRI_GITIGNORE=true`.
A boolean flag that sets true reads the variable `false` as no change.
So `--gitignore` takes a value, and with it `HAKKIRI_GITIGNORE=false` turns the walk filter off.

Alternative: `--gitignore` and `--no-gitignore` as two plain flags.
It lost because the variable could then only turn the filter on, which is the default.

### 5. A package's file does not set them

The options belong to the run, and a package that set `gitignore: false` would change what every project that pins it reads.
The three keys in a package's settings file are an error that gives the key, the line and the package.
This is as for any key a package may not set.

Alternative: apply them like `level`, the last config winning.
It lost because `level` hides alerts the package's own rules give, while these set which files the project reads.

### 6. What stays different from Vale

Vale walks files git ignores and lints a given file whether or not a section matches it, reporting nothing when no rule runs ([lintPrepared][v-runs-nothing]).
hakkiri skips the ignored files and warns about the named file, and both differences are scenarios of the `vale-differences` spec.

[check]: ../../../specs/check-command/spec.md
[v-walk]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/lint.go#L283-L300
[v-runs-nothing]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/lint.go#L353-L358
