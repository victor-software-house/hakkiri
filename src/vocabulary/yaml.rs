//! A YAML vocabulary, `vocab.yml` or `vocab.yaml`, as Vale reads it.
//!
//! The file holds `affix`, `accept` and `reject`. An entry of a list is a term,
//! a term mapped to nothing, or a term mapped to the options `regex`,
//! `ignorecase` and `flags`. Each entry gives the patterns that a line of a
//! text list would.
//!
//! The file parses through ctl-core's input, which refuses a key it does not
//! define and gives each fault its line. The shape of an entry is checked here
//! on the document's nodes, so one run reports every faulty entry, where Vale
//! stops at the first.

use std::path::{Component, Path, PathBuf};
use std::{fmt, fs};

use ctl_core::input::{FieldPath, Input};
use hakkiri::dictionary::Dictionary;
use hakkiri::rule::{Exceptions, LoadError, LoadProblem};
use serde::Deserialize;
use serde::de::{self, Deserializer, IgnoredAny, MapAccess, SeqAccess, Visitor};

use super::DICTIONARIES;

/// The affix file of the built-in dictionary, which needs no file.
const BUILTIN: &str = "en_US-web";

/// What a vocabulary file gives: its accepted and rejected patterns.
#[derive(Debug, Default)]
pub(super) struct Lists {
    pub(super) accepted: Vec<String>,
    pub(super) rejected: Vec<String>,
}

/// The file as written.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    #[serde(default)]
    affix: Option<String>,
    #[serde(default)]
    accept: Option<Vec<Node>>,
    #[serde(default)]
    reject: Option<Vec<Node>>,
}

/// A node of an entry, as far as its shape matters.
#[derive(Debug)]
enum Node {
    /// `null`, `~`, or nothing after a key.
    Null,
    Text(String),
    Bool(bool),
    /// A number. The text of the file holds how it was written.
    Number,
    List,
    Map(Vec<(String, Node)>),
}

impl<'de> Deserialize<'de> for Node {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(NodeVisitor)
    }
}

struct NodeVisitor;

