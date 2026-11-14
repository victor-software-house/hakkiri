//! The project config, `.ctl/hakkiri.yaml`: the packages a project loads, the
//! lowest level it reports, and the styles and rule settings of each set of
//! files.
//!
//! [`load`] reads the file once, loads every style folder of every package, and
//! refuses what it cannot run, with the file and line of the fault.
//! A package may ship a settings file in the same format, and the settings of
//! each package rank below the project's own at equal specificity.
//! The globs that match a file apply from the least specific to the most, and
//! the values the project tags `!override` apply after all the others.
//! [`Project::plan`] then says which rules run on a file.

use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::{fmt, fs, io, mem, path, slice};

use ctl_core::input::{FieldPath, Input, Tagged};
use garde::Validate;
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use hakkiri::Language;
use hakkiri::engine::DEFAULT_CODE_SPAN_SIZE;
use hakkiri::pattern::Pattern;
use hakkiri::rule::{
    Exceptions, Level, LoadError, LoadProblem, Param, Rule, Unrunnable, read_style,
    vale_repetition, vale_spelling, vocabulary_rules,
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

/// One setting under a glob: the list of `styles`, the word that sets a
/// rule's level, the mapping of `params`, or the number of words of
/// `code_span_size`.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Setting {
    /// A list of style names.
    List(Vec<String>),
    /// A level, `on` or `off`.
    Word(String),
    /// The parameters of some rules, by rule name.
    Map(Ordered<Value>),
    /// A number of words.
    Number(u64),
}

/// A value under `params`, read as written so a fault names its place.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Value {
    Null(()),
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(String),
    #[expect(
        dead_code,
        reason = "A list is read only to be refused, with its place."
    )]
    List(Vec<Value>),
    Map(Ordered<Value>),
}

/// A parameter of a rule as a glob sets it, and where the config writes it.
#[derive(Clone, Debug)]
struct Written {
    param: Param,
    /// The config file that holds it, as a fault names it.
    file: String,
    line: Option<usize>,
}

/// The parameters one glob sets for a rule, by name.
type Params = BTreeMap<String, Written>;

/// What a set of parameters is called when it picks a rule: the same
/// parameters with the same values.
fn variant_key<'a>(params: impl Iterator<Item = (&'a str, &'a Param)>) -> String {
    params
        .map(|(name, param)| format!("{name}={param:?}"))
        .collect::<Vec<_>>()
        .join(";")
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
    files: Ordered<Ordered<Tagged<Setting>>>,
    /// How many words a code span counts as in a word count.
    #[serde(default)]
    #[garde(custom(words_ok))]
    code_span_size: Option<Tagged<u64>>,
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

