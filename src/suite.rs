//! `hakkiri test`: find the files that hold cases, run each case, and say which
//! rules no case made flag anything.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

use hakkiri::alert::Alert;
use hakkiri::rule::{self, LoadError, Rule, TestCase, alert_lines, cases_from_text};
use hakkiri::{Language, parse};

use crate::cli::TestArgs;
use crate::config::{self, DEFAULT, Project};
use crate::files::is_skipped;
use crate::report::{CaseResult, Failure, Tests};
use crate::run::lint_planned;

/// The suffix of a file that holds only cases.
const TEST_FILE: &str = ".test.yml";

/// Runs `test`.
pub(crate) fn test(args: &TestArgs) -> Tests {
    let (files, mut failures) = find(&args.paths, &|_| false, &|_| true);
    let mut runner = Runner {
        config: args
            .config
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT)),
        project: None,
        rules: HashMap::new(),
        results: Vec::new(),
        files: 0,
        fired: HashSet::new(),
    };
    for file in &files {
        if let Err(failure) = runner.file(file) {
            failures.push(failure);
        }
    }
    if runner.results.is_empty() {
        failures.push(Failure::file(
            &paths_shown(&args.paths),
            "no case found: a rule file with `tests` or a `*.test.yml` file holds them",
        ));
    }
    let uncovered = if args.coverage {
        let (rule_files, rule_failures) = find(&args.paths, &skips_inert, &is_rule_file);
        failures.extend(rule_failures);
        let mut named: Vec<(String, Vec<String>)> = Vec::new();
        for file in &rule_files {
            let Ok((name, _)) = runner.style_of(file) else {
                continue;
            };
            match named.iter_mut().find(|(known, _)| *known == name) {
                Some((_, files)) => files.push(plain(file)),
                None => named.push((name, vec![plain(file)])),
            }
        }
        named
            .into_iter()
            .filter(|(name, _)| !runner.fired.contains(name))
            .map(|(name, files)| match files.as_slice() {
                [_] => name,
                _ => format!("{name} (named by {})", files.join(", ")),
            })
            .collect()
    } else {
        Vec::new()
    };
    Tests::new(
        runner.files,
        runner.results,
        uncovered,
        failures,
        args.coverage,
    )
}

