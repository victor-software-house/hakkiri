//! The project config, `.ctl/hakkiri.yaml`: the packages a project loads, the
//! lowest level it reports, and the styles and rule settings of each set of
//! files.
//!
//! [`load`] reads the file once, loads every style folder of every package, and
//! refuses what it cannot run, with the file and line of the fault.
//! A package may ship a settings file in the same format, and the settings of
//! each package apply before the project's own.
//! [`Project::plan`] then says which rules run on a file.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::{fmt, fs, io};

use ctl_core::input::Input;
use garde::Validate;
use globset::{GlobBuilder, GlobMatcher};
use hakkiri::Language;
use hakkiri::pattern::Pattern;
use hakkiri::rule::{
    Exceptions, Level, LoadError, LoadProblem, Rule, Unrunnable, read_style, vocabulary_rules,
};
use serde::Deserialize;
use serde::de::{Deserializer, MapAccess, Visitor};

use crate::cli::{FailLevel, OnUnknown, OnUnmatched};
use crate::formats::Formats;
use crate::package;
use crate::vocabulary::{self, SETTINGS_FOLDER, Vocabulary};

/// Where the config is when no flag or variable names another file.
pub(crate) const DEFAULT: &str = ".ctl/hakkiri.yaml";

/// The most packages one chain may hold: a package the project lists, the
/// packages it lists, and so on.
const MAX_DEPTH: usize = 10;

/// The levels a config names, in the words it writes them.
const LEVELS: &str = "suggestion, warning or error";

/// The level a config names, `None` for any other word.
fn level_named(name: &str) -> Option<Level> {
    match name {
        "suggestion" => Some(Level::Suggestion),
        "warning" => Some(Level::Warning),
        "error" => Some(Level::Error),
        _ => None,
    }
}

/// A mapping read in the order it was written, which the plain map types lose.
#[derive(Debug)]
struct Ordered<V>(Vec<(String, V)>);

impl<V> Default for Ordered<V> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<'de, V: Deserialize<'de>> Deserialize<'de> for Ordered<V> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Entries<V>(PhantomData<V>);

        impl<'de, V: Deserialize<'de>> Visitor<'de> for Entries<V> {
            type Value = Ordered<V>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a mapping")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Ordered<V>, A::Error> {
                let mut entries = Vec::new();
                while let Some(entry) = map.next_entry::<String, V>()? {
                    entries.push(entry);
                }
                Ok(Ordered(entries))
            }
        }

        deserializer.deserialize_map(Entries(PhantomData))
    }
}

/// One setting under a glob: the list of `styles`, or the word that sets a
/// rule's level.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Setting {
    /// A list of style names.
    List(Vec<String>),
    /// A level, `on` or `off`.
    Word(String),
}

/// The folders of each vocabulary the config names, for the check that runs
/// after the packages load. Before they load it is empty, and no vocabulary is
/// refused.
#[derive(Debug, Default)]
struct Known {
    holders: Option<BTreeMap<String, Vec<PathBuf>>>,
}

/// The file as written.
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
#[garde(context(Known))]
struct Raw {
    #[serde(default)]
    #[garde(custom(level_ok))]
    level: Option<String>,
    #[serde(default)]
    #[garde(dive)]
    packages: Vec<Entry>,
    #[serde(default)]
    #[garde(inner(inner(custom(vocab_known))))]
    vocab: Option<Vec<String>>,
    #[serde(default)]
    #[garde(skip)]
    files: Ordered<Ordered<Setting>>,
    /// The options of a run, which only the project's config sets.
    #[serde(default)]
    #[garde(skip)]
    fail_level: Option<FailLevel>,
    #[serde(default)]
    #[garde(skip)]
    gitignore: Option<bool>,
    #[serde(default)]
    #[garde(skip)]
    on_unmatched: Option<OnUnmatched>,
    #[serde(default)]
    #[garde(skip)]
    on_unknown: Option<OnUnknown>,
    /// The reader of each extension, file name or glob.
    #[serde(default)]
    #[garde(skip)]
    formats: BTreeMap<String, String>,
}

/// One entry of `packages`.
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
#[garde(context(Known))]
#[garde(allow_unvalidated)]
#[garde(custom(entry_ok))]
struct Entry {
    path: Option<String>,
    url: Option<String>,
    sha256: Option<String>,
}

#[expect(
    clippy::ref_option,
    reason = "garde calls a custom rule with a reference to the field."
)]
fn level_ok(value: &Option<String>, _: &Known) -> garde::Result {
    match value.as_deref() {
        Some(name) if level_named(name).is_none() => Err(garde::Error::new(format!(
            "`{name}` is not a level: use {LEVELS}"
        ))),
        _ => Ok(()),
    }
}

fn entry_ok(entry: &Entry, _: &Known) -> garde::Result {
    match (&entry.path, &entry.url, &entry.sha256) {
        (Some(_), Some(_), _) => Err(garde::Error::new(
            "an entry holds a `path` or a `url`, never both",
        )),
        (Some(_), None, Some(_)) => Err(garde::Error::new(
            "`sha256` belongs to a `url`, and this entry holds a `path`",
        )),
        (None, Some(_), None) => Err(garde::Error::new("a `url` needs its `sha256`")),
        (None, Some(url), Some(sha256)) => url_ok(url, sha256),
        (None, None, _) => Err(garde::Error::new(
            "an entry needs a `path` to a folder of styles",
        )),
        (Some(_), None, None) => Ok(()),
    }
}