#[expect(
    clippy::ref_option,
    reason = "garde calls a custom rule with a reference to the field."
)]
fn words_ok(value: &Option<Tagged<u64>>, _: &Known) -> garde::Result {
    match value {
        Some(Tagged(words, _)) if u8::try_from(*words).is_err() => {
            Err(garde::Error::new(too_many_words(*words)))
        }
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

/// The choice a word names: `off`, `on` or a level, `None` for any other word.
fn choice_named(word: &str) -> Option<Choice> {
    match word {
        "off" => Some(Choice::Off),
        "on" => Some(Choice::On),
        _ => level_named(word).map(Choice::Level),
    }
}

/// The tag that forces a value, as the YAML text of a config writes it.
const OVERRIDE: &str = "!override";

/// How specific a glob is: four counts, compared in order, a larger count more
/// specific.
/// They are the parts between slashes made of literal characters only, the
/// parts that hold a literal character and something else, the parts that are
/// `**` (fewer is more specific, so [`Reverse`]), and the literal characters of
/// the whole glob.
type Specificity = (usize, usize, Reverse<usize>, usize);

/// The specificity of the glob `pattern`, one of the [`alternatives`] of a
/// glob, so it holds no `{…}` group.
///
/// A literal character is one outside a `[…]` class that is not `*` or `?`,
/// and a backslash with the character after it is one literal character.
fn specificity(pattern: &str) -> Specificity {
    let (mut only, mut mixed, mut recursive, mut literals) = (0, 0, 0, 0);
    let mut close = |raw: &str, held: usize, other: bool| {
        literals += held;
        if raw == "**" {
            recursive += 1;
        } else if held > 0 {
            if other {
                mixed += 1;
            } else {
                only += 1;
            }
        }
    };
    // The members seen in a class so far, and whether it is negated.
    let mut class: Option<(usize, bool)> = None;
    let (mut start, mut held, mut other) = (0, 0, false);
    let mut chars = pattern.char_indices();
    while let Some((at, c)) = chars.next() {
        if let Some((members, negated)) = class.as_mut() {
            other = true;
            match c {
                '!' | '^' if *members == 0 && !*negated => *negated = true,
                ']' if *members > 0 => class = None,
                _ => *members += 1,
            }
            continue;
        }
        match c {
            '/' => {
                close(&pattern[start..at], held, other);
                (start, held, other) = (at + 1, 0, false);
            }
            '\\' => {
                chars.next();
                held += 1;
            }
            '[' => {
                class = Some((0, false));
                other = true;
            }
            '*' | '?' => other = true,
            _ => held += 1,
        }
    }
    close(&pattern[start..], held, other);
    (only, mixed, Reverse(recursive), literals)
}

/// The most alternatives one glob may expand to.
const MAX_ALTERNATIVES: usize = 256;

/// The alternatives of the glob `pattern`: one glob for each choice of every
/// `{…}` group, which together match what `pattern` matches.
///
/// A brace in a `[…]` class or after a backslash is no group. More than
/// [`MAX_ALTERNATIVES`] is `None`.
fn alternatives(pattern: &str) -> Option<Vec<String>> {
    let mut done = Vec::new();
    let mut pending = vec![pattern.to_owned()];
    while let Some(glob) = pending.pop() {
        match first_group(&glob) {
            None => done.push(glob),
            Some((open, close, commas)) => {
                let mut from = open + 1;
                for until in commas.into_iter().chain([close]) {
                    pending.push(format!(
                        "{}{}{}",
                        &glob[..open],
                        &glob[from..until],
                        &glob[close + 1..]
                    ));
                    from = until + 1;
                }
            }
        }
        if done.len() + pending.len() > MAX_ALTERNATIVES {
            return None;
        }
    }
    done.reverse();
    Some(done)
}

/// The byte offsets of the first `{…}` group of `glob`: its opening brace, its
/// closing brace, and the commas between its choices, those of a group inside
/// it left out.
fn first_group(glob: &str) -> Option<(usize, usize, Vec<usize>)> {
    let (mut open, mut depth, mut commas) = (None, 0_usize, Vec::new());
    // The members seen in a class so far, and whether it is negated, as
    // `specificity` reads a class.
    let mut class: Option<(usize, bool)> = None;
    let mut chars = glob.char_indices();
    while let Some((at, c)) = chars.next() {
        if let Some((members, negated)) = class.as_mut() {
            match c {
                '!' | '^' if *members == 0 && !*negated => *negated = true,
                ']' if *members > 0 => class = None,
                _ => *members += 1,
            }
            continue;
        }
        match c {
            '\\' => {
                chars.next();
            }
            '[' => class = Some((0, false)),
            '{' => {
                open.get_or_insert(at);
                depth += 1;
            }
            '}' if depth > 0 => {
                depth -= 1;
                if depth == 0 {
                    return open.map(|open| (open, at, commas));
                }
            }
            ',' if depth == 1 => commas.push(at),
            _ => {}
        }
    }
    None
}

/// The glob `pattern`, read by the rules of gitignore and `EditorConfig`.
///
/// `*`, `?` and a class stay inside one folder, and `**` spans folders.
/// A pattern with no `/` matches the file name in any folder, and a leading `/`
/// ties a pattern to the project root. A trailing `/` matches every file the
/// folder holds, and a trailing `/**` every file below the folder, never the
/// folder's own name.
pub(crate) fn standard_glob(pattern: &str) -> Result<globset::Glob, globset::Error> {
    let rooted = pattern.strip_prefix('/');
    let body = rooted.unwrap_or(pattern);
    let mut text = if rooted.is_none() && !body.trim_end_matches('/').contains('/') {
        format!("**/{body}")
    } else {
        body.to_owned()
    };
    if text.ends_with('/') {
        text.push_str("**");
    }
    GlobBuilder::new(&text).literal_separator(true).build()
}

/// The [`alternatives`] of the glob `pattern`, each compiled with its
/// specificity, or the message of the fault that refuses `pattern`.
fn compile(pattern: &str) -> Result<Vec<(globset::Glob, Specificity)>, String> {
    let not_a_glob = |error: globset::Error| format!("is not a glob: {error}");
    standard_glob(pattern).map_err(not_a_glob)?;
    let all = alternatives(pattern).ok_or_else(|| {
        format!("has more than {MAX_ALTERNATIVES} alternatives: split it into several globs")
    })?;
    all.iter()
        .map(|alternative| {
            standard_glob(alternative)
                .map(|compiled| (compiled, specificity(alternative)))
                .map_err(not_a_glob)
        })
        .collect()
}

/// What one glob sets for a file that matches it: the settings that carry no
/// tag, or the ones the project tags `!override`.
#[derive(Debug, Default)]
struct Sets {
    styles: Option<Vec<String>>,
    /// The vocabularies the glob names, for the files it matches. A list
    /// written empty drops the lists of the earlier globs of the same text.
    vocab: Option<Vec<String>>,
    /// The patterns whose matches are hidden as blocks from the files the glob
    /// matches.
    block_ignores: Option<Vec<Pattern>>,
    /// The patterns whose matches are hidden as inline text.
    token_ignores: Option<Vec<Pattern>>,
    /// How many words a code span counts as in the files the glob matches.
    code_span_size: Option<u8>,
    choices: Vec<(String, Choice)>,
    /// The parameters the glob sets, by rule name, in the order written.
    params: Vec<(String, Params)>,
}

impl Sets {
    /// Whether the glob wrote an empty `vocab` list.
    fn clears_vocab(&self) -> bool {
        self.vocab.as_ref().is_some_and(Vec::is_empty)
    }
}

/// One glob of `files` and what it sets.
#[derive(Debug)]
struct Glob {
    /// The glob as written, which two config files share when they write it
    /// alike.
    pattern: String,
    /// The [`alternatives`] of the glob, each compiled, with its specificity.
    /// The most specific one that matches a file ranks the glob for that file.
    branches: Vec<(globset::Glob, Specificity)>,
    /// The place of its config file among the layers, a package in load order
    /// and the project last, which ranks it second.
    origin: usize,
    /// What it sets without a tag.
    normal: Sets,
    /// What it sets with `!override`, which only the project's config holds.
    forced: Sets,
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
    /// Every glob of the packages and the project, and where each of their
    /// alternatives ranks.
    ranking: Ranking,
    /// The rule each set of parameters gives, by the rule's place in `rules`
    /// and the parameters' [`variant_key`]: one for every set that the globs
    /// matching some file can add up to.
    variants: HashMap<(usize, String), Rule>,
    /// The vocabularies the config names, by name.
    vocabularies: BTreeMap<String, Vocabulary>,
    /// The lowest level an alert reports at.
    pub(crate) level: Option<Level>,
    /// The top-level `code_span_size` without a tag of the last config file
    /// that sets it, the project's after its packages'. It ranks below every
    /// glob.
    code_span_size: Option<u8>,
    /// The top-level `code_span_size` the project tags `!override`, which
    /// applies after every value without the tag.
    forced_code_span_size: Option<u8>,
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
    /// What the load skipped, such as a package style folder named for a
    /// built-in style.
    pub(crate) warnings: Vec<String>,
    /// The project root: the folder that holds the `.ctl` folder of the
    /// config, with symlinks resolved.
    root: PathBuf,
}

/// Where the alternatives of every glob rank: the least specific first, then by
/// origin, then in load order.
#[derive(Debug)]
struct Ranking {
    /// Every glob of the packages and the project, in load order: each
    /// package's, then the project's, each as written.
    globs: Vec<Glob>,
    /// Every alternative of every glob, so one pass over a path finds all
    /// those that match it.
    set: GlobSet,
    /// For each alternative of `set`, its glob in load order and its rank.
    branches: Vec<(usize, usize)>,
    /// The glob of each rank, from the lowest, a glob once for each of its
    /// alternatives.
    order: Vec<usize>,
}

impl Ranking {
    /// Ranks the alternatives of `globs`, which [`load`] holds in load order.
    /// Alternatives that do not compile into one set are a fault, and then no
    /// glob matches.
    fn of(mut globs: Vec<Glob>, faults: &mut Faults<'_>) -> Self {
        let mut all = Vec::new();
        for (index, glob) in globs.iter_mut().enumerate() {
            for (branch, specificity) in mem::take(&mut glob.branches) {
                all.push((specificity, glob.origin, index, branch));
            }
        }
        let mut order: Vec<usize> = (0..all.len()).collect();
        order.sort_by_key(|&branch| {
            let (specificity, origin, index, _) = &all[branch];
            (*specificity, *origin, *index)
        });
        let mut rank = vec![0; all.len()];
        for (place, &branch) in order.iter().enumerate() {
            rank[branch] = place;
        }
        let mut set = GlobSetBuilder::new();
        for (_, _, _, branch) in &all {
            set.add(branch.clone());
        }
        match set.build() {
            Ok(set) => Self {
                globs,
                set,
                branches: all
                    .iter()
                    .zip(rank)
                    .map(|((_, _, index, _), rank)| (*index, rank))
                    .collect(),
                order: order.into_iter().map(|branch| all[branch].2).collect(),
            },
            Err(error) => {
                let message = format!("the globs do not compile together: {error}");
                faults.add(None, "files", message);
                Self {
                    globs,
                    set: GlobSet::empty(),
                    branches: Vec::new(),
                    order: Vec::new(),
                }
            }
        }
    }

    /// Every glob in the rank of its alternatives, from the lowest, a glob once
    /// for each of its alternatives.
    fn ranked(&self) -> Vec<&Glob> {
        self.order.iter().map(|&index| &self.globs[index]).collect()
    }

    /// The globs that match `path`, from the lowest rank to the highest, each
    /// once at the rank of its most specific alternative that matches.
    fn matching(&self, path: &str) -> Vec<&Glob> {
        let mut best: HashMap<usize, usize> = HashMap::new();
        for branch in self.set.matches(path) {
            let (index, rank) = self.branches[branch];
            let top = best.entry(index).or_insert(rank);
            *top = (*top).max(rank);
        }
        let mut ranked: Vec<(usize, usize)> = best
            .into_iter()
            .map(|(index, rank)| (rank, index))
            .collect();
        ranked.sort_unstable();
        ranked
            .into_iter()
            .map(|(_, index)| &self.globs[index])
            .collect()
    }
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
    /// name, one for each, in rank order.
    pub(crate) accepted: Vec<&'a Exceptions>,
    /// The block patterns of the globs matching the file, in rank order.
    pub(crate) block_ignores: Vec<&'a Pattern>,
    /// The token patterns of the globs matching the file, in the same order.
    pub(crate) token_ignores: Vec<&'a Pattern>,
    /// How many words a code span counts as in a word count of the file.
    /// The matching glob of the highest rank gives it, else the config's top
    /// level, else one, and a forced value beats them all.
    pub(crate) code_span_size: u8,
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
        self.ranking.set.is_match(path)
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
        let absolute = path::absolute(file).ok()?;
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
    /// The globs that match apply in rank order, a higher rank winning for the
    /// same key, then the values the project tags `!override` apply in the same
    /// order.
    /// A `styles` list replaces the one of a lower rank, and an empty list also
    /// drops every rule setting of a lower rank, as Vale's empty
    /// `BasedOnStyles` does.
    /// A rule runs when a listed style holds it and the config does not turn it
    /// off, or when the config turns it on or sets its level, whatever style
    /// holds it. A level once set stays when a glob of higher rank turns the
    /// rule off or on, as in Vale.
    /// A parameter a glob sets reaches the files that glob matches, a higher
    /// rank winning for the same parameter, and an empty `styles` list drops
    /// the parameters of a lower rank as it drops the levels.
    /// The lists of `vocab`, `block_ignores` and `token_ignores` add up over
    /// every matching glob; when one forces a list for a key, the forced lists
    /// of that key replace the others.
    /// A rule hakkiri cannot run is held to the same test, and each one that
    /// passes it is in [`Plan::unrunnable`].
    pub(crate) fn plan(&self, path: &str) -> Option<Plan<'_>> {
        let matching = self.ranking.matching(path);
        if matching.is_empty() {
            return None;
        }
        let fold = Fold::of(
            &matching,
            self.code_span_size.unwrap_or(DEFAULT_CODE_SPAN_SIZE),
            self.forced_code_span_size,
        );
        let Fold {
            styles,
            switches,
            set_levels,
            set_params,
            named,
            block_ignores,
            token_ignores,
            code_span_size,
        } = fold;
        let mut rules = Vec::new();
        let mut levels = HashMap::new();
        let mut listed = HashSet::new();
        for index in styles
            .iter()
            .filter_map(|style| self.styles.get(style))
            .flatten()
        {
            listed.insert(*index);
            let rule = self.rule_for(*index, &set_params)?;
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
                rules.push(self.rule_for(index, &set_params)?);
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
            code_span_size,
        })
    }
}

