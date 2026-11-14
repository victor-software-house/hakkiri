//! The `formats` key: which reader a file gets when its extension is not the
//! one that names it, or when no extension names one.

use std::collections::BTreeMap;
use std::path::Path;

use globset::GlobMatcher;
use hakkiri::Language;

use crate::config::standard_glob;

/// A key that holds one of these is a glob, as in Vale's `[formats]`.
const GLOB_MARKS: [char; 4] = ['*', '?', '[', '{'];

/// One `formats` entry: the language it gives, and its matcher when the key is
/// a glob.
#[derive(Debug)]
struct Entry {
    language: Language,
    glob: Option<GlobMatcher>,
}

/// The `formats` of a config, which map an extension, a file name or a glob to
/// a reader.
#[derive(Debug, Default)]
pub(crate) struct Formats {
    /// The entries by key, in the order of the keys as text.
    entries: BTreeMap<String, Entry>,
}

impl Formats {
    /// Adds the entry `key` with the reader `reader`, replacing an earlier
    /// entry of the same key.
    ///
    /// # Errors
    ///
    /// Returns why `key` is not a glob when it holds a mark of one and does not
    /// compile, and says the readers when `reader` names none.
    pub(crate) fn insert(&mut self, key: &str, reader: &str) -> Result<(), String> {
        let language = Language::from_reader(reader).ok_or_else(|| {
            let names = Language::ALL.map(Language::name);
            let (last, first) = names.split_last().unwrap_or((&"", &[]));
            format!(
                "`{reader}` is not a language hakkiri reads: use {} or {last}",
                first.join(", ")
            )
        })?;
        let glob = if key.contains(GLOB_MARKS) || key.contains('/') {
            let compiled = standard_glob(key).map_err(|error| format!("is not a glob: {error}"))?;
            Some(compiled.compile_matcher())
        } else {
            None
        };
        self.entries
            .insert(key.to_owned(), Entry { language, glob });
        Ok(())
    }

    /// The reader the entries give the file at `path`, written relative to the
    /// project root with `/` separators, as Vale's `[formats]` does.
    ///
    /// An entry named by the file's extension comes first. Then, in the order
    /// of the keys as text, an entry whose key is the file's name, or a glob
    /// that matches its path by the rules of gitignore.
    pub(crate) fn language_of(&self, path: &str) -> Option<Language> {
        let extension = Path::new(path)
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or_default();
        if let Some(entry) = self.entries.get(extension) {
            return Some(entry.language);
        }
        let name = path.rsplit('/').next().unwrap_or(path);
        self.entries.iter().find_map(|(key, entry)| {
            let hit = match &entry.glob {
                Some(glob) => glob.is_match(path),
                None => key == name,
            };
            hit.then_some(entry.language)
        })
    }
}

#[cfg(test)]
mod tests;
