//! `hakkiri check`: load the config, find the files, lint them on a pool of
//! threads, and gather what they report.

use std::collections::BTreeMap;
use std::num::NonZeroUsize;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{fs, io, thread};

use hakkiri::alert::Alert;
use hakkiri::control::apply_levels;
use hakkiri::engine::lint_accepting;
use hakkiri::rule::{Level, Unrunnable};
use hakkiri::{Tree, parse_ignoring};

use crate::cli::CheckArgs;
use crate::config::{self, DEFAULT, Plan};
use crate::files::{Job, Options, find};
use crate::report::{Check, Failure, Row};

/// The stack of a lint thread: as much as the main thread has, because a deep
/// document takes a deep walk of its tree.
const STACK: usize = 8 * 1024 * 1024;

/// Runs `check`.
pub(crate) fn check(args: &CheckArgs) -> Check {
    let config = args.config.as_deref().unwrap_or_else(|| Path::new(DEFAULT));
    let project = match config::load(config, args.cache_dir.as_deref()) {
        Ok(project) => project,
        Err(error) => {
            let level = args.fail_level.unwrap_or_default();
            return Check::stopped(&error, Level::from(level));
        }
    };
    let fail_level = Level::from(args.fail_level.or(project.fail_level).unwrap_or_default());
    let options = Options {
        gitignore: args.gitignore(project.gitignore),
        on_unmatched: args
            .on_unmatched
            .or(project.on_unmatched)
            .unwrap_or_default(),
    };
    let found = find(&project, &args.paths, options);
    for warning in &found.warnings {
        eprintln!("warning {warning}");
    }
    let (jobs, mut failures) = (found.jobs, found.failures);
    if !found.reached {
        failures.push(Failure::root(
            config,
            format!(
                "no path given lies under the project root `{}`, so nothing was linted",
                project.root().display()
            ),
        ));
    }
    let outcomes = match parallel(&jobs, lint_file) {
        Ok(outcomes) => outcomes,
        Err(error) => {
            failures.push(Failure::file(
                ".",
                format!("cannot start a thread: {error}"),
            ));
            Vec::new()
        }
    };
    // Each rule hakkiri cannot run fails the run once, however many files
    // the config runs it on.
    let held: BTreeMap<&str, &Unrunnable> = jobs
        .iter()
        .flat_map(|job| job.plan.unrunnable.iter().copied())
        .map(|rule| (rule.name.as_str(), rule))
        .collect();
    failures.extend(held.into_values().map(Failure::unrunnable));
    let mut alerts = Vec::new();
    let mut linted = 0;
    for (job, outcome) in jobs.iter().zip(outcomes) {
        match outcome {
            Ok(found) => {
                linted += 1;
                alerts.extend(
                    found
                        .into_iter()
                        .filter(|alert| project.level.is_none_or(|lowest| alert.level >= lowest))
                        .map(|alert| Row::new(&job.path, alert)),
                );
            }
            Err(message) => failures.push(Failure::file(&job.path, message)),
        }
    }
    Check::new(alerts, failures, linted, fail_level)
}

/// The alerts of one file, at the levels the config sets, or why it gave none.
fn lint_file(job: &Job<'_>) -> Result<Vec<Alert>, String> {
    let source = fs::read_to_string(&job.file).map_err(|error| error.to_string())?;
    let tree = parse_ignoring(
        job.language,
        &source,
        &job.plan.block_ignores,
        &job.plan.token_ignores,
    )
    .map_err(|error| error.to_string())?;
    lint_planned(&job.plan, &tree, &source, &job.extension)
}

/// The alerts of `source` under `plan`, at the levels the config sets.
pub(crate) fn lint_planned(
    plan: &Plan<'_>,
    tree: &Tree,
    source: &str,
    extension: &str,
) -> Result<Vec<Alert>, String> {
    let mut alerts = lint_accepting(&plan.rules, tree, source, extension, &plan.accepted)
        .map_err(|error| error.to_string())?;
    for alert in &mut alerts {
        if let Some(level) = plan.levels.get(alert.rule.as_str()) {
            alert.level = *level;
        }
    }
    apply_levels(tree, source, &mut alerts);
    Ok(alerts)
}

/// `work` on every item, on as many threads as the machine has, and the
/// results in the order of the items.
///
/// The threads take items from one shared index, so a slow file holds up
/// only its own thread.
///
/// # Errors
///
/// Returns the failure to start a thread.
fn parallel<T: Sync, R: Send>(items: &[T], work: impl Fn(&T) -> R + Sync) -> io::Result<Vec<R>> {
    let threads = thread::available_parallelism()
        .map_or(1, NonZeroUsize::get)
        .min(items.len());
    let next = AtomicUsize::new(0);
    let mut done = thread::scope(|scope| -> io::Result<Vec<(usize, R)>> {
        let mut workers = Vec::new();
        for _ in 0..threads {
            workers.push(
                thread::Builder::new()
                    .stack_size(STACK)
                    .spawn_scoped(scope, || {
                        let mut out = Vec::new();
                        loop {
                            let index = next.fetch_add(1, Ordering::Relaxed);
                            let Some(item) = items.get(index) else {
                                break;
                            };
                            out.push((index, work(item)));
                        }
                        out
                    })?,
            );
        }
        let mut done = Vec::new();
        for worker in workers {
            match worker.join() {
                Ok(out) => done.extend(out),
                Err(panic) => std::panic::resume_unwind(panic),
            }
        }
        Ok(done)
    })?;
    done.sort_by_key(|(index, _)| *index);
    Ok(done.into_iter().map(|(_, result)| result).collect())
}