/// Applies the rule keys of one glob: `off` and `on` switch a rule, and a level
/// switches it on and sets the level of its alerts.
fn choose<'a>(
    choices: &'a [(String, Choice)],
    switches: &mut HashMap<String, bool>,
    set_levels: &mut HashMap<&'a str, Level>,
) {
    for (name, choice) in choices {
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
}

/// Which lists of a set a pass over the globs reads.
#[derive(Clone, Copy)]
struct Lists {
    vocab: bool,
    block_ignores: bool,
    token_ignores: bool,
}

/// What the globs that match a file have set so far.
struct Fold<'a> {
    styles: &'a [String],
    switches: HashMap<String, bool>,
    set_levels: HashMap<&'a str, Level>,
    set_params: HashMap<&'a str, BTreeMap<&'a str, &'a Written>>,
    named: Vec<&'a str>,
    block_ignores: Vec<&'a Pattern>,
    token_ignores: Vec<&'a Pattern>,
    code_span_size: u8,
}

impl<'a> Fold<'a> {
    /// What the `globs` that match a file set, in rank order, starting from
    /// `code_span_size`: first the settings without the tag, then the forced
    /// ones, with the project's forced `code_span_size` ahead of them.
    fn of(globs: &[&'a Glob], code_span_size: u8, forced_code_span_size: Option<u8>) -> Self {
        let mut fold = Self {
            styles: &[],
            switches: HashMap::new(),
            set_levels: HashMap::new(),
            set_params: HashMap::new(),
            named: Vec::new(),
            block_ignores: Vec::new(),
            token_ignores: Vec::new(),
            code_span_size,
        };
        // A forced list replaces the lists without the tag for its key.
        let unforced = Lists {
            vocab: globs.iter().all(|glob| glob.forced.vocab.is_none()),
            block_ignores: globs.iter().all(|glob| glob.forced.block_ignores.is_none()),
            token_ignores: globs.iter().all(|glob| glob.forced.token_ignores.is_none()),
        };
        for glob in globs {
            fold.apply(&glob.normal, unforced);
        }
        fold.code_span_size = forced_code_span_size.unwrap_or(fold.code_span_size);
        for glob in globs {
            fold.apply(
                &glob.forced,
                Lists {
                    vocab: true,
                    block_ignores: true,
                    token_ignores: true,
                },
            );
        }
        fold
    }

    /// Applies what one glob sets, a later call winning for the same key.
    /// A list that `lists` leaves out is not read.
    fn apply(&mut self, sets: &'a Sets, lists: Lists) {
        self.code_span_size = sets.code_span_size.unwrap_or(self.code_span_size);
        if lists.block_ignores {
            self.block_ignores
                .extend(sets.block_ignores.iter().flatten());
        }
        if lists.token_ignores {
            self.token_ignores
                .extend(sets.token_ignores.iter().flatten());
        }
        let vocab: &[String] = sets
            .vocab
            .as_deref()
            .filter(|_| lists.vocab)
            .unwrap_or_default();
        for name in vocab {
            if !self.named.contains(&name.as_str()) {
                self.named.push(name);
            }
        }
        if let Some(listed) = &sets.styles {
            self.styles = listed;
            if listed.is_empty() {
                self.switches.clear();
                self.set_levels.clear();
                self.set_params.clear();
            }
        }
        for (rule, params) in &sets.params {
            self.set_params.entry(rule.as_str()).or_default().extend(
                params
                    .iter()
                    .map(|(name, written)| (name.as_str(), written)),
            );
        }
        choose(&sets.choices, &mut self.switches, &mut self.set_levels);
        // A glob that names a vocabulary runs the rules of that vocabulary on
        // its files, after the glob's own settings, as in Vale.
        set_vocabulary_rules(&mut self.switches, vocab);
    }
}

impl Project {
    /// The rule at `index`, read with the parameters `set` holds for its name
    /// when it holds any.
    fn rule_for(
        &self,
        index: usize,
        set: &HashMap<&str, BTreeMap<&str, &Written>>,
    ) -> Option<&Rule> {
        let rule = self.rules.get(index)?;
        match set
            .get(rule.name.as_str())
            .filter(|params| !params.is_empty())
        {
            Some(params) => {
                let key = variant_key(params.iter().map(|(name, w)| (*name, &w.param)));
                self.variants.get(&(index, key))
            }
            None => Some(rule),
        }
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

/// The 1-based line where `input` writes the key at `path`, or where its
/// nearest written ancestor's key is, and `None` when the text does not parse.
///
/// A fault about a key itself, such as a glob that is no glob, belongs here,
/// since the value of a key that holds a block collection starts on the line
/// below it.
fn key_line_of(input: &Input, path: &FieldPath) -> Option<usize> {
    input.place_key(path).map(|at| at.line)
}

/// The 1-based line where `input` writes the value at `path`, or where its
/// nearest written ancestor is, and `None` when the text does not parse.
///
/// The path names each key as one step, so a glob such as `docs/**/*.md` or
/// `*.{md,rs}` is one key and not a run of steps.
fn line_of(input: &Input, path: &FieldPath) -> Option<usize> {
    input.place(path).map(|at| at.line)
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
    /// The accepted terms of the project's vocabularies, which the rules read
    /// as exceptions.
    accepted: Vec<String>,
    /// What the load skipped, each with the config file and line that led to
    /// it.
    warnings: Vec<String>,
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
    /// The text, read once, so each fault finds its line through the same
    /// document.
    input: Input,
    raw: Raw,
}

/// The packages of a config, and the packages they list, with the settings
/// files they ship.
///
/// The packages a package lists come before it, in the order written, so its
/// own settings rank above theirs at equal specificity.
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
    fn entries(&mut self, entries: &[Entry], input: &Input, file: &str, depth: usize) {
        let mut faults = Faults {
            file,
            problems: Vec::new(),
        };
        for (index, entry) in entries.iter().enumerate() {
            let key = if entry.url.is_some() { "url" } else { "path" };
            let entry_path = FieldPath::root().key("packages").index(index).key(key);
            let line = line_of(input, &entry_path);
            if depth > 0 && entry.path.is_some() {
                faults.add(
                    key_line_of(input, &entry_path),
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
                written.parent().map_or_default(Path::to_path_buf)
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
        let input = Input::new(&name, &text);
        let raw: Raw = match input.load() {
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
                    key_line_of(&input, &FieldPath::root().key(key)),
                    key,
                    "belongs to the project's config, not a package's settings".to_owned(),
                );
            }
        }
        self.problems.append(&mut faults.problems);
        self.entries(&raw.packages, &input, &name, depth + 1);
        self.layers.push(Layer { name, input, raw });
    }
}

/// Reads the config at `path`, and loads the style folders of its packages. A
/// package pinned by `url` is fetched into `cache_dir`, or into the per-user
/// cache folder when there is none, unless that folder already holds it.
///
/// A package whose top folder holds `.ctl/hakkiri.yaml` ships settings in the
/// format of the config: they rank below the project's at equal specificity,
/// and the packages that file lists are fetched as the project's are.
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
    let name = input.name().to_owned();
    let mut faults = Faults {
        file: &name,
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
    walk.entries(&raw.packages, &input, input.name(), 0);
    faults.problems.append(&mut walk.problems);
    let mut layers = walk.layers;
    layers.push(Layer {
        name: name.clone(),
        input,
        raw,
    });
    let (level, code_span_size, forced_code_span_size) = last_settings(&layers);
    let project = layers.len() - 1;
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
    for (origin, layer) in layers.into_iter().enumerate() {
        if let Err(error) = layer.input.check(&layer.raw, &known) {
            faults.problems.extend(LoadError::from(error).problems);
        }
        let mut at = Faults {
            file: &layer.name,
            problems: Vec::new(),
        };
        refuse_top_level_tag(&layer, origin == project, &mut at);
        for (key, reader) in &layer.raw.formats {
            if let Err(message) = formats.insert(key, reader) {
                let path = FieldPath::root().key("formats").key(key);
                at.add(line_of(&layer.input, &path), key, message);
            }
        }
        let read = read_globs(
            layer.raw.files,
            &layer.input,
            &loaded,
            &mut at,
            origin,
            origin == project,
        );
        push_globs(&mut globs, read, &mut listed);
        faults.problems.append(&mut at.problems);
    }
    let ranking = Ranking::of(globs, &mut faults);
    refuse_names(&ranking.globs, &loaded, &mut faults);
    let variants = build_variants(&ranking.ranked(), &loaded, &mut faults);
    if !faults.problems.is_empty() {
        return Err(LoadError {
            problems: faults.problems,
        });
    }
    Ok(Project {
        rules: loaded.rules,
        styles: loaded.styles,
        unrunnable: loaded.unrunnable,
        ranking,
        variants,
        vocabularies: loaded.vocabularies,
        level,
        code_span_size,
        forced_code_span_size,
        fail_level,
        gitignore,
        on_unmatched,
        on_unknown,
        formats,
        packages: loaded.packages,
        warnings: loaded.warnings,
        root,
    })
}

/// Refuses the tag of the top-level `code_span_size` of `layer` when it is no
/// `!override` of the project's config, on the line of the key.
fn refuse_top_level_tag(layer: &Layer, project: bool, faults: &mut Faults<'_>) {
    if let Some(Tagged(_, Some(tag))) = &layer.raw.code_span_size
        && let Err(message) = forces(tag, project)
    {
        let key = FieldPath::root().key("code_span_size");
        faults.add(key_line_of(&layer.input, &key), "code_span_size", message);
    }
}

/// Adds the globs `read` to `globs`, in the order written, as one config file
/// gave them.
///
/// A `styles` list adds to the one an earlier file gave the same glob text, and
/// an empty `vocab` list drops the lists of the earlier globs of that text.
fn push_globs(globs: &mut Vec<Glob>, read: Vec<Glob>, listed: &mut HashMap<String, Vec<String>>) {
    for mut glob in read {
        add_up(&mut glob, listed);
        if glob.normal.clears_vocab() {
            for earlier in globs.iter_mut().filter(|g| g.pattern == glob.pattern) {
                if let Some(list) = earlier.normal.vocab.as_mut() {
                    list.clear();
                }
            }
        }
        globs.push(glob);
    }
}

/// The `level` and the top-level `code_span_size` of the last config file that
/// sets each: the project's after its packages'.
/// The third value is the `code_span_size` the project tags `!override`.
fn last_settings(layers: &[Layer]) -> (Option<Level>, Option<u8>, Option<u8>) {
    let level = layers
        .iter()
        .rev()
        .find_map(|layer| layer.raw.level.as_deref().and_then(level_named));
    let code_span_size = layers
        .iter()
        .rev()
        .find_map(|layer| match &layer.raw.code_span_size {
            Some(Tagged(words, None)) => Some(*words),
            _ => None,
        })
        .and_then(|words| u8::try_from(words).ok());
    let forced = layers
        .last()
        .and_then(|layer| match &layer.raw.code_span_size {
            Some(Tagged(words, Some(tag))) if tag == OVERRIDE => Some(*words),
            _ => None,
        })
        .and_then(|words| u8::try_from(words).ok());
    (level, code_span_size, forced)
}

/// Makes the `styles` list of `glob` add to the list an earlier config file
/// gave the same glob text, as Vale merges the sections of one name.
///
/// An empty list stays empty, which drops the earlier styles and the rule
/// settings of a lower rank. `listed` holds the list each glob text ended with.
/// A list the project forces is taken as written, so it is not added up.
fn add_up(glob: &mut Glob, listed: &mut HashMap<String, Vec<String>>) {
    let Some(own) = glob.normal.styles.as_mut() else {
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
    loaded.accepted.clone_from(&accepted);
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
                .map_or_default(|name| name.to_string_lossy().into_owned());
            let shown = written.join(&name);
            // Vale reads its own style first and passes over a folder of the
            // same name, so the folder's rules never run.
            if loaded.styles.contains_key(&name) && !folders.contains_key(&name) {
                let mut skipped = Faults {
                    file: &origin,
                    problems: Vec::new(),
                };
                skipped.add(
                    line,
                    key,
                    format!(
                        "`{}` is skipped: the built-in style `{name}` wins",
                        shown.display()
                    ),
                );
                loaded.warnings.push(
                    LoadError {
                        problems: skipped.problems,
                    }
                    .to_string(),
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

/// Whether a value tagged `tag` is forced, or the fault that its tag is.
///
/// Only the project's config forces a value, with `!override`, and no other
/// tag is read.
fn forces(tag: &str, project: bool) -> Result<bool, String> {
    match tag {
        OVERRIDE if project => Ok(true),
        OVERRIDE => Err(format!(
            "`{OVERRIDE}` is not allowed in a package's settings: only the project's config forces a value"
        )),
        other => Err(format!(
            "`{other}` is not a tag hakkiri reads: use `{OVERRIDE}` or none"
        )),
    }
}

/// Reads each glob of `files`, and refuses a style or a rule that names nothing
/// `loaded` holds, a level that is none, and a tag that is not `!override` or
/// that a package's settings file holds.
///
/// `origin` is the place of the config file among the layers, and `project`
/// whether it is the project's.
fn read_globs(
    files: Ordered<Ordered<Tagged<Setting>>>,
    input: &Input,
    loaded: &Loaded,
    faults: &mut Faults<'_>,
    origin: usize,
    project: bool,
) -> Vec<Glob> {
    let known: HashSet<&str> = loaded
        .rules
        .iter()
        .map(|rule| rule.name.as_str())
        .chain(loaded.unrunnable.iter().map(|held| held.rule.name.as_str()))
        .collect();
    let mut globs = Vec::new();
    for (pattern, settings) in files.0 {
        // One key of the path, whatever dots, stars and slashes the glob holds.
        let glob = FieldPath::root().key("files").key(&pattern);
        let line = key_line_of(input, &glob);
        let branches = match compile(&pattern) {
            Ok(branches) => branches,
            Err(message) => {
                faults.add(line, &pattern, message);
                continue;
            }
        };
        let (mut normal, mut forced) = (Sets::default(), Sets::default());
        for (key, Tagged(setting, tag)) in settings.0 {
            let at = line_of(input, &glob.clone().key(&key));
            let force = match tag.as_deref().map(|tag| forces(tag, project)) {
                None => false,
                Some(Ok(force)) => force,
                Some(Err(message)) => {
                    faults.add(key_line_of(input, &glob.clone().key(&key)), &key, message);
                    continue;
                }
            };
            let sets = if force { &mut forced } else { &mut normal };
            match (key.as_str(), setting) {
                ("styles", Setting::List(names)) => {
                    refuse_unknown_styles(&names, input, &glob, loaded, faults);
                    sets.styles = Some(names);
                }
                ("vocab", Setting::List(list)) => sets.vocab = Some(list),
                ("block_ignores", Setting::List(list)) => {
                    sets.block_ignores =
                        Some(compile_all(&list, input, &glob, "block_ignores", faults));
                }
                ("token_ignores", Setting::List(list)) => {
                    sets.token_ignores =
                        Some(compile_all(&list, input, &glob, "token_ignores", faults));
                }
                (list @ ("styles" | "vocab" | "block_ignores" | "token_ignores"), _) => {
                    faults.add(at, list, expects_a_list(list).to_owned());
                }
                ("params", Setting::Map(rules)) => {
                    sets.params = read_params(rules, input, &glob, &known, faults);
                }
                ("params", Setting::List(_) | Setting::Word(_) | Setting::Number(_)) => {
                    faults.add(
                        at,
                        "params",
                        "is a mapping of rule names to their parameters".to_owned(),
                    );
                }
                ("code_span_size", setting) => {
                    sets.code_span_size = glob_words(&setting, at, faults);
                }
                (_, setting) if !known.contains(key.as_str()) => {
                    let message = if matches!(setting, Setting::List(_)) {
                        "is not `styles`, and names no rule in any style the config loads"
                    } else {
                        "names no rule in any style the config loads"
                    };
                    faults.add(
                        key_line_of(input, &glob.clone().key(&key)),
                        &key,
                        message.to_owned(),
                    );
                }
                (_, Setting::List(_) | Setting::Map(_) | Setting::Number(_)) => {
                    faults.add(at, &key, format!("is set to {LEVELS}, `on` or `off`"));
                }
                (_, Setting::Word(word)) => match choice_named(&word) {
                    Some(choice) => sets.choices.push((key.clone(), choice)),
                    None => faults.add(
                        at,
                        &key,
                        format!("`{word}` is not a level: use {LEVELS}, `on` or `off`"),
                    ),
                },
            }
        }
        globs.push(Glob {
            pattern: pattern.clone(),
            branches,
            origin,
            normal,
            forced,
        });
    }
    globs
}

/// Refuses each name of the `styles` list under the glob at `glob` that no
/// package holds, on the line of that name.
fn refuse_unknown_styles(
    names: &[String],
    input: &Input,
    glob: &FieldPath,
    loaded: &Loaded,
    faults: &mut Faults<'_>,
) {
    let list = glob.clone().key("styles");
    for (index, name) in names.iter().enumerate() {
        if !loaded.styles.contains_key(name) {
            faults.add(
                line_of(input, &list.clone().index(index)),
                "styles",
                format!("`{name}` is not a style in any package"),
            );
        }
    }
}

/// The parameters `rules` holds, the value of `params` under the glob at
/// `glob` in `input`, and a fault on the line of each name or value that is
/// not one.
///
/// A rule must be one the config loads and its parameters a mapping of one
/// value each: a number, `true` or `false`, or text. What a rule's check does
/// with a parameter is refused when its rules are read again.
fn read_params(
    rules: Ordered<Value>,
    input: &Input,
    glob: &FieldPath,
    known: &HashSet<&str>,
    faults: &mut Faults<'_>,
) -> Vec<(String, Params)> {
    let mut read = Vec::new();
    for (rule, value) in rules.0 {
        let at = glob.clone().key("params").key(&rule);
        if !known.contains(rule.as_str()) {
            faults.add(
                key_line_of(input, &at),
                &rule,
                "names no rule in any style the config loads".to_owned(),
            );
            continue;
        }
        let Value::Map(values) = value else {
            faults.add(
                line_of(input, &at),
                &rule,
                "is a mapping of parameter names to values".to_owned(),
            );
            continue;
        };
        let mut params = Params::new();
        for (name, value) in values.0 {
            let place = line_of(input, &at.clone().key(&name));
            let param = match value {
                Value::Bool(value) => Param::Bool(value),
                Value::Int(value) => Param::Int(value),
                Value::Float(value) => Param::Float(value),
                Value::Text(value) => Param::Text(value),
                Value::Null(()) => {
                    faults.add(place, &name, "needs a value".to_owned());
                    continue;
                }
                Value::List(_) | Value::Map(_) => {
                    faults.add(
                        place,
                        &name,
                        "takes one value: a number, `true` or `false`, or text".to_owned(),
                    );
                    continue;
                }
            };
            params.insert(
                name,
                Written {
                    param,
                    file: faults.file.to_owned(),
                    line: place,
                },
            );
        }
        read.push((rule, params));
    }
    read
}

/// What a glob's `key` holds, when it is one of the keys that hold a list.
fn expects_a_list(key: &str) -> &'static str {
    match key {
        "styles" => "is a list of style names",
        "vocab" => "is a list of vocabulary names",
        _ => "is a list of patterns",
    }
}

/// The words a glob's `code_span_size` sets, or a fault on the line `at`.
fn glob_words(setting: &Setting, at: Option<usize>, faults: &mut Faults<'_>) -> Option<u8> {
    let Setting::Number(words) = setting else {
        faults.add(
            at,
            "code_span_size",
            "is the number of words a code span counts as, such as 0 or 1".to_owned(),
        );
        return None;
    };
    let size = u8::try_from(*words).ok();
    if size.is_none() {
        faults.add(at, "code_span_size", too_many_words(*words));
    }
    size
}

/// The fault of a `code_span_size` over the most a count holds.
fn too_many_words(words: u64) -> String {
    format!("`{words}` is too many words: use 0 to {}", u8::MAX)
}

/// The patterns `list` holds, the value of `key` under the glob at `glob` in
/// `input`, compiled, and a fault on the line of each that does not compile.
fn compile_all(
    list: &[String],
    input: &Input,
    glob: &FieldPath,
    key: &str,
    faults: &mut Faults<'_>,
) -> Vec<Pattern> {
    let items = glob.clone().key(key);
    list.iter()
        .enumerate()
        .filter_map(|(index, source)| match Pattern::new(source) {
            Ok(pattern) => Some(pattern),
            Err(error) => {
                let at = line_of(input, &items.clone().index(index));
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
        for sets in [&glob.normal, &glob.forced] {
            enabled.extend(sets.styles.iter().flatten().map(String::as_str));
        }
    }
    for style in enabled {
        if let Some(problems) = loaded.names.get(style) {
            faults.problems.extend(problems.iter().cloned());
        }
    }
}

/// Reads each rule again for every set of parameters that the globs matching
/// some file can give it, and refuses a parameter its rule cannot take.
///
/// A file gets the parameters of the globs that match it, in rank order, and an
/// empty `styles` list drops those of a lower rank. Then it gets the forced
/// parameters of the same globs, in the same order, and an empty forced list
/// drops every parameter before it.
/// Each glob either matches a file or does not, so the sets a rule can get are
/// those that each choice leaves, first over the parameters without the tag and
/// then over the forced ones. Sets that come out alike count once.
/// `globs` holds a glob once at the rank of each of its alternatives, so the
/// place it takes for any file is among the choices.
/// A set no file gives may be read too, and its faults are those of its
/// parameters.
/// A fault is placed on the line of the parameter it names, or of the first
/// parameter of the set when it names none. When a set of several is refused,
/// each parameter is read alone, so every fault shows at once.
fn build_variants(
    globs: &[&Glob],
    loaded: &Loaded,
    faults: &mut Faults<'_>,
) -> HashMap<(usize, String), Rule> {
    let mut variants = HashMap::new();
    let names: BTreeSet<&str> = globs
        .iter()
        .flat_map(|glob| [&glob.normal, &glob.forced])
        .flat_map(|sets| sets.params.iter().map(|(rule, _)| rule.as_str()))
        .collect();
    let halves: [fn(&Glob) -> &Sets; 2] = [|glob| &glob.normal, |glob| &glob.forced];
    for name in names {
        let Some(index) = loaded.rules.iter().position(|rule| rule.name == name) else {
            continue;
        };
        let mut sets: Vec<Params> = vec![Params::new()];
        for half in halves {
            for glob in globs {
                let held = half(glob);
                let clears = held.styles.as_ref().is_some_and(Vec::is_empty);
                let own = held
                    .params
                    .iter()
                    .find(|(rule, _)| rule == name)
                    .map(|(_, params)| params);
                if !clears && own.is_none() {
                    continue;
                }
                let mut next = sets.clone();
                for set in &sets {
                    let mut matched = if clears { Params::new() } else { set.clone() };
                    matched.extend(
                        own.into_iter()
                            .flatten()
                            .map(|(k, v)| (k.clone(), v.clone())),
                    );
                    next.push(matched);
                }
                let mut seen = HashSet::new();
                next.retain(|set| {
                    seen.insert(variant_key(set.iter().map(|(k, w)| (k.as_str(), &w.param))))
                });
                sets = next;
            }
        }
        for set in sets.into_iter().filter(|set| !set.is_empty()) {
            let params: Vec<(String, Param)> = set
                .iter()
                .map(|(name, written)| (name.clone(), written.param.clone()))
                .collect();
            let key = variant_key(set.iter().map(|(k, w)| (k.as_str(), &w.param)));
            match loaded.rules[index].with_params(&loaded.packages, &loaded.accepted, &params) {
                Ok(rule) => {
                    variants.insert((index, key), rule);
                }
                Err(error) => {
                    // Each parameter alone, so a fault of one does not hide
                    // the fault of another; a fault of the set as a whole is
                    // what the set gave.
                    let alone: Vec<LoadProblem> = if params.len() > 1 {
                        params
                            .iter()
                            .filter_map(|one| {
                                loaded.rules[index]
                                    .with_params(
                                        &loaded.packages,
                                        &loaded.accepted,
                                        slice::from_ref(one),
                                    )
                                    .err()
                            })
                            .flat_map(|error| error.problems)
                            .collect()
                    } else {
                        Vec::new()
                    };
                    let problems = if alone.is_empty() {
                        error.problems
                    } else {
                        alone
                    };
                    for problem in problems {
                        let at = set.get(&problem.field).or_else(|| set.values().next());
                        let message = if problem.message.starts_with("rule `") {
                            problem.message
                        } else {
                            format!("rule `{name}`: {}", problem.message)
                        };
                        let fault = LoadProblem {
                            file: at.map_or_else(String::new, |written| written.file.clone()),
                            line: at.and_then(|written| written.line),
                            column: None,
                            field: problem.field,
                            message,
                            unrunnable: false,
                        };
                        if !faults.problems.contains(&fault) {
                            faults.problems.push(fault);
                        }
                    }
                }
            }
        }
    }
    variants
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
    /// Whether a glob uses the style `Vale`: it lists the style, names a rule
    /// of it, or sets the parameters of one.
    uses_vale: bool,
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
        for (pattern, settings) in &layer.raw.files.0 {
            let glob = FieldPath::root().key("files").key(pattern);
            for (key, Tagged(setting, _)) in &settings.0 {
                self.uses_vale |= key.starts_with("Vale.")
                    || match (key.as_str(), setting) {
                        ("styles", Setting::List(names)) => names.iter().any(|name| name == "Vale"),
                        ("params", Setting::Map(rules)) => {
                            rules.0.iter().any(|(rule, _)| rule.starts_with("Vale."))
                        }
                        _ => false,
                    };
                if let ("vocab", Setting::List(names)) = (key.as_str(), setting) {
                    let list = glob.clone().key("vocab");
                    for (index, name) in names.iter().enumerate() {
                        let at = line_of(&layer.input, &list.clone().index(index));
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

/// The folders that may hold the affix file a YAML vocabulary named `name`
/// names: the dictionaries folder of the package that holds the vocabulary,
/// then that of every other package, in order.
fn affix_folders(found: &[Found], name: &str) -> Vec<PathBuf> {
    let holds = |package: &&Found| vocabulary::find(&package.folder, name).is_some();
    found
        .iter()
        .filter(holds)
        .chain(found.iter().filter(|package| !holds(package)))
        .map(|package| package.folder.join(vocabulary::DICTIONARIES))
        .collect()
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
            [(folder, shown)] => match vocabulary::read(folder, shown, &affix_folders(found, name))
            {
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

/// Builds the rules of the style `Vale` and of the vocabularies the config
/// names, as Vale builds them.
/// The style `Vale` holds `Vale.Repetition`, and `Vale.Spelling` when a glob
/// uses the style, and the lists of the top level give it `Vale.Terms` and
/// `Vale.Avoid`. Each vocabulary a glob lists gives `Vale.<Name>.Terms` and
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
        // The style `Vale` holds `Vale.Repetition`, and `Vale.Spelling` when a
        // glob uses the style, as Vale builds a rule when it is used.
        let built_in = if style {
            vale_repetition(accepted).and_then(|repetition| {
                let mut rules = vec![repetition];
                if vocab.uses_vale {
                    rules.push(vale_spelling(&loaded.packages, accepted)?);
                }
                Ok(rules)
            })
        } else {
            Ok(Vec::new())
        };
        let rules = built_in.and_then(|mut rules| {
            rules.extend(vocabulary_rules(
                &prefix,
                &own_accepted,
                &own_rejected,
                accepted,
            )?);
            Ok(rules)
        });
        match rules {
            Ok(rules) if style || !rules.is_empty() => {
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

#[cfg(test)]
mod tests;
