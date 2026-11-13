//! A vocabulary: the terms a project accepts and rejects, kept in the folder
//! `config/vocabularies/<name>/` of a package.
//!
//! A vocabulary holds every `accept.txt`, `reject.txt`, `vocab.yml` and
//! `vocab.yaml` below its folder and no other file. A line of a text list is
//! one term, after trimming; a blank line and a line that starts with a hash
//! and a space are skipped. A term is a plain text or a pattern, as a rule's
//! `exceptions` read it. The YAML files are read by [`yaml`].

mod yaml;

use std::path::Component::Normal;
use std::path::{Path, PathBuf};
use std::{fs, io};

use hakkiri::rule::{Exceptions, LoadProblem};

/// The folder of a package that holds its dictionaries, where a YAML
/// vocabulary looks for the affix file it names.
pub(crate) const DICTIONARIES: &str = "config/dictionaries";

/// The folder of a package that holds its vocabularies, under `config`.
const FOLDER: &str = "vocabularies";

/// The folder of a package's styles that holds the settings of Vale, such as
/// its vocabularies.
pub(crate) const SETTINGS_FOLDER: &str = "config";

/// What a vocabulary gives a rule.
#[derive(Debug)]
pub(crate) struct Vocabulary {
    /// The accepted terms, in the order the lists walk and the lines read.
    pub(crate) accepted: Vec<String>,
    /// The rejected terms, in the same order.
    pub(crate) rejected: Vec<String>,
    /// What holds back a match of an accepted term.
    pub(crate) exceptions: Exceptions,
}

/// Where the vocabulary `name` is in the package folder `package`, when that
/// folder holds one.
pub(crate) fn find(package: &Path, name: &str) -> Option<PathBuf> {
    let mut parts = Path::new(name).components();
    let is_name = matches!((parts.next(), parts.next()), (Some(Normal(_)), None));
    let folder = package.join(SETTINGS_FOLDER).join(FOLDER).join(name);
    (is_name && folder.is_dir()).then_some(folder)
}

/// What a file of a vocabulary is.
#[derive(Clone, Copy)]
enum Kind {
    Accept,
    Reject,
    Yaml,
}

/// Reads the vocabulary in `folder`, whose files `shown` names in the errors.
/// `dictionaries` are the folders that may hold the affix file a YAML file
/// names, in the order they are searched.
///
/// # Errors
///
/// Returns a problem for each list that cannot be read, each term that is not
/// a pattern and each YAML entry that is wrong, with the file and the line.
pub(crate) fn read(
    folder: &Path,
    shown: &Path,
    dictionaries: &[PathBuf],
) -> Result<Vocabulary, Vec<LoadProblem>> {
    let mut lists = Vec::new();
    if let Err(error) = walk(folder, &mut lists) {
        return Err(vec![problem(shown, None, error.to_string())]);
    }
    let mut accepted = Vec::new();
    let mut rejected = Vec::new();
    let mut problems = Vec::new();
    for (path, kind) in lists {
        let relative = path.strip_prefix(folder).unwrap_or(&path);
        let file = shown.join(relative);
        if matches!(kind, Kind::Yaml) {
            match yaml::read(&path, &file, dictionaries) {
                Ok(read) => {
                    accepted.extend(read.accepted);
                    rejected.extend(read.rejected);
                }
                Err(found) => problems.extend(found),
            }
            continue;
        }
        let accepts = matches!(kind, Kind::Accept);
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(error) => {
                problems.push(problem(&file, None, error.to_string()));
                continue;
            }
        };
        for (index, line) in text.lines().enumerate() {
            let term = line.trim();
            if term.is_empty() || term.starts_with("# ") {
                continue;
            }
            match Exceptions::of(&[term.to_owned()]) {
                Ok(_) if accepts => accepted.push(term.to_owned()),
                Ok(_) => rejected.push(term.to_owned()),
                Err(error) => problems.push(problem(
                    &file,
                    Some(index + 1),
                    format!("`{term}` is not a pattern: {error}"),
                )),
            }
        }
    }
    if !problems.is_empty() {
        return Err(problems);
    }
    match Exceptions::of(&accepted) {
        Ok(exceptions) => Ok(Vocabulary {
            accepted,
            rejected,
            exceptions,
        }),
        Err(error) => Err(vec![problem(shown, None, error.to_string())]),
    }
}

/// Each list under `folder`, in name order, and what it is.
fn walk(folder: &Path, lists: &mut Vec<(PathBuf, Kind)>) -> io::Result<()> {
    let mut entries = fs::read_dir(folder)?.collect::<io::Result<Vec<_>>>()?;
    entries.sort_by_key(fs::DirEntry::file_name);
    for entry in entries {
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            walk(&path, lists)?;
        } else if entry.file_name() == "accept.txt" {
            lists.push((path, Kind::Accept));
        } else if entry.file_name() == "reject.txt" {
            lists.push((path, Kind::Reject));
        } else if entry.file_name() == "vocab.yml" || entry.file_name() == "vocab.yaml" {
            lists.push((path, Kind::Yaml));
        }
    }
    Ok(())
}

fn problem(file: &Path, line: Option<usize>, message: String) -> LoadProblem {
    LoadProblem {
        file: file.display().to_string(),
        line,
        column: None,
        field: String::new(),
        message,
        unrunnable: false,
    }
}
