//! A vocabulary: the terms a project accepts and rejects, kept in the folder
//! `config/vocabularies/<name>/` of a package.
//!
//! A vocabulary holds every `accept.txt` and `reject.txt` below its folder and
//! no other file. A line of a list is one term, after trimming; a blank line
//! and a line that starts with a hash and a space are skipped. A term is a
//! plain text or a pattern, as a rule's `exceptions` read it.

use std::path::{Path, PathBuf};
use std::{fs, io};

use hakkiri::rule::{Exceptions, LoadProblem};

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
    let is_name = matches!(
        (parts.next(), parts.next()),
        (Some(std::path::Component::Normal(_)), None)
    );
    let folder = package.join(SETTINGS_FOLDER).join(FOLDER).join(name);
    (is_name && folder.is_dir()).then_some(folder)
}

/// Reads the vocabulary in `folder`, whose files `shown` names in the errors.
///
/// # Errors
///
/// Returns a problem for each list that cannot be read and each term that is
/// not a pattern, with the file and the line.
pub(crate) fn read(folder: &Path, shown: &Path) -> Result<Vocabulary, Vec<LoadProblem>> {
    let mut lists = Vec::new();
    if let Err(error) = walk(folder, &mut lists) {
        return Err(vec![problem(shown, None, error.to_string())]);
    }
    let mut accepted = Vec::new();
    let mut rejected = Vec::new();
    let mut problems = Vec::new();
    for (path, accepts) in lists {
        let relative = path.strip_prefix(folder).unwrap_or(&path);
        let file = shown.join(relative);
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

/// Each list under `folder`, in name order, and whether it accepts.
fn walk(folder: &Path, lists: &mut Vec<(PathBuf, bool)>) -> io::Result<()> {
    let mut entries = fs::read_dir(folder)?.collect::<io::Result<Vec<_>>>()?;
    entries.sort_by_key(fs::DirEntry::file_name);
    for entry in entries {
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            walk(&path, lists)?;
        } else if entry.file_name() == "accept.txt" {
            lists.push((path, true));
        } else if entry.file_name() == "reject.txt" {
            lists.push((path, false));
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