/// Refuses a name of the top-level `vocab` that no package holds, or that two
/// packages hold, once the packages are loaded.
fn vocab_known(name: &str, known: &Known) -> garde::Result {
    let Some(holders) = &known.holders else {
        return Ok(());
    };
    match holders.get(name).map(Vec::as_slice) {
        Some([_]) => Ok(()),
        Some([first, second, ..]) => Err(garde::Error::new(format!(
            "the vocabulary `{name}` is in both `{}` and `{}`",
            first.display(),
            second.display()
        ))),
        _ => Err(garde::Error::new(format!(
            "`{name}` is not a vocabulary in any package"
        ))),
    }
}

/// Whether a package `url` and its `sha256` have the shape a fetch needs.
fn url_ok(url: &str, sha256: &str) -> garde::Result {
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err(garde::Error::new(
            "a package `url` starts with `https://` or `http://`",
        ));
    }
    if sha256.len() != 64
        || !sha256
            .bytes()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
    {
        return Err(garde::Error::new("`sha256` is 64 lowercase hex digits"));
    }
    Ok(())
}

/// What a rule key under a glob does to the rule.
#[derive(Clone, Copy, Debug)]
enum Choice {
    /// The rule does not run.
    Off,
    /// The rule runs, and its alerts keep the level of its rule file unless
    /// a level was set before.
    On,
    /// The rule runs, and its alerts take this level.
    Level(Level),
}

/// One glob of `files` and what it sets.
#[derive(Debug)]
struct Glob {
    /// The glob as written, which two config files share when they write it
    /// alike.
    pattern: String,
    matcher: GlobMatcher,
    styles: Option<Vec<String>>,
    /// The vocabularies the glob names, for the files it matches.
    vocab: Vec<String>,
    /// Whether the glob wrote an empty `vocab` list, which drops the lists of
    /// the earlier globs of the same text.
    clears_vocab: bool,
    /// The patterns whose matches are hidden as blocks from the files the glob
    /// matches.
    block_ignores: Vec<Pattern>,
    /// The patterns whose matches are hidden as inline text.
    token_ignores: Vec<Pattern>,
    choices: Vec<(String, Choice)>,
}

/// A config with its packages loaded.
#[derive(Debug)]
pub(crate) struct Project {
    /// Every rule of every style, in package order and then file name order.
    rules: Vec<Rule>,
    /// Where each style's rules are in `rules`.
    styles: BTreeMap<String, Vec<usize>>,
    /// The rules that are valid Vale and that hakkiri cannot run.
    unrunnable: Vec<Held>,
    globs: Vec<Glob>,
    /// The vocabularies the config names, by name.
    vocabularies: BTreeMap<String, Vocabulary>,
    /// The lowest level an alert reports at.
    pub(crate) level: Option<Level>,
    /// The config's `fail_level`.
    pub(crate) fail_level: Option<FailLevel>,
    /// The config's `gitignore`.
    pub(crate) gitignore: Option<bool>,
    /// The config's `on_unmatched`.
    pub(crate) on_unmatched: Option<OnUnmatched>,
    /// The config's `on_unknown`.
    pub(crate) on_unknown: Option<OnUnknown>,
    /// The `formats` of the packages and the project, a later one winning for
    /// the same key.
    formats: Formats,
    /// The package folders, which a walk leaves alone.
    pub(crate) packages: Vec<PathBuf>,
    /// The project root: the folder that holds the `.ctl` folder of the
    /// config, with symlinks resolved.
    root: PathBuf,
}

/// The rules that run on one file, and the levels the config sets for some.
#[derive(Debug)]
pub(crate) struct Plan<'a> {
    /// The rules to run: those of the listed styles in the order listed, then
    /// those a config enables by name in the order their styles load.
    pub(crate) rules: Vec<&'a Rule>,
    /// The level an alert of a rule takes, by rule name.
    pub(crate) levels: HashMap<&'a str, Level>,
    /// The rules the config runs on the file that hakkiri cannot run: each one
    /// fails the run until the config turns it `off`.
    pub(crate) unrunnable: Vec<&'a Unrunnable>,
    /// The exceptions of the vocabularies that the globs matching the file
    /// name, one for each, in the order the globs are written.
    pub(crate) accepted: Vec<&'a Exceptions>,
    /// The block patterns of the globs matching the file, in the order the
    /// globs are written.
    pub(crate) block_ignores: Vec<&'a Pattern>,
    /// The token patterns of the globs matching the file, in the same order.
    pub(crate) token_ignores: Vec<&'a Pattern>,
}

/// Whether the resolved directory `real` is under `root` or holds it.
pub(crate) fn leads_to(root: &Path, real: &Path) -> bool {
    real.starts_with(root) || root.starts_with(real)
}

impl Project {
    /// The project root.
    pub(crate) fn root(&self) -> &Path {
        &self.root
    }

    /// Whether the resolved directory `real` is under the project root or
    /// holds it, so a walk of it can reach a file under the root.
    pub(crate) fn leads_to_root(&self, real: &Path) -> bool {
        leads_to(&self.root, real)
    }

