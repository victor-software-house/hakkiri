//! The dictionaries and word lists of a `spelling` rule, read from the
//! package that holds the rule and from no other place.

use std::collections::HashSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

use super::LoadError;
use super::build::Source;
use super::schema::SpellingFields;
use crate::dictionary::{AffixFile, Book, Dictionary, DictionaryError, Speller, read_affixes};

/// Where a package keeps its dictionaries.
const DICTIONARIES: &str = "config/dictionaries";

/// Where a package keeps its word lists.
const IGNORE: &str = "config/ignore";

/// The name of the built-in dictionary's affix file.
const BUILTIN_AFFIX: &str = "en_US-web";

/// The built-in dictionary.
fn builtin(source: &Source<'_>) -> Result<Book, LoadError> {
    Dictionary::english().map(Book::Builtin).map_err(|error| {
        source.problem(
            "dictionaries",
            format!("rule `{}`: the built-in dictionary: {error}", source.name),
        )
    })
}

/// The file at `relative` in `package`, `None` when there is none.
///
/// # Errors
///
/// Returns what is wrong when `relative` leaves the package: it is absolute,
/// climbs with `..`, or reaches a file outside the package through a link.
fn find(package: &Path, relative: &str) -> Result<Option<PathBuf>, String> {
    let inside = Path::new(relative)
        .components()
        .all(|part| matches!(part, Component::Normal(_) | Component::CurDir));
    if !inside {
        return Err(format!("`{relative}` leaves the package"));
    }
    let path = package.join(relative);
    if !path.exists() {
        return Ok(None);
    }
    let reached = path
        .canonicalize()
        .map_err(|error| format!("`{relative}`: {error}"))?;
    let root = package
        .canonicalize()
        .map_err(|error| format!("the package: {error}"))?;
    if reached.starts_with(root) {
        Ok(Some(reached))
    } else {
        Err(format!("`{relative}` leaves the package"))
    }
}

/// The dictionary whose two files are `affix` and `words`, each a path in
/// `package`.
fn read_dictionary(
    source: &Source<'_>,
    field: &str,
    named: &str,
    affix: &Path,
    words: &Path,
) -> Result<Book, LoadError> {
    let refuse = |message: String| {
        source.problem(
            field,
            format!("rule `{}`: dictionary `{named}`: {message}", source.name),
        )
    };
    let read = |path: &Path| fs::read(path).map_err(|error| refuse(error.to_string()));
    let (aff, dic) = (read(affix)?, read(words)?);
    Dictionary::parse(&aff, &dic)
        .map(|dictionary| Book::Package(Box::new(dictionary)))
        .map_err(|error| refuse(error.to_string()))
}

/// The two files of dictionary `name`, `name.aff` and `name.dic`, in the
/// folder `folder` of `package`.
fn named(source: &Source<'_>, package: &Path, folder: &str, name: &str) -> Result<Book, LoadError> {
    let refuse = |message: String| {
        source.problem(
            "dictionaries",
            format!("rule `{}`: dictionary `{name}`: {message}", source.name),
        )
    };
    let locate = |extension: &str| {
        let relative = format!("{folder}/{name}.{extension}");
        match find(package, &relative) {
            Ok(Some(path)) => Ok(path),
            Ok(None) => Err(refuse(format!("`{name}.{extension}` is not in `{folder}`"))),
            Err(message) => Err(refuse(message)),
        }
    };
    read_dictionary(
        source,
        "dictionaries",
        name,
        &locate("aff")?,
        &locate("dic")?,
    )
}

/// The file `relative` names for `aff` or `dic`: at that path in the package,
/// or in its dictionaries folder.
fn dictionary_file(
    source: &Source<'_>,
    package: &Path,
    field: &str,
    relative: &str,
) -> Result<PathBuf, LoadError> {
    let refuse =
        |message: String| source.problem(field, format!("rule `{}`: {message}", source.name));
    for candidate in [relative.to_owned(), format!("{DICTIONARIES}/{relative}")] {
        if let Some(path) = find(package, &candidate).map_err(refuse)? {
            return Ok(path);
        }
    }
    Err(refuse(format!(
        "`{relative}` is in neither the package nor `{DICTIONARIES}`"
    )))
}

/// The words each list of `ignore` holds, one word to a line.
///
/// A list is the file at that path in the package, or in its word list
/// folder, and each that exists is read.
fn lists(
    source: &Source<'_>,
    package: &Path,
    ignore: &[String],
) -> Result<HashSet<String>, LoadError> {
    let refuse =
        |message: String| source.problem("ignore", format!("rule `{}`: {message}", source.name));
    let mut listed = HashSet::new();
    for entry in ignore {
        for candidate in [entry.clone(), format!("{IGNORE}/{entry}")] {
            let Some(path) = find(package, &candidate).map_err(refuse)? else {
                continue;
            };
            let text =
                fs::read_to_string(&path).map_err(|error| refuse(format!("`{entry}`: {error}")))?;
            listed.extend(
                text.lines()
                    .map(str::trim)
                    .filter(|word| !word.is_empty() && *word != "#")
                    .map(str::to_owned),
            );
        }
    }
    Ok(listed)
}