impl<'de> Visitor<'de> for NodeVisitor {
    type Value = Node;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a term, or a term mapped to its options")
    }

    fn visit_unit<E: de::Error>(self) -> Result<Node, E> {
        Ok(Node::Null)
    }

    fn visit_bool<E: de::Error>(self, value: bool) -> Result<Node, E> {
        Ok(Node::Bool(value))
    }

    fn visit_i64<E: de::Error>(self, _: i64) -> Result<Node, E> {
        Ok(Node::Number)
    }

    fn visit_u64<E: de::Error>(self, _: u64) -> Result<Node, E> {
        Ok(Node::Number)
    }

    fn visit_f64<E: de::Error>(self, _: f64) -> Result<Node, E> {
        Ok(Node::Number)
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<Node, E> {
        Ok(Node::Text(value.to_owned()))
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Node, A::Error> {
        while seq.next_element::<IgnoredAny>()?.is_some() {}
        Ok(Node::List)
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Node, A::Error> {
        let mut pairs = Vec::new();
        while let Some(pair) = map.next_entry::<String, Node>()? {
            pairs.push(pair);
        }
        Ok(Node::Map(pairs))
    }
}

/// One entry, read.
struct Entry {
    term: String,
    regex: bool,
    ignorecase: bool,
    flags: String,
}

/// The options of an entry and their types.
const OPTIONS: [&str; 3] = ["flags", "ignorecase", "regex"];

/// Reads the vocabulary file at `path`, shown as `shown` in the problems.
/// `dictionaries` are the folders that may hold the affix file the file names,
/// in the order they are searched.
///
/// # Errors
///
/// Returns every problem found, each with the file and the line.
pub(super) fn read(
    path: &Path,
    shown: &Path,
    dictionaries: &[PathBuf],
) -> Result<Lists, Vec<LoadProblem>> {
    let name = shown.display().to_string();
    let text = fs::read_to_string(path).map_err(|error| {
        vec![LoadProblem {
            file: name.clone(),
            line: None,
            column: None,
            field: String::new(),
            message: error.to_string(),
            unrunnable: false,
        }]
    })?;
    let input = Input::new(&name, text);
    let file: File = input
        .parse()
        .map_err(|error| LoadError::from(error).problems)?;
    let mut reading = Reading {
        input: &input,
        problems: Vec::new(),
    };
    let expander = reading.expander(file.affix.as_deref().unwrap_or_default(), dictionaries);
    let mut lists = Lists::default();
    for (key, nodes, into) in [
        ("accept", &file.accept, &mut lists.accepted),
        ("reject", &file.reject, &mut lists.rejected),
    ] {
        for (index, node) in nodes.iter().flatten().enumerate() {
            let at = FieldPath::root().key(key).index(index);
            if let Some(entry) = reading.entry(&at, node)
                && let Some(patterns) = reading.patterns(&at, &entry, expander.as_ref())
            {
                into.extend(patterns);
            }
        }
    }
    if reading.problems.is_empty() {
        Ok(lists)
    } else {
        Err(reading.problems)
    }
}

/// The input and the problems found in it so far.
struct Reading<'a> {
    input: &'a Input,
    problems: Vec<LoadProblem>,
}

impl Reading<'_> {
    fn problem(&mut self, at: &FieldPath, message: String) {
        let place = self.input.place(at);
        self.problems.push(LoadProblem {
            file: self.input.name().to_owned(),
            line: place.map(|place| place.line),
            column: place.map(|place| place.column),
            field: at.to_string(),
            message,
            unrunnable: false,
        });
    }

    /// A problem on the key at the end of `at`, which is where a fault in the
    /// key belongs.
    fn key_problem(&mut self, at: &FieldPath, message: String) {
        let place = self.input.place_key(at);
        self.problems.push(LoadProblem {
            file: self.input.name().to_owned(),
            line: place.map(|place| place.line),
            column: place.map(|place| place.column),
            field: at.to_string(),
            message,
            unrunnable: false,
        });
    }

    /// The text of the scalar written at `at`, as the file spells it: a number
    /// or a word such as `null` reaches here as a value, and its spelling is
    /// the term.
    fn written(&self, at: &FieldPath) -> String {
        let Some(place) = self.input.place(at) else {
            return String::new();
        };
        let line = self
            .input
            .text()
            .lines()
            .nth(place.line.saturating_sub(1))
            .unwrap_or_default();
        let from: String = line.chars().skip(place.column.saturating_sub(1)).collect();
        let end = from
            .find([',', ']', '}'])
            .into_iter()
            .chain(from.find(" #"))
            .min()
            .unwrap_or(from.len());
        from.get(..end).unwrap_or_default().trim().to_owned()
    }

    /// The text of a scalar node at `at`, or `None` when the node is no scalar.
    fn scalar(&self, at: &FieldPath, node: &Node) -> Option<String> {
        match node {
            Node::Text(text) => Some(text.clone()),
            Node::Null | Node::Bool(_) | Node::Number => Some(self.written(at)),
            Node::List | Node::Map(_) => None,
        }
    }

    /// The entry the node at `at` stands for, or `None` after a problem.
    fn entry(&mut self, at: &FieldPath, node: &Node) -> Option<Entry> {
        let shape = "an entry is a term, or a term mapped to its options".to_owned();
        let mut entry = Entry {
            term: String::new(),
            regex: false,
            ignorecase: false,
            flags: String::new(),
        };
        let pairs = match node {
            Node::List => {
                self.problem(at, shape);
                return None;
            }
            Node::Map(pairs) => pairs,
            // Vale skips an entry that is null, such as `-` alone.
            Node::Null => return None,
            scalar @ (Node::Text(_) | Node::Bool(_) | Node::Number) => {
                entry.term = self.scalar(at, scalar).unwrap_or_default();
                return Some(entry);
            }
        };
        let [(term, value)] = pairs.as_slice() else {
            self.problem(at, shape);
            return None;
        };
        entry.term.clone_from(term);
        let options = at.clone().key(term);
        match value {
            Node::Null => Some(entry),
            Node::List => {
                self.problem(&options, format!("`{term}` should map to its options"));
                None
            }
            Node::Map(given) => {
                let before = self.problems.len();
                for (key, value) in given {
                    let at = options.clone().key(key);
                    match (key.as_str(), value) {
                        ("regex" | "ignorecase", value) => match truth(value) {
                            Some(on) if key == "regex" => entry.regex = on,
                            Some(on) => entry.ignorecase = on,
                            None => self.problem(&at, format!("`{key}` is `true` or `false`")),
                        },
                        ("flags", Node::Null) => {}
                        ("flags", value) => match self.scalar(&at, value) {
                            Some(flags) => entry.flags = flags,
                            None => self.problem(&at, "`flags` is a string".to_owned()),
                        },
                        _ => self.key_problem(
                            &at,
                            format!(
                                "`{key}` is not a vocabulary option; expected {}",
                                OPTIONS.join(", ")
                            ),
                        ),
                    }
                }
                (self.problems.len() == before).then_some(entry)
            }
            scalar @ (Node::Text(_) | Node::Bool(_) | Node::Number) => {
                let value = self.scalar(&options, scalar).unwrap_or_default();
                self.problem(
                    &options,
                    format!(
                        "`{term}` maps to `{value}`, but a vocabulary entry maps to options; \
                         for a replacement, use a substitution rule"
                    ),
                );
                None
            }
        }
    }

    /// The patterns an entry stands for, as the lines of a text list would, or
    /// `None` after a problem.
    fn patterns(
        &mut self,
        at: &FieldPath,
        entry: &Entry,
        expander: Option<&Expander>,
    ) -> Option<Vec<String>> {
        let term = entry.term.trim();
        if term.is_empty() {
            self.problem(at, "an entry needs a term".to_owned());
            return None;
        }
        let mut patterns = if entry.regex && !entry.flags.is_empty() {
            self.problem(at, format!("`{term}` cannot have both `regex` and `flags`"));
            return None;
        } else if entry.regex {
            if let Err(error) = Exceptions::of(&[term.to_owned()]) {
                self.problem(at, format!("`{term}` is not a pattern: {error}"));
                return None;
            }
            vec![term.to_owned()]
        } else if entry.flags.is_empty() {
            vec![literal(term, entry.ignorecase)]
        } else {
            // A problem with the affix file was reported when it was read.
            let words = match expander?.expand(term, &entry.flags) {
                Ok(words) => words,
                Err(message) => {
                    self.problem(at, format!("`{term}`: {message}"));
                    return None;
                }
            };
            words
                .iter()
                .map(|word| literal(word, entry.ignorecase))
                .collect()
        };
        if entry.ignorecase {
            for pattern in &mut patterns {
                pattern.insert_str(0, "(?i)");
            }
        }
        Some(patterns)
    }

    /// The dictionary that expands the flags of the file, from the affix file
    /// `affix` names: in the `dictionaries` folders, in order, or the built-in
    /// one. A problem with it is reported once, and no entry expands then.
    fn expander(&mut self, affix: &str, dictionaries: &[PathBuf]) -> Option<Expander> {
        let at = FieldPath::root().key("affix");
        let named = !affix.is_empty();
        if named && !is_file_name(affix) {
            self.problem(&at, format!("`{affix}` is not a file name"));
            return None;
        }
        let found = named
            .then(|| {
                dictionaries
                    .iter()
                    .map(|folder| folder.join(format!("{affix}.aff")))
                    .find(|path| path.is_file())
            })
            .flatten();
        match found {
            Some(path) => {
                let read = fs::read(&path).map_err(|error| error.to_string());
                match read.and_then(|aff| {
                    Dictionary::parse(&aff, b"0\n").map_err(|error| error.to_string())
                }) {
                    Ok(dictionary) => Some(Expander::Package(Box::new(dictionary))),
                    Err(message) => {
                        self.problem(&at, format!("`{affix}.aff`: {message}"));
                        None
                    }
                }
            }
            None if !named || affix == BUILTIN => match Dictionary::english() {
                Ok(dictionary) => Some(Expander::Builtin(dictionary)),
                Err(error) => {
                    self.problem(&at, format!("the built-in dictionary: {error}"));
                    None
                }
            },
            None => {
                self.problem(
                    &at,
                    format!("`{affix}.aff` is not in the `{DICTIONARIES}` folder of any package"),
                );
                None
            }
        }
    }
}