    /// The language a `formats` entry gives the file at `path`, written
    /// relative to the project root with `/` separators, when one does.
    pub(crate) fn mapped_language(&self, path: &str) -> Option<Language> {
        self.formats.language_of(path)
    }

    /// Whether some glob matches `path`, written relative to the project root
    /// with `/` separators, whatever rules the globs give it.
    pub(crate) fn selects(&self, path: &str) -> bool {
        self.globs.iter().any(|glob| glob.matcher.is_match(path))
    }

    /// Whether `argument` can lead to a file under the project root: a file
    /// that is under it, or a directory that is under it or holds it.
    pub(crate) fn reaches(&self, argument: &Path, directory: bool) -> bool {
        if directory {
            fs::canonicalize(argument).is_ok_and(|real| self.leads_to_root(&real))
        } else {
            self.relative(argument).is_some()
        }
    }

    /// The path of the file `file` relative to the project root, with `/`
    /// separators, or `None` when it is outside the root or cannot be resolved.
    ///
    /// The folders of `file` are resolved through symlinks, as the root is, but
    /// its own name is kept: a symlink to a file is matched by the name it is
    /// reached by, as Vale matches it.
    pub(crate) fn relative(&self, file: &Path) -> Option<String> {
        let absolute = std::path::absolute(file).ok()?;
        let real = fs::canonicalize(absolute.parent()?)
            .ok()?
            .join(absolute.file_name()?);
        let inside = real.strip_prefix(&self.root).ok()?;
        Some(
            inside
                .components()
                .map(|part| part.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/"),
        )
    }

    /// The rules for `path`, written relative to the project root with `/`
    /// separators, or `None` when no glob matches it or it has no rule to run.
    ///
    /// The globs that match apply in the order written, a later one winning for
    /// the same key.
    /// A `styles` list replaces the earlier one, and an empty list also drops
    /// every rule setting before it, as Vale's empty `BasedOnStyles` does.
    /// A rule runs when a listed style holds it and the config does not turn it
    /// off, or when the config turns it on or sets its level, whatever style
    /// holds it. A level once set stays when a later glob turns the rule off or
    /// on, as in Vale.
    /// A rule hakkiri cannot run is held to the same test, and each one that
    /// passes it is in [`Plan::unrunnable`].
    pub(crate) fn plan(&self, path: &str) -> Option<Plan<'_>> {
        let mut matched = false;
        let mut styles: &[String] = &[];
        let mut switches: HashMap<String, bool> = HashMap::new();
        let mut set_levels: HashMap<&str, Level> = HashMap::new();
        let mut named: Vec<&str> = Vec::new();
        let mut block_ignores: Vec<&Pattern> = Vec::new();
        let mut token_ignores: Vec<&Pattern> = Vec::new();
        for glob in self.globs.iter().filter(|glob| glob.matcher.is_match(path)) {
            matched = true;
            block_ignores.extend(&glob.block_ignores);
            token_ignores.extend(&glob.token_ignores);
            for name in &glob.vocab {
                if !named.contains(&name.as_str()) {
                    named.push(name);
                }
            }
            if let Some(listed) = &glob.styles {
                styles = listed;
                if listed.is_empty() {
                    switches.clear();
                    set_levels.clear();
                }
            }
            for (name, choice) in &glob.choices {
                match choice {
                    Choice::Off => {
                        switches.insert(name.clone(), false);
                    }
                    Choice::On => {
                        switches.insert(name.clone(), true);
                    }
                    Choice::Level(level) => {
                        switches.insert(name.clone(), true);
                        set_levels.insert(name.as_str(), *level);
                    }
                }
            }
            // A glob that names a vocabulary runs the rules of that vocabulary
            // on its files, after the glob's own settings, as in Vale.
            set_vocabulary_rules(&mut switches, &glob.vocab);
        }
        if !matched {
            return None;
        }
        let mut rules = Vec::new();
        let mut levels = HashMap::new();
        let mut listed = HashSet::new();
        for index in styles
            .iter()
            .filter_map(|style| self.styles.get(style))
            .flatten()
        {
            listed.insert(*index);
            let rule = self.rules.get(*index)?;
            if switches.get(rule.name.as_str()) != Some(&false) {
                rules.push(rule);
                if let Some(level) = set_levels.get(rule.name.as_str()) {
                    levels.insert(rule.name.as_str(), *level);
                }
            }
        }
        for (index, rule) in self.rules.iter().enumerate() {
            if listed.contains(&index) {
                continue;
            }
            if switches.get(rule.name.as_str()) == Some(&true) {
                rules.push(rule);
                if let Some(level) = set_levels.get(rule.name.as_str()) {
                    levels.insert(rule.name.as_str(), *level);
                }
            }
        }
        let unrunnable: Vec<&Unrunnable> = self
            .unrunnable
            .iter()
            .filter(|held| {
                let on = switches.get(held.rule.name.as_str());
                if styles.contains(&held.style) {
                    on != Some(&false)
                } else {
                    on == Some(&true)
                }
            })
            .map(|held| &held.rule)
            .collect();
        let accepted = named
            .iter()
            .filter_map(|name| self.vocabularies.get(*name))
            .map(|vocabulary| &vocabulary.exceptions)
            .collect();
        (!rules.is_empty() || !unrunnable.is_empty()).then_some(Plan {
            rules,
            levels,
            unrunnable,
            accepted,
            block_ignores,
            token_ignores,
        })
    }
}

