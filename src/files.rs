//! Finding the files a run lints: the files and directories it is given, and
//! the rules the config runs on each.

use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs;
use std::path::{Component, Path, PathBuf};

use hakkiri::Language;
use ignore::WalkBuilder;

use crate::cli::OnUnmatched;
use crate::config::{Plan, Project, leads_to};
use crate::report::Failure;

/// The folders a walk never enters.
const SKIPPED: [&str; 2] = ["node_modules", ".git"];

/// Whether a walk never enters a folder named `name`.
pub(crate) fn is_skipped(name: &OsStr) -> bool {
    SKIPPED.iter().any(|skipped| name == *skipped)
}

/// One file to lint.
#[derive(Debug)]
pub(crate) struct Job<'a> {
    /// The path as the report prints it: relative to the current directory
    /// where it can be, with `/` separators.
    pub(crate) path: String,
    /// Where the file is.
    pub(crate) file: PathBuf,
    /// The language its extension names.
    pub(crate) language: Language,
    /// Its extension, without the dot.
    pub(crate) extension: String,
    /// The rules that run on it.
    pub(crate) plan: Plan<'a>,
}

/// How a run chooses its files.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Options {
    /// Whether a walk skips what git ignores.
    pub(crate) gitignore: bool,
    /// What to do with a named file that no glob selects.
    pub(crate) on_unmatched: OnUnmatched,
}

/// What a walk finds, in the order it finds it.
struct Finder<'a> {
    project: &'a Project,
    options: Options,
    current: Option<PathBuf>,
    warnings: Vec<String>,
    seen: HashSet<String>,
    jobs: Vec<Job<'a>>,
    failures: Vec<Failure>,
}

/// The path as a report prints it: no `.` part unless that is all there is,
/// `/` between the parts, and relative to `current` when it lies inside it.
fn display(path: &Path, current: Option<&Path>) -> String {
    let relative = current
        .and_then(|current| path.strip_prefix(current).ok())
        .unwrap_or(path);
    let shown = relative
        .components()
        .filter(|part| !matches!(part, Component::CurDir))
        .map(|part| part.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/");
    if shown.is_empty() {
        ".".to_owned()
    } else {
        shown
    }
}

impl Finder<'_> {
    /// Lints `file` when a language reads it and a glob selects it.
    ///
    /// A file the run was given by name that no language reads is a failure,
    /// and a file a walk found is skipped.
    fn file(&mut self, file: &Path, named: bool) {
        let path = display(file, self.current.as_deref());
        let Some(language) = Language::from_path(file) else {
            if named {
                self.failures.push(Failure::file(
                    &path,
                    "no language reads this file: hakkiri reads Markdown, MDX, Rust, \
                     TypeScript, TSX and JavaScript files by their extension",
                ));
            }
            return;
        };
        let Some(relative) = self.project.relative(file) else {
            return;
        };
        if !self.seen.insert(relative.clone()) {
            return;
        }
        let Some(plan) = self.project.plan(&relative) else {
            if named && !self.project.selects(&relative) {
                self.unmatched(&path);
            }
            return;
        };
        let extension = file
            .extension()
            .map(|extension| extension.to_string_lossy().into_owned())
            .unwrap_or_default();
        self.jobs.push(Job {
            path,
            file: file.to_path_buf(),
            language,
            extension,
            plan,
        });
    }

    /// Says what a run does with the named file `path`, which no glob selects.
    fn unmatched(&mut self, path: &str) {
        let message = "no glob selects it, so it is not linted";
        match self.options.on_unmatched {
            OnUnmatched::Warn => self.warnings.push(format!("{path}: {message}")),
            OnUnmatched::Error => self.failures.push(Failure::file(path, message)),
            OnUnmatched::Ignore => {}
        }
    }

    /// Walks the directory `directory` in name order, leaving alone what
    /// [`Finder::skips`] names and, unless the run reads them, the files and
    /// folders that `.gitignore`, `.git/info/exclude` and the global git
    /// excludes ignore.
    fn walk(&mut self, directory: &Path) {
        let root = self.project.root().to_path_buf();
        let packages = self.project.packages.clone();
        let git = self.options.gitignore;
        let walker = WalkBuilder::new(directory)
            .hidden(false)
            .ignore(false)
            .parents(git)
            .git_ignore(git)
            .git_global(git)
            .git_exclude(git)
            .sort_by_file_path(Path::cmp)
            .filter_entry(move |entry| {
                entry.depth() == 0
                    || !entry.file_type().is_some_and(|kind| kind.is_dir())
                    || !skips(&root, &packages, entry.path())
            })
            .build();
        for entry in walker {
            match entry {
                Ok(entry) => {
                    let path = entry.path();
                    if entry.file_type().is_some_and(|kind| kind.is_file()) || path.is_file() {
                        self.file(path, false);
                    }
                }
                Err(error) => {
                    let at = if let ignore::Error::WithPath { path, .. } = &error {
                        path.as_path()
                    } else {
                        directory
                    };
                    let shown = display(at, self.current.as_deref());
                    self.failures.push(Failure::file(&shown, error.to_string()));
                }
            }
        }
    }
}

/// Whether a walk leaves the directory `path` alone: `node_modules`, `.git`, a
/// folder of a package, or a folder that is neither under the project `root`
/// nor on the way to it.
fn skips(root: &Path, packages: &[PathBuf], path: &Path) -> bool {
    path.file_name().is_some_and(is_skipped)
        || fs::canonicalize(path)
            .is_ok_and(|real| packages.contains(&real) || !leads_to(root, &real))
}

/// What a search found.
pub(crate) struct Found<'a> {
    /// The files to lint.
    pub(crate) jobs: Vec<Job<'a>>,
    /// The paths that could not be read.
    pub(crate) failures: Vec<Failure>,
    /// One line for each path given that lies outside the project root.
    pub(crate) warnings: Vec<String>,
    /// Whether any path given lies under the project root, or holds it.
    pub(crate) reached: bool,
}

/// The files to lint for `paths`, or for the current directory when it is
/// empty.
pub(crate) fn find<'a>(project: &'a Project, paths: &[PathBuf], options: Options) -> Found<'a> {
    let mut finder = Finder {
        project,
        options,
        current: std::env::current_dir().ok(),
        warnings: Vec::new(),
        seen: HashSet::new(),
        jobs: Vec::new(),
        failures: Vec::new(),
    };
    let roots = if paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        paths.to_vec()
    };
    let mut reached = false;
    for root in roots {
        match fs::metadata(&root) {
            Ok(meta) if !project.reaches(&root, meta.is_dir()) => finder.warnings.push(format!(
                "{}: outside the project root, so no glob matches it and it is not linted",
                display(&root, finder.current.as_deref())
            )),
            Ok(meta) if meta.is_dir() => {
                reached = true;
                finder.walk(&root);
            }
            Ok(_) => {
                reached = true;
                finder.file(&root, true);
            }
            Err(error) => {
                reached = true;
                let path = display(&root, finder.current.as_deref());
                finder
                    .failures
                    .push(Failure::file(&path, error.to_string()));
            }
        }
    }
    Found {
        jobs: finder.jobs,
        failures: finder.failures,
        warnings: finder.warnings,
        reached,
    }
}