/// A dictionary that gives the forms of a word for its affix flags.
enum Expander {
    Builtin(&'static Dictionary),
    Package(Box<Dictionary>),
}

impl Expander {
    fn expand(&self, word: &str, flags: &str) -> Result<Vec<String>, String> {
        match self {
            Self::Builtin(dictionary) => dictionary.expand(word, flags),
            Self::Package(dictionary) => dictionary.expand(word, flags),
        }
        .map_err(|error| error.to_string())
    }
}

/// The truth a node gives an option such as `regex`: `true` or `false`, the
/// words YAML 1.1 reads as booleans, which Vale's YAML reader takes for a
/// boolean option, and nothing, which leaves the option off.
fn truth(node: &Node) -> Option<bool> {
    match node {
        Node::Null => Some(false),
        Node::Bool(on) => Some(*on),
        Node::Text(word) => match word.as_str() {
            "y" | "Y" | "yes" | "Yes" | "YES" | "on" | "On" | "ON" => Some(true),
            "n" | "N" | "no" | "No" | "NO" | "off" | "Off" | "OFF" => Some(false),
            _ => None,
        },
        Node::Number | Node::List | Node::Map(_) => None,
    }
}

/// Whether `name` is one file name, with no folder in it.
fn is_file_name(name: &str) -> bool {
    let mut parts = Path::new(name).components();
    matches!(
        (parts.next(), parts.next()),
        (Some(Component::Normal(_)), None)
    )
}

/// The pattern that matches `term` as written. A term whose only special
/// character is a period stays as it is, which a text list reads as plain
/// text with a literal period, as Vale does. With `ignorecase` the pattern
/// starts with `(?i)`, which makes a text list read it as a regular
/// expression, so the period is quoted as well.
fn literal(term: &str, ignorecase: bool) -> String {
    const SPECIAL: &str = "\\.+*?()|[]{}^$";
    let bare = term.replace('.', "");
    if !ignorecase && bare.chars().all(|character| !SPECIAL.contains(character)) {
        return term.to_owned();
    }
    let mut quoted = String::with_capacity(term.len() * 2);
    for character in term.chars() {
        if SPECIAL.contains(character) {
            quoted.push('\\');
        }
        quoted.push(character);
    }
    quoted
}