/// Turns the rules of each vocabulary in `names` on.
fn set_vocabulary_rules<S: AsRef<str>>(switches: &mut HashMap<String, bool>, names: &[S]) {
    for name in names {
        for rule in ["Terms", "Avoid"] {
            switches.insert(format!("Vale.{}.{rule}", name.as_ref()), true);
        }
    }
}

/// The 1-based line of the first line at or after line `from` that is the key
/// `key`, written bare or quoted and perhaps after a list dash, or `None` when
/// there is none.
fn key_line(text: &str, key: &str, from: usize) -> Option<usize> {
    text.lines()
        .enumerate()
        .skip(from.saturating_sub(1))
        .find_map(|(index, line)| {
            let item = line.trim_start();
            let item = item.strip_prefix("- ").map_or(item, str::trim_start);
            let rest = item.trim_start_matches(['\'', '"']);
            let after = rest.strip_prefix(key)?;
            after
                .trim_start_matches(['\'', '"'])
                .starts_with(':')
                .then_some(index + 1)
        })
}

/// What a config fault names: the file, and the line when the text shows it.
struct Faults<'a> {
    file: &'a str,
    problems: Vec<LoadProblem>,
}

impl Faults<'_> {
    fn add(&mut self, line: Option<usize>, field: &str, message: String) {
        self.problems.push(LoadProblem {
            file: self.file.to_owned(),
            line,
            column: None,
            field: field.to_owned(),
            message,
            unrunnable: false,
        });
    }
}

/// The rules of every package, found by style.
#[derive(Default)]
struct Loaded {
    /// Every rule of every style, in package order and then file name order.
    rules: Vec<Rule>,
    /// Where each style's rules are in `rules`.
    styles: BTreeMap<String, Vec<usize>>,
    /// The real path of each package folder.
    packages: Vec<PathBuf>,
    /// The faults in the rule names of each style, which are errors only once
    /// a glob enables the style.
    names: BTreeMap<String, Vec<LoadProblem>>,
    /// The rules that are valid Vale and that hakkiri cannot run, with the
    /// style that holds each.
    unrunnable: Vec<Held>,
    /// The vocabularies the config names, by name.
    vocabularies: BTreeMap<String, Vocabulary>,
    /// The folders that hold each vocabulary the config names, as the config
    /// writes them.
    holders: BTreeMap<String, Vec<PathBuf>>,
}

/// A package folder that was found, and where its entry is written.
struct Found {
    /// The config file that holds the entry, as a fault names it.
    origin: String,
    written: PathBuf,
    folder: PathBuf,
    key: &'static str,
    line: Option<usize>,
}

/// A rule hakkiri cannot run, and the style that holds it.
#[derive(Debug)]
struct Held {
    style: String,
    rule: Unrunnable,
}

/// The folder that holds the `.ctl` folder of the config file `config`, or the
/// folder of `config` when it is not in a `.ctl` folder, with symlinks
/// resolved.
fn project_root(config: &Path) -> io::Result<PathBuf> {
    let real = fs::canonicalize(config)?;
    let folder = real
        .parent()
        .ok_or_else(|| io::Error::other("the config file has no folder"))?;
    let root = match folder.file_name() {
        Some(name) if name == ".ctl" => folder.parent().unwrap_or(folder),
        _ => folder,
    };
    Ok(root.to_path_buf())
}

/// One config file whose settings apply: a package's, or the project's.
#[derive(Debug)]
struct Layer {
    /// How a fault names the file.
    name: String,
    text: String,
    raw: Raw,
}

/// The packages of a config, and the packages they list, with the settings
/// files they ship.
///
/// The packages a package lists come before it, in the order written, so its
/// own settings override theirs.
struct Walk<'a> {
    root: &'a Path,
    cache_dir: Option<&'a Path>,
    /// The package folders, in load order.
    found: Vec<Found>,
    /// The settings files of the packages, in load order.
    layers: Vec<Layer>,
    /// The real path of each package folder.
    packages: Vec<PathBuf>,
    /// The packages already loaded: a `sha256` for a download, a real path for
    /// a folder, so a package named twice loads once.
    done: HashSet<String>,
    problems: Vec<LoadProblem>,
}

impl<'a> Walk<'a> {
    fn new(root: &'a Path, cache_dir: Option<&'a Path>) -> Self {
        Self {
            root,
            cache_dir,
            found: Vec::new(),
            layers: Vec::new(),
            packages: Vec::new(),
            done: HashSet::new(),
            problems: Vec::new(),
        }
    }

