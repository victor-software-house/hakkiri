//! The project config, `.ctl/hakkiri.yaml`: the packages a project loads, the
//! lowest level it reports, and the styles and rule settings of each set of
//! files.
//!
//! [`load`] reads the file once, loads every style folder of every package, and
//! refuses what it cannot run, with the file and line of the fault.
//! [`Project::plan`] then says which rules run on a file.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::{fmt, fs, io};

use ctl_core::input::Input;
use garde::Validate;
use globset::{GlobBuilder, GlobMatcher};
use hakkiri::rule::{Level, LoadError, LoadProblem, Rule, read_style};
use serde::Deserialize;
use serde::de::{Deserializer, MapAccess, Visitor};

use crate::package;

/// Where the config is when no flag or variable names another file.
pub(crate) const DEFAULT: &str = ".ctl/hakkiri.yaml";

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

/// The file as written.
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
struct Raw {
    #[serde(default)]
    #[garde(custom(level_ok))]
    level: Option<String>,
    #[serde(default)]
    #[garde(dive)]
    packages: Vec<Entry>,
    #[serde(default)]
    #[garde(skip)]
    files: Ordered<Ordered<Setting>>,
}

/// One entry of `packages`.
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
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
#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "garde calls a custom rule with a reference to its context."
)]
fn level_ok(value: &Option<String>, _: &()) -> garde::Result {
    match value.as_deref() {
        Some(name) if level_named(name).is_none() => Err(garde::Error::new(format!(
            "`{name}` is not a level: use {LEVELS}"
        ))),
        _ => Ok(()),
    }
}

#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "garde calls a custom rule with a reference to its context."
)]
fn entry_ok(entry: &Entry, _: &()) -> garde::Result {
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
    matcher: GlobMatcher,
    styles: Option<Vec<String>>,
    choices: Vec<(String, Choice)>,
}

/// A config with its packages loaded.
#[derive(Debug)]
pub(crate) struct Project {
    /// Every rule of every style, in package order and then file name order.
    rules: Vec<Rule>,
    /// Where each style's rules are in `rules`.
    styles: BTreeMap<String, Vec<usize>>,
    globs: Vec<Glob>,
    /// The lowest level an alert reports at.
    pub(crate) level: Option<Level>,
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
}

impl Project {
    /// The project root.
    pub(crate) fn root(&self) -> &Path {
        &self.root
    }

    /// Whether the resolved directory `real` is under the project root or
    /// holds it, so a walk of it can reach a file under the root.
    pub(crate) fn leads_to_root(&self, real: &Path) -> bool {
        real.starts_with(&self.root) || self.root.starts_with(real)
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
    pub(crate) fn plan(&self, path: &str) -> Option<Plan<'_>> {
        let mut matched = false;
        let mut styles: &[String] = &[];
        let mut switches: HashMap<&str, bool> = HashMap::new();
        let mut set_levels: HashMap<&str, Level> = HashMap::new();
        for glob in self.globs.iter().filter(|glob| glob.matcher.is_match(path)) {
            matched = true;
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
                        switches.insert(name.as_str(), false);
                    }
                    Choice::On => {
                        switches.insert(name.as_str(), true);
                    }
                    Choice::Level(level) => {
                        switches.insert(name.as_str(), true);
                        set_levels.insert(name.as_str(), *level);
                    }
                }
            }
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
        (!rules.is_empty()).then_some(Plan { rules, levels })
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

/// Reads the config at `path`, and loads the style folders of its packages. A
/// package pinned by `url` is fetched into `cache_dir`, or into the per-user
/// cache folder when there is none, unless that folder already holds it.
///
/// # Errors
///
/// Returns every fault found.
/// A fault is a file that cannot be read, a key the config does not define, a
/// package that cannot be read or that repeats a style, or a rule that does not
/// load. It is also a glob, a style or a rule name that names nothing.
pub(crate) fn load(path: &Path, cache_dir: Option<&Path>) -> Result<Project, LoadError> {
    let input = Input::read(path).map_err(LoadError::from)?;
    let raw: Raw = input.load().map_err(LoadError::from)?;
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
    let loaded = load_packages(&raw.packages, &root, text, cache_dir, &mut faults);
    let globs = read_globs(raw.files, text, &loaded, &mut faults);
    if !faults.problems.is_empty() {
        return Err(LoadError {
            problems: faults.problems,
        });
    }
    Ok(Project {
        rules: loaded.rules,
        styles: loaded.styles,
        globs,
        level: raw.level.as_deref().and_then(level_named),
        packages: loaded.packages,
        root,
    })
}

/// Loads the style folders of each package `entries` names, each `path` read
/// relative to `root` and each `url` fetched into `cache_dir`.
fn load_packages(
    entries: &[Entry],
    root: &Path,
    text: &str,
    cache_dir: Option<&Path>,
    faults: &mut Faults<'_>,
) -> Loaded {
    let mut loaded = Loaded::default();
    let mut folders: BTreeMap<String, PathBuf> = BTreeMap::new();
    let mut from = key_line(text, "packages", 1).unwrap_or(1);
    for entry in entries {
        let key = if entry.url.is_some() { "url" } else { "path" };
        let line = key_line(text, key, from);
        from = line.map_or(from, |line| line + 1);
        let (written, folder) = match (&entry.path, &entry.url, &entry.sha256) {
            (Some(path), _, _) => (PathBuf::from(path), root.join(path)),
            (None, Some(url), Some(sha256)) => match package::fetch(cache_dir, url, sha256) {
                Ok(folder) => (PathBuf::from(url), folder),
                Err(message) => {
                    faults.add(line, key, message);
                    continue;
                }
            },
            (None, _, _) => continue,
        };
        match fs::canonicalize(&folder) {
            Ok(real) => loaded.packages.push(real),
            Err(error) => {
                faults.add(line, key, format!("`{}`: {error}", written.display()));
                continue;
            }
        }
        for style in style_folders(&folder, line, faults) {
            let name = style
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default();
            let shown = written.join(&name);
            if let Some(first) = folders.get(&name) {
                faults.add(
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
            match read_style(&style) {
                Ok(read) => {
                    let first = loaded.rules.len();
                    loaded.rules.extend(read.rules);
                    loaded
                        .styles
                        .insert(name.clone(), (first..loaded.rules.len()).collect());
                    if !read.names.is_empty() {
                        loaded.names.insert(name.clone(), read.names);
                    }
                }
                Err(error) => faults.problems.extend(error.problems),
            }
            folders.insert(name, shown);
        }
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
    let known: HashSet<&str> = loaded.rules.iter().map(|rule| rule.name.as_str()).collect();
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
            matcher,
            styles: listed,
            choices,
        });
    }
    refuse_names(&globs, loaded, faults);
    globs
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

/// The folder of a package's styles that holds Vale's vocabularies and other
/// settings, which is no style.
const SETTINGS_FOLDER: &str = "config";

/// The style folders of the package folder `package`, in name order: each
/// folder inside it whose name does not begin with a dot, except
/// [`SETTINGS_FOLDER`].
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