/// The dictionaries and word lists of the rule.
///
/// `dictionaries` names dictionaries in `dicpath`, or in the package's
/// dictionaries folder, and `append` adds the built-in one to them.
/// `aff` and `dic` name one dictionary and nothing else.
/// A rule that names none uses the built-in dictionary.
pub(super) fn speller(source: &Source<'_>, fields: &SpellingFields) -> Result<Speller, LoadError> {
    let (aff, dic, dicpath) = (
        fields.aff.value(),
        fields.dic.value(),
        fields.dicpath.value(),
    );
    let names = fields.dictionaries.values();
    let ignore = fields.ignore.values();
    let reads_files = [aff, dic, dicpath].iter().any(|text| !text.is_empty())
        || !names.is_empty()
        || !ignore.is_empty();
    let Some(package) = source.package else {
        if reads_files {
            return Err(source.problem(
                "dictionaries",
                format!(
                    "rule `{}` names a dictionary or a word list, which only a rule in a \
                     package can read",
                    source.name
                ),
            ));
        }
        return Ok(Speller::new(vec![builtin(source)?], HashSet::new()));
    };
    let mut books = Vec::new();
    if aff.is_empty() != dic.is_empty() {
        let (field, other) = if aff.is_empty() {
            ("aff", "dic")
        } else {
            ("dic", "aff")
        };
        return Err(source.problem(
            field,
            format!(
                "rule `{}`: `aff` and `dic` name the two files of one dictionary: \
                 set `{field}` beside `{other}`",
                source.name
            ),
        ));
    }
    if aff.is_empty() {
        let folder = if dicpath.is_empty() {
            DICTIONARIES
        } else {
            match find(package, dicpath) {
                Ok(Some(path)) if path.is_dir() => dicpath,
                Ok(_) => {
                    return Err(source.problem(
                        "dicpath",
                        format!(
                            "rule `{}`: `{dicpath}` is not a folder in the package",
                            source.name
                        ),
                    ));
                }
                Err(message) => {
                    return Err(
                        source.problem("dicpath", format!("rule `{}`: {message}", source.name))
                    );
                }
            }
        };
        for name in names {
            books.push(named(source, package, folder, name)?);
        }
        if names.is_empty() || fields.append.value() {
            books.push(builtin(source)?);
        }
    } else {
        let affix = dictionary_file(source, package, "aff", aff)?;
        let words = dictionary_file(source, package, "dic", dic)?;
        books.push(read_dictionary(source, "aff", aff, &affix, &words)?);
    }
    Ok(Speller::new(books, lists(source, package, ignore)?))
}

/// An affix file that names the forms of a word.
pub(super) enum Forms {
    /// The built-in dictionary's.
    Builtin(&'static Dictionary),
    /// One read from the package.
    File(Box<AffixFile>),
}

impl Forms {
    /// The forms that `word` makes with the flags `flags`.
    ///
    /// # Errors
    ///
    /// Returns a [`DictionaryError`] when a flag names no affix class, or
    /// flags have no word.
    pub(super) fn of(&self, word: &str, flags: &str) -> Result<Vec<String>, DictionaryError> {
        match self {
            Self::Builtin(dictionary) => dictionary.expand(word, flags),
            Self::File(file) => file.expand(word, flags),
        }
    }
}

/// The affix file `name` of an `affix` field: `name.aff` in the package's
/// dictionaries folder, or else the built-in one when `name` is `en_US-web`.
pub(super) fn forms(source: &Source<'_>, name: &str) -> Result<Forms, LoadError> {
    let refuse = |message: String| {
        source.problem(
            "affix",
            format!("rule `{}`: affix `{name}`: {message}", source.name),
        )
    };
    if let Some(package) = source.package
        && let Some(path) = find(package, &format!("{DICTIONARIES}/{name}.aff")).map_err(refuse)?
    {
        let bytes = fs::read(&path).map_err(|error| refuse(error.to_string()))?;
        return read_affixes(&bytes)
            .map(|file| Forms::File(Box::new(file)))
            .map_err(|error| refuse(error.to_string()));
    }
    if name == BUILTIN_AFFIX {
        return Dictionary::english()
            .map(Forms::Builtin)
            .map_err(|error| refuse(format!("the built-in dictionary: {error}")));
    }
    if source.package.is_none() {
        return Err(refuse(format!(
            "`{name}.aff` is a file, which only a rule in a package can read"
        )));
    }
    Err(refuse(format!("`{name}.aff` is not in `{DICTIONARIES}`")))
}