    /// Finds each package `entries` names, each `path` read relative to the
    /// project root and each `url` fetched into the cache folder, then the
    /// packages that package lists.
    ///
    /// `file` is the config that holds `entries`, as a fault names it, and
    /// `depth` the packages between it and the project.
    fn entries(&mut self, entries: &[Entry], text: &str, file: &str, depth: usize) {
        let mut faults = Faults {
            file,
            problems: Vec::new(),
        };
        let mut from = key_line(text, "packages", 1).unwrap_or(1);
        for entry in entries {
            let key = if entry.url.is_some() { "url" } else { "path" };
            let line = key_line(text, key, from);
            from = line.map_or(from, |line| line + 1);
            if depth > 0 && entry.path.is_some() {
                faults.add(
                    line,
                    key,
                    "is not allowed in a package's settings: pin the package by `url` and `sha256`"
                        .to_owned(),
                );
                continue;
            }
            if depth >= MAX_DEPTH {
                faults.add(
                    line,
                    key,
                    format!("is over the limit of {MAX_DEPTH} packages in a chain"),
                );
                continue;
            }
            let (written, folder, hash) = match (&entry.path, &entry.url, &entry.sha256) {
                (Some(path), _, _) => (PathBuf::from(path), self.root.join(path), None),
                (None, Some(url), Some(sha256)) => {
                    match package::fetch(self.cache_dir, url, sha256) {
                        Ok(folder) => (PathBuf::from(url), folder, Some(sha256.clone())),
                        Err(message) => {
                            faults.add(line, key, message);
                            continue;
                        }
                    }
                }
                (None, _, _) => continue,
            };
            let real = match fs::canonicalize(&folder) {
                Ok(real) => real,
                Err(error) => {
                    faults.add(line, key, format!("`{}`: {error}", written.display()));
                    continue;
                }
            };
            let is_download = hash.is_some();
            if !self
                .done
                .insert(hash.unwrap_or_else(|| real.display().to_string()))
            {
                continue;
            }
            let top = if is_download {
                written.clone()
            } else {
                written.parent().map(Path::to_path_buf).unwrap_or_default()
            };
            self.settings(&real, &top, depth);
            self.packages.push(real);
            self.found.push(Found {
                origin: file.to_owned(),
                written,
                folder,
                key,
                line,
            });
        }
        self.problems.extend(faults.problems);
    }

    /// The folder that holds the package whose styles folder is `styles`: its
    /// parent, when `styles` is named `styles`.
    ///
    /// The project's own config is not a package's: its `.ctl` folder sits in
    /// the project root, so a parent that is the project root, or holds it, is
    /// no package top.
    fn top_of(&self, styles: &Path) -> Option<PathBuf> {
        let top = styles.parent()?;
        (styles.file_name()? == "styles" && top != self.root && !self.root.starts_with(top))
            .then(|| top.to_path_buf())
    }

    /// Reads the settings file of the package whose styles folder is `styles`,
    /// when its top folder holds one, and walks the packages it lists.
    ///
    /// `shown` is the top folder as the config writes it.
    fn settings(&mut self, styles: &Path, shown: &Path, depth: usize) {
        let Some(top) = self.top_of(styles) else {
            return;
        };
        let path = top.join(DEFAULT);
        if !path.is_file() {
            return;
        }
        let name = shown.join(DEFAULT).display().to_string();
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(error) => {
                self.problems.push(LoadProblem {
                    file: name,
                    line: None,
                    column: None,
                    field: String::new(),
                    message: error.to_string(),
                    unrunnable: false,
                });
                return;
            }
        };
        let raw: Raw = match Input::new(&name, &text).load() {
            Ok(raw) => raw,
            Err(error) => {
                self.problems.extend(LoadError::from(error).problems);
                return;
            }
        };
        let mut faults = Faults {
            file: &name,
            problems: Vec::new(),
        };
        for (key, set) in [
            ("fail_level", raw.fail_level.is_some()),
            ("gitignore", raw.gitignore.is_some()),
            ("on_unmatched", raw.on_unmatched.is_some()),
            ("on_unknown", raw.on_unknown.is_some()),
        ] {
            if set {
                faults.add(
                    key_line(&text, key, 1),
                    key,
                    "belongs to the project's config, not a package's settings".to_owned(),
                );
            }
        }
        self.problems.append(&mut faults.problems);
        self.entries(&raw.packages, &text, &name, depth + 1);
        self.layers.push(Layer { name, text, raw });
    }
}

