//! Finding the files a run lints: the files and directories it is given, and
//! the rules the config runs on each.

use std::collections::HashSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

use hakkiri::Language;

use crate::config::{Plan, Project};
use crate::report::Failure;

/// The folders a walk never enters.
const SKIPPED: [&str; 2] = ["node_modules", ".git"];

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

/// What a walk finds, in the order it finds it.
struct Finder<'a> {
    project: &'a Project,
    current: Option<PathBuf>,
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
        let Some(plan) = self.project.plan(&relative) else {
            return;
        };
        if !self.seen.insert(relative) {
            return;
        }
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

    /// Walks the directory `directory` in name order.
    fn walk(&mut self, directory: &Path) {
        let entries = match fs::read_dir(directory) {
            Ok(entries) => entries,
            Err(error) => {
                let path = display(directory, self.current.as_deref());
                self.failures.push(Failure::file(&path, error.to_string()));
                return;
            }
        };
        let mut paths: Vec<PathBuf> = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .collect();
        paths.sort();
        for path in paths {
            let Ok(kind) = fs::symlink_metadata(&path).map(|meta| meta.file_type()) else {
                continue;
            };
            if kind.is_dir() {
                if !self.skips(&path) {
                    self.walk(&path);
                }
            } else if kind.is_file() || path.is_file() {
                self.file(&path, false);
            }
        }
    }

    /// Whether a walk leaves the directory `path` alone: `node_modules`,
    /// `.git`, a folder of a package, or a folder that is neither under the
    /// project root nor on the way to it.
    fn skips(&self, path: &Path) -> bool {
        path.file_name()
            .is_some_and(|name| SKIPPED.iter().any(|skipped| name == *skipped))
            || fs::canonicalize(path).is_ok_and(|real| {
                self.project.packages.contains(&real) || !self.project.leads_to_root(&real)
            })
    }
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
pub(crate) fn find<'a>(project: &'a Project, paths: &[PathBuf]) -> Found<'a> {
    let mut finder = Finder {
        project,
        current: std::env::current_dir().ok(),
        seen: HashSet::new(),
        jobs: Vec::new(),
        failures: Vec::new(),
    };
    let roots = if paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        paths.to_vec()
    };
    let mut warnings = Vec::new();
    let mut reached = false;
    for root in roots {
        match fs::metadata(&root) {
            Ok(meta) if !project.reaches(&root, meta.is_dir()) => warnings.push(format!(
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
        warnings,
        reached,
    }
}