/// `path` as a report prints it: `/` between the parts and no `.` part.
fn plain(path: &Path) -> String {
    path.components()
        .filter(|part| !matches!(part, Component::CurDir))
        .map(|part| part.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

/// The paths a run was given, as a failure names them.
fn paths_shown(paths: &[PathBuf]) -> String {
    if paths.is_empty() {
        ".".to_owned()
    } else {
        paths
            .iter()
            .map(|path| plain(path))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Whether a walk leaves a folder named `name` alone when it lists rules: a
/// dot or an underscore folder holds what the style loader ignores.
fn skips_inert(name: &str) -> bool {
    name.starts_with('.') || name.starts_with('_')
}

/// The `.yml` files `paths` name or hold, in sorted order, and the paths that
/// could not be read.
///
/// A file is taken as given. A walk skips a folder when it is a dot folder,
/// when `inert` says so, or when `check`'s walk skips it, except the folder it
/// was given, and keeps a file when `keep` says so.
fn find(
    paths: &[PathBuf],
    inert: &dyn Fn(&str) -> bool,
    keep: &dyn Fn(&Path) -> bool,
) -> (Vec<PathBuf>, Vec<Failure>) {
    let roots = if paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        paths.to_vec()
    };
    let mut found = Vec::new();
    let mut failures = Vec::new();
    for root in roots {
        match fs::metadata(&root) {
            Ok(meta) if meta.is_dir() => walk(&root, inert, keep, &mut found, &mut failures),
            Ok(_) => {
                if keep(&root) {
                    found.push(root);
                }
            }
            Err(error) => failures.push(Failure::file(&plain(&root), error.to_string())),
        }
    }
    found.sort();
    found.dedup();
    (found, failures)
}

/// Adds the `.yml` files under `directory` that `keep` accepts to `found`.
fn walk(
    directory: &Path,
    inert: &dyn Fn(&str) -> bool,
    keep: &dyn Fn(&Path) -> bool,
    found: &mut Vec<PathBuf>,
    failures: &mut Vec<Failure>,
) {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) => {
            failures.push(Failure::file(&plain(directory), error.to_string()));
            return;
        }
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        if kind.is_dir() {
            if !name.starts_with('.') && !inert(&name) && !is_skipped(&entry.file_name()) {
                walk(&path, inert, keep, found, failures);
            }
        } else if path.extension().is_some_and(|extension| extension == "yml") && keep(&path) {
            found.push(path);
        }
    }
}

/// Whether the text has a top-level `key:` line.
fn has_key(text: &str, key: &str) -> bool {
    text.lines().any(|line| line.starts_with(key))
}

/// Whether `file` is a rule file: YAML that holds `extends`, and is no test
/// file.
fn is_rule_file(file: &Path) -> bool {
    !file.to_string_lossy().ends_with(TEST_FILE)
        && fs::read_to_string(file).is_ok_and(|text| has_key(&text, "extends:"))
}

/// `file` made absolute, with its `.` and `..` parts folded away and its
/// symlinks left alone, as Go's `filepath.Abs` does for Vale.
///
/// The fold ignores symlinks on purpose: `a/link/../b` is `a/b` even when
/// `link` points elsewhere, where the operating system would step out of the
/// link's target. A rule's text, name and package all come from this one path,
/// so they name one file. The style loader names a rule from the folder it was
/// given, so a style folder that is a symlink keeps the name of the link.
fn absolute(file: &Path) -> Option<PathBuf> {
    let mut clean = PathBuf::new();
    for part in std::path::absolute(file).ok()?.components() {
        match part {
            Component::ParentDir => {
                clean.pop();
            }
            Component::CurDir => {}
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => clean.push(part),
        }
    }
    Some(clean)
}

/// Where a case runs.
enum Setup<'a> {
    /// Against this rule alone.
    Rule(&'a Rule),
    /// Against the rule file at this path alone.
    File(PathBuf),
    /// Through the project config.
    Project,
}

/// What a run has done so far.
struct Runner {
    config: PathBuf,
    /// The project config, read when the first case needs it.
    project: Option<Result<Project, String>>,
    /// The rule files cases named, read once.
    rules: HashMap<PathBuf, Result<Rule, String>>,
    results: Vec<CaseResult>,
    /// The files that held cases.
    files: usize,
    /// The rules that flagged something in some case.
    fired: HashSet<String>,
}

/// A load error on one line.
fn one_line(error: &LoadError) -> String {
    error.to_string().replace('\n', "; ")
}

impl Runner {
    /// Reads the cases of `file` and runs each one.
    ///
    /// # Errors
    ///
    /// Returns why the file's cases could not be read.
    fn file(&mut self, file: &Path) -> Result<(), Failure> {
        let shown = plain(file);
        let unread = |message: String| Failure::file(&shown, message);
        let folded =
            absolute(file).ok_or_else(|| unread("the path has no absolute form".to_owned()))?;
        if shown.ends_with(TEST_FILE) {
            let text = fs::read_to_string(&folded).map_err(|error| unread(error.to_string()))?;
            let cases = cases_from_text(&shown, &text).map_err(|error| unread(one_line(&error)))?;
            self.run_all(file, &cases, None);
            return Ok(());
        }
        let text = fs::read_to_string(&folded).map_err(|error| unread(error.to_string()))?;
        if !has_key(&text, "extends:") || !has_key(&text, "tests:") {
            return Ok(());
        }
        let (name, package) = self.style_of(file).map_err(unread)?;
        let rule = Rule::from_text_in(&name, &shown, &text, &package)
            .map_err(|error| unread(one_line(&error)))?;
        let cases = rule.tests.clone();
        self.run_all(file, &cases, Some(&rule));
        Ok(())
    }

    /// Runs `cases`, read from `file`, each against `own` when it is given.
    fn run_all(&mut self, file: &Path, cases: &[TestCase], own: Option<&Rule>) {
        if cases.is_empty() {
            return;
        }
        self.files += 1;
        for case in cases {
            let setup = match (own, &case.rule) {
                (Some(rule), _) => Setup::Rule(rule),
                (None, Some(rule)) => {
                    Setup::File(file.parent().unwrap_or_else(|| Path::new(".")).join(rule))
                }
                (None, None) => Setup::Project,
            };
            let outcome = self.lint(case, setup);
            let result = self.result(file, case, outcome);
            self.results.push(result);
        }
    }

    /// The result of `case` given what linting it gave.
    fn result(
        &mut self,
        file: &Path,
        case: &TestCase,
        outcome: Result<Vec<Alert>, String>,
    ) -> CaseResult {
        let mut result = CaseResult {
            file: plain(file),
            name: case.name.clone(),
            passed: false,
            reason: None,
            error: None,
            got: None,
            want: None,
            about: case.about.clone(),
        };
        match outcome {
            Err(error) => result.error = Some(error),
            Ok(alerts) => {
                self.fired
                    .extend(alerts.iter().map(|alert| alert.rule.clone()));
                let got = alert_lines(&alerts);
                result.reason = case.verdict(&got);
                result.passed = result.reason.is_none();
                if !result.passed {
                    result.got = Some(got);
                    result.want.clone_from(&case.want);
                }
            }
        }
        result
    }

    /// The alerts `case` gets under `setup`, or why it could not run.
    fn lint(&mut self, case: &TestCase, setup: Setup<'_>) -> Result<Vec<Alert>, String> {
        let name = format!("input.{}", case.format);
        let language = Language::from_path(Path::new(&name))
            .ok_or_else(|| format!("no language reads the format `{}`", case.format))?;
        let tree = parse(language, &case.input).map_err(|error| error.to_string())?;
        match setup {
            Setup::Rule(rule) => lint_alone(rule, &tree, case),
            Setup::File(path) => {
                let rule = self.rule_at(&path)?;
                lint_alone(rule, &tree, case)
            }
            Setup::Project => {
                let project = self.project()?;
                let Some(plan) = project.plan(&name) else {
                    return Ok(Vec::new());
                };
                let mut alerts = lint_planned(&plan, &tree, &case.input, &case.format)?;
                alerts.retain(|alert| project.level.is_none_or(|lowest| alert.level >= lowest));
                Ok(alerts)
            }
        }
    }

    /// The project config, read once.
    fn project(&mut self) -> Result<&Project, String> {
        let config = &self.config;
        self.project
            .get_or_insert_with(|| config::load(config).map_err(|error| one_line(&error)))
            .as_ref()
            .map_err(Clone::clone)
    }

    /// The packages of the project config, which name a rule file by the style
    /// folder it sits in. None when there is no readable config.
    fn packages(&mut self) -> Vec<PathBuf> {
        self.project()
            .map(|project| project.packages.clone())
            .unwrap_or_default()
    }

    /// The name and the package of the rule at `file`, as the style loader
    /// gives them.
    ///
    /// Under a package of the project config, the style is the first folder
    /// below the package, and the folders after it join the name.
    /// Elsewhere the style is the folder that holds the file, as Vale names an
    /// isolated rule.
    /// The path is made absolute with symlinks left alone, so a style folder
    /// that is a link keeps the name of the link.
    /// A folder above the file is the package when its real path is one of the
    /// config's, which are real paths, so a project reached through a link
    /// names its rules as `check` does.
    ///
    /// # Errors
    ///
    /// Returns why the rule has no name.
    fn style_of(&mut self, file: &Path) -> Result<(String, PathBuf), String> {
        let packages = self.packages();
        let absolute = absolute(file).ok_or("the path cannot be made absolute")?;
        let candidates = [Some(absolute.clone()), fs::canonicalize(file).ok()];
        for candidate in candidates.iter().flatten() {
            for package in candidate.ancestors().skip(1) {
                if !fs::canonicalize(package).is_ok_and(|real| packages.contains(&real)) {
                    continue;
                }
                let Ok(relative) = candidate.strip_prefix(package) else {
                    continue;
                };
                let mut parts = relative.components();
                if let (Some(first), Some(_)) = (parts.next(), parts.next()) {
                    let style = package.join(first);
                    return Ok((rule::rule_name(&style, candidate)?, package.to_path_buf()));
                }
            }
        }
        let style = absolute.parent().ok_or("the rule has no style folder")?;
        let package = style.parent().ok_or("the rule has no package")?;
        Ok((rule::rule_name(style, &absolute)?, package.to_path_buf()))
    }

    /// The rule file at `path`, read once.
    fn rule_at(&mut self, path: &Path) -> Result<&Rule, String> {
        if !self.rules.contains_key(path) {
            let read = self.read_rule(path);
            self.rules.insert(path.to_path_buf(), read);
        }
        self.rules
            .get(path)
            .ok_or_else(|| format!("{}: the rule was not kept", plain(path)))?
            .as_ref()
            .map_err(Clone::clone)
    }

    /// Reads the rule file at `path`.
    fn read_rule(&mut self, path: &Path) -> Result<Rule, String> {
        let shown = plain(path);
        let folded =
            absolute(path).ok_or_else(|| format!("{shown}: the path has no absolute form"))?;
        let text = fs::read_to_string(&folded).map_err(|error| format!("{shown}: {error}"))?;
        let (name, package) = self
            .style_of(path)
            .map_err(|message| format!("{shown}: {message}"))?;
        Rule::from_text_in(&name, &shown, &text, &package).map_err(|error| one_line(&error))
    }
}

/// Lints the case's input with `rule` alone, at every level.
fn lint_alone(rule: &Rule, tree: &hakkiri::Tree, case: &TestCase) -> Result<Vec<Alert>, String> {
    hakkiri::engine::lint(&[rule], tree, &case.input, &case.format)
        .map_err(|error| error.to_string())
}