/// Reads the config at `path`, and loads the style folders of its packages. A
/// package pinned by `url` is fetched into `cache_dir`, or into the per-user
/// cache folder when there is none, unless that folder already holds it.
///
/// A package whose top folder holds `.ctl/hakkiri.yaml` ships settings in the
/// format of the config: they apply ahead of the project's, and the packages
/// that file lists are fetched as the project's are.
///
/// # Errors
///
/// Returns every fault found.
/// A fault is a file that cannot be read, a key the config does not define, a
/// package that cannot be read or that repeats a style, or a rule that does not
/// load. It is also a glob, a style or a rule name that names nothing, and a
/// chain of packages over [`MAX_DEPTH`].
pub(crate) fn load(path: &Path, cache_dir: Option<&Path>) -> Result<Project, LoadError> {
    let input = Input::read(path).map_err(LoadError::from)?;
    let raw: Raw = input.load().map_err(LoadError::from)?;
    let (fail_level, gitignore, on_unmatched, on_unknown) = (
        raw.fail_level,
        raw.gitignore,
        raw.on_unmatched,
        raw.on_unknown,
    );
    let text = input.text();
    let mut faults = Faults {
        file: input.name(),
        problems: Vec::new(),
    };
    let root = match project_root(path) {
        Ok(root) => root,
        Err(error) => {
            faults.add(None, "config", format!("`{}`: {error}", path.display()));
            return Err(LoadError {
                problems: faults.problems,
            });
        }
    };
    let mut walk = Walk::new(&root, cache_dir);
    walk.entries(&raw.packages, text, input.name(), 0);
    faults.problems.append(&mut walk.problems);
    let mut layers = walk.layers;
    layers.push(Layer {
        name: input.name().to_owned(),
        text: text.to_owned(),
        raw,
    });
    let level = layers
        .iter()
        .rev()
        .find_map(|layer| layer.raw.level.as_deref().and_then(level_named));
    let mut vocab = Named::default();
    for layer in &layers {
        vocab.add_layer(layer);
    }
    let loaded = load_packages(walk.found, walk.packages, &vocab, &mut faults);
    let known = Known {
        holders: Some(loaded.holders.clone()),
    };
    let mut globs: Vec<Glob> = Vec::new();
    let mut listed: HashMap<String, Vec<String>> = HashMap::new();
    let mut formats = Formats::default();
    for layer in layers {
        if let Err(error) = Input::new(&layer.name, &layer.text).check(&layer.raw, &known) {
            faults.problems.extend(LoadError::from(error).problems);
        }
        let mut at = Faults {
            file: &layer.name,
            problems: Vec::new(),
        };
        for (key, reader) in &layer.raw.formats {
            if let Err(message) = formats.insert(key, reader) {
                at.add(key_line(&layer.text, key, 1), key, message);
            }
        }
        for mut glob in read_globs(layer.raw.files, &layer.text, &loaded, &mut at) {
            add_up(&mut glob, &mut listed);
            if glob.clears_vocab {
                for earlier in globs.iter_mut().filter(|g| g.pattern == glob.pattern) {
                    earlier.vocab.clear();
                }
            }
            globs.push(glob);
        }
        faults.problems.append(&mut at.problems);
    }
    refuse_names(&globs, &loaded, &mut faults);
    if !faults.problems.is_empty() {
        return Err(LoadError {
            problems: faults.problems,
        });
    }
    Ok(Project {
        rules: loaded.rules,
        styles: loaded.styles,
        unrunnable: loaded.unrunnable,
        globs,
        vocabularies: loaded.vocabularies,
        level,
        fail_level,
        gitignore,
        on_unmatched,
        on_unknown,
        formats,
        packages: loaded.packages,
        root,
    })
}

/// Makes the `styles` list of `glob` add to the list an earlier config file
/// gave the same glob text, as Vale merges the sections of one name.
///
/// An empty list stays empty, which drops the earlier styles and the rule
/// settings before it. `listed` holds the list each glob text ended with.
fn add_up(glob: &mut Glob, listed: &mut HashMap<String, Vec<String>>) {
    let Some(own) = glob.styles.as_mut() else {
        return;
    };
    if !own.is_empty()
        && let Some(earlier) = listed.get(&glob.pattern)
    {
        let mut all = earlier.clone();
        for name in own.iter() {
            if !all.contains(name) {
                all.push(name.clone());
            }
        }
        *own = all;
    }
    listed.insert(glob.pattern.clone(), own.clone());
}

/// Loads the style folders of each package in `found`, in order.
fn load_packages(
    found: Vec<Found>,
    packages: Vec<PathBuf>,
    vocab: &Named,
    faults: &mut Faults<'_>,
) -> Loaded {
    let mut loaded = Loaded::default();
    let mut folders: BTreeMap<String, PathBuf> = BTreeMap::new();
    // A rule may extend a rule in any package, so every package is on the
    // search path before the first style is read.
    loaded.packages = packages;
    let search = loaded.packages.clone();
    read_vocabularies(&found, vocab, &mut loaded, faults);
    let top: Vec<&Vocabulary> = vocab
        .top
        .iter()
        .filter_map(|name| loaded.vocabularies.get(name))
        .collect();
    let accepted: Vec<String> = top
        .iter()
        .flat_map(|vocabulary| vocabulary.accepted.iter().cloned())
        .collect();
    let rejected: Vec<String> = top
        .iter()
        .flat_map(|vocabulary| vocabulary.rejected.iter().cloned())
        .collect();
    build_vocabulary_rules(&mut loaded, vocab, &accepted, &rejected, faults);
    for Found {
        origin,
        written,
        folder,
        key,
        line,
    } in found
    {
        let mut at = Faults {
            file: &origin,
            problems: Vec::new(),
        };
        for style in style_folders(&folder, line, &mut at) {
            let name = style
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default();
            let shown = written.join(&name);
            if loaded.styles.contains_key(&name) && !folders.contains_key(&name) {
                faults.add(
                    line,
                    key,
                    format!(
                        "the style `{name}` is built in from a vocabulary, and `{}` holds a style of that name",
                        shown.display()
                    ),
                );
                continue;
            }
            if let Some(first) = folders.get(&name) {
                at.add(
                    line,
                    key,
                    format!(
                        "the style `{name}` is in both `{}` and `{}`",
                        first.display(),
                        shown.display()
                    ),
                );
                continue;
            }
            match read_style(&style, &search, &accepted) {
                Ok(read) => {
                    let first = loaded.rules.len();
                    loaded.rules.extend(read.rules);
                    loaded
                        .styles
                        .insert(name.clone(), (first..loaded.rules.len()).collect());
                    if !read.names.is_empty() {
                        loaded.names.insert(name.clone(), read.names);
                    }
                    loaded
                        .unrunnable
                        .extend(read.unrunnable.into_iter().map(|rule| Held {
                            style: name.clone(),
                            rule,
                        }));
                }
                Err(error) => at.problems.extend(error.problems),
            }
            folders.insert(name, shown);
        }
        faults.problems.append(&mut at.problems);
    }
    loaded
}

/// Reads each glob of `files`, and refuses a style or a rule that names nothing
/// `loaded` holds, and a level that is none.
fn read_globs(
    files: Ordered<Ordered<Setting>>,
    text: &str,
    loaded: &Loaded,
    faults: &mut Faults<'_>,
) -> Vec<Glob> {
    let known: HashSet<&str> = loaded
        .rules
        .iter()
        .map(|rule| rule.name.as_str())
        .chain(loaded.unrunnable.iter().map(|held| held.rule.name.as_str()))
        .collect();
    let mut globs = Vec::new();
    for (pattern, settings) in files.0 {
        let line = key_line(text, &pattern, 1);
        let matcher = match GlobBuilder::new(&pattern).build() {
            Ok(glob) => glob.compile_matcher(),
            Err(error) => {
                faults.add(line, &pattern, format!("is not a glob: {error}"));
                continue;
            }
        };
        let inner = line.map_or(1, |line| line + 1);
        let mut listed = None;
        let mut vocabularies = None;
        let mut block_ignores = Vec::new();
        let mut token_ignores = Vec::new();
        let mut choices = Vec::new();
        for (key, setting) in settings.0 {
            let at = key_line(text, &key, inner);
            match (key.as_str(), setting) {
                ("styles", Setting::List(names)) => {
                    for name in &names {
                        if !loaded.styles.contains_key(name) {
                            faults.add(
                                at,
                                "styles",
                                format!("`{name}` is not a style in any package"),
                            );
                        }
                    }
                    listed = Some(names);
                }
                ("styles", Setting::Word(_)) => {
                    faults.add(at, "styles", "is a list of style names".to_owned());
                }
                ("vocab", Setting::List(list)) => vocabularies = Some(list),
                ("vocab", Setting::Word(_)) => {
                    faults.add(at, "vocab", "is a list of vocabulary names".to_owned());
                }
                ("block_ignores", Setting::List(list)) => {
                    block_ignores = compile_all(&list, at, "block_ignores", faults);
                }
                ("token_ignores", Setting::List(list)) => {
                    token_ignores = compile_all(&list, at, "token_ignores", faults);
                }
                ("block_ignores" | "token_ignores", Setting::Word(_)) => {
                    faults.add(at, &key, "is a list of patterns".to_owned());
                }
                (_, setting) if !known.contains(key.as_str()) => {
                    let message = if matches!(setting, Setting::List(_)) {
                        "is not `styles`, and names no rule in any style the config loads"
                    } else {
                        "names no rule in any style the config loads"
                    };
                    faults.add(at, &key, message.to_owned());
                }
                (_, Setting::List(_)) => {
                    faults.add(at, &key, format!("is set to {LEVELS}, `on` or `off`"));
                }
                (_, Setting::Word(word)) => match (word.as_str(), level_named(&word)) {
                    ("off", _) => choices.push((key.clone(), Choice::Off)),
                    ("on", _) => choices.push((key.clone(), Choice::On)),
                    (_, Some(level)) => choices.push((key.clone(), Choice::Level(level))),
                    (_, None) => faults.add(
                        at,
                        &key,
                        format!("`{word}` is not a level: use {LEVELS}, `on` or `off`"),
                    ),
                },
            }
        }
        globs.push(Glob {
            pattern: pattern.clone(),
            matcher,
            styles: listed,
            clears_vocab: vocabularies.as_ref().is_some_and(Vec::is_empty),
            vocab: vocabularies.unwrap_or_default(),
            block_ignores,
            token_ignores,
            choices,
        });
    }
    globs
}

/// The patterns `list` holds, compiled, and a fault on line `at` of the key
/// `key` for each that does not compile.
fn compile_all(
    list: &[String],
    at: Option<usize>,
    key: &str,
    faults: &mut Faults<'_>,
) -> Vec<Pattern> {
    list.iter()
        .filter_map(|source| match Pattern::new(source) {
            Ok(pattern) => Some(pattern),
            Err(error) => {
                faults.add(at, key, error.to_string());
                None
            }
        })
        .collect()
}

/// Refuses the faults in the rule names of each style a glob lists, as Vale
/// refuses them for a style a config enables.
/// A style no glob lists lints with no error.
fn refuse_names(globs: &[Glob], loaded: &Loaded, faults: &mut Faults<'_>) {
    let mut enabled: BTreeSet<&str> = BTreeSet::new();
    for glob in globs {
        enabled.extend(glob.styles.iter().flatten().map(String::as_str));
    }
    for style in enabled {
        if let Some(problems) = loaded.names.get(style) {
            faults.problems.extend(problems.iter().cloned());
        }
    }
}

/// The vocabularies the config files name: at their top level, and in the
/// globs of `files`.
#[derive(Default)]
struct Named {
    /// Each name once, with the line of the key that first names it and the
    /// file that holds the key.
    all: Vec<(String, Option<usize>, String)>,
    /// The names of the top level, in order: those of each package's file and
    /// then the project's. A written empty list drops the names before it.
    top: Vec<String>,
    /// Every name a top level wrote, cleared or not, which the config check
    /// reports on its own line.
    written: Vec<String>,
    /// The names that a glob lists, each once.
    globs: Vec<String>,
}

impl Named {
    /// Adds the vocabularies `layer` names: its top-level list and the lists
    /// of its globs.
    fn add_layer(&mut self, layer: &Layer) {
        // A list that is written and empty clears the names before it, as
        // Vale's `Vocab = NONE` does; a key that is absent keeps them.
        if let Some(names) = &layer.raw.vocab {
            if names.is_empty() {
                self.top.clear();
            }
            for name in names {
                self.add(name, None, &layer.name);
                if !self.top.contains(name) {
                    self.top.push(name.clone());
                }
                if !self.written.contains(name) {
                    self.written.push(name.clone());
                }
            }
        }
        let text = &layer.text;
        for (pattern, settings) in &layer.raw.files.0 {
            let inner = key_line(text, pattern, 1).map_or(1, |line| line + 1);
            for (key, setting) in &settings.0 {
                if let ("vocab", Setting::List(names)) = (key.as_str(), setting) {
                    let at = key_line(text, "vocab", inner);
                    for name in names {
                        self.add(name, at, &layer.name);
                        if !self.globs.contains(name) {
                            self.globs.push(name.clone());
                        }
                    }
                }
            }
        }
    }

    fn add(&mut self, name: &str, line: Option<usize>, file: &str) {
        if !self.all.iter().any(|(known, _, _)| known == name) {
            self.all.push((name.to_owned(), line, file.to_owned()));
        }
    }
}

/// Reads each vocabulary `vocab` names from the package folders in `found`,
/// and refuses a name no package holds and a name two packages hold.
fn read_vocabularies(found: &[Found], vocab: &Named, loaded: &mut Loaded, faults: &mut Faults<'_>) {
    for (name, line, file) in &vocab.all {
        let mut at = Faults {
            file,
            problems: Vec::new(),
        };
        let holders: Vec<(PathBuf, PathBuf)> = found
            .iter()
            .filter_map(|package| {
                let folder = vocabulary::find(&package.folder, name)?;
                let shown = package
                    .written
                    .join(SETTINGS_FOLDER)
                    .join("vocabularies")
                    .join(name);
                Some((folder, shown))
            })
            .collect();
        loaded.holders.insert(
            name.clone(),
            holders.iter().map(|(_, shown)| shown.clone()).collect(),
        );
        // A name of the top level is refused when the config is checked
        // again, on the line of its own entry.
        let refused = !vocab.written.contains(name);
        match holders.as_slice() {
            [] if refused => at.add(
                *line,
                "vocab",
                format!("`{name}` is not a vocabulary in any package"),
            ),
            [(folder, shown)] => match vocabulary::read(folder, shown) {
                Ok(read) => {
                    loaded.vocabularies.insert(name.clone(), read);
                }
                Err(problems) => at.problems.extend(problems),
            },
            [(_, first), (_, second), ..] if refused => at.add(
                *line,
                "vocab",
                format!(
                    "the vocabulary `{name}` is in both `{}` and `{}`",
                    first.display(),
                    second.display()
                ),
            ),
            _ => {}
        }
        faults.problems.append(&mut at.problems);
    }
}

/// Builds the rules of the vocabularies the config names, as Vale builds them.
/// The lists of the top level give `Vale.Terms` and `Vale.Avoid` in a style
/// `Vale`. Each vocabulary a glob lists gives `Vale.<Name>.Terms` and
/// `Vale.<Name>.Avoid`, which that glob runs on its files.
fn build_vocabulary_rules(
    loaded: &mut Loaded,
    vocab: &Named,
    accepted: &[String],
    rejected: &[String],
    faults: &mut Faults<'_>,
) {
    let mut built = vec![(
        "Vale".to_owned(),
        accepted.to_vec(),
        rejected.to_vec(),
        true,
    )];
    for name in &vocab.globs {
        if let Some(vocabulary) = loaded.vocabularies.get(name) {
            built.push((
                format!("Vale.{name}"),
                vocabulary.accepted.clone(),
                vocabulary.rejected.clone(),
                false,
            ));
        }
    }
    for (prefix, own_accepted, own_rejected, style) in built {
        match vocabulary_rules(&prefix, &own_accepted, &own_rejected, accepted) {
            Ok(rules) if !rules.is_empty() => {
                let first = loaded.rules.len();
                loaded.rules.extend(rules);
                if style {
                    loaded
                        .styles
                        .insert(prefix, (first..loaded.rules.len()).collect());
                }
            }
            Ok(_) => {}
            Err(error) => faults.problems.extend(error.problems),
        }
    }
}

/// The style folders of the package folder `package`, in name order: each
/// folder inside it whose name does not begin with a dot, except
/// [`SETTINGS_FOLDER`], which holds Vale's vocabularies and other settings.
fn style_folders(package: &Path, line: Option<usize>, faults: &mut Faults<'_>) -> Vec<PathBuf> {
    let entries = match fs::read_dir(package) {
        Ok(entries) => entries,
        Err(error) => {
            faults.add(line, "path", format!("`{}`: {error}", package.display()));
            return Vec::new();
        }
    };
    let mut folders: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_dir()
                && path.file_name().is_some_and(|name| {
                    let name = name.to_string_lossy();
                    !name.starts_with('.') && name != SETTINGS_FOLDER
                })
        })
        .collect();
    folders.sort();
    folders
}
