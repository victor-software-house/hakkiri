//! The command-line grammar: one struct of arguments per verb. A flag's doc
//! comment is its `--help` text.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use ctl_core::{ColorLong, EnvPrefix, FormatLong};
use hakkiri::rule::Level;
use serde::Deserialize;

/// The whole command line. Every long option also reads `HAKKIRI_<LONG>`, such
/// as `HAKKIRI_CONFIG` or `HAKKIRI_FORMAT`; the command line wins over the
/// variable.
#[derive(Debug, Parser)]
#[command(
    version,
    about = "A prose linter for Markdown, MDX and source comments",
    arg_required_else_help = true,
    env_prefix = "HAKKIRI"
)]
pub(crate) struct Cli {
    /// The verb to run.
    #[command(subcommand)]
    pub(crate) command: Command,

    /// `--format`, shared by every verb.
    #[command(flatten)]
    pub(crate) format: FormatLong,

    /// `--color` and `--no-color`, shared by every verb.
    #[command(flatten)]
    pub(crate) color: ColorLong,
}

/// The verbs.
#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// Lint files and directories under the project config.
    Check(CheckArgs),
    /// Run the test cases of rule files and of `*.test.yml` files.
    Test(TestArgs),
    /// Print the installed-version operator contract.
    Instructions,
}

/// Arguments of `hakkiri test`.
#[derive(Args, Debug)]
pub(crate) struct TestArgs {
    /// Rule files, test files and directories that hold them. Defaults to the
    /// current directory.
    #[arg(value_hint = clap::ValueHint::AnyPath)]
    pub(crate) paths: Vec<PathBuf>,

    /// The project config, read only by a case that runs through it. Defaults
    /// to `.ctl/hakkiri.yaml`.
    #[arg(long, value_hint = clap::ValueHint::FilePath)]
    pub(crate) config: Option<PathBuf>,

    /// The folder that keeps fetched packages, read only by a case that runs
    /// through a config that pins a package by `url`. Defaults to the per-user
    /// cache folder `hakkiri`.
    #[arg(long, value_hint = clap::ValueHint::DirPath)]
    pub(crate) cache_dir: Option<PathBuf>,

    /// Fail when a rule under the paths flags nothing in any case.
    #[arg(long)]
    pub(crate) coverage: bool,
}

/// Arguments of `hakkiri check`.
#[derive(Args, Debug)]
pub(crate) struct CheckArgs {
    /// Files and directories to lint. Defaults to the current directory.
    #[arg(value_hint = clap::ValueHint::AnyPath)]
    pub(crate) paths: Vec<PathBuf>,

    /// The project config. Defaults to `.ctl/hakkiri.yaml`.
    #[arg(long, value_hint = clap::ValueHint::FilePath)]
    pub(crate) config: Option<PathBuf>,

    /// The folder that keeps fetched packages. Defaults to the per-user cache
    /// folder `hakkiri`.
    #[arg(long, value_hint = clap::ValueHint::DirPath)]
    pub(crate) cache_dir: Option<PathBuf>,

    /// The lowest level of an alert that makes the run exit 1. Defaults to the
    /// config's `fail_level`, then to `error`.
    #[arg(long, value_enum)]
    pub(crate) fail_level: Option<FailLevel>,

    /// Whether a walk skips what git ignores: `--gitignore=false` reads it.
    /// Defaults to the config's `gitignore`, then to `true`.
    #[arg(
        long,
        value_name = "BOOL",
        num_args = 0..=1,
        require_equals = true,
        default_missing_value = "true",
        conflicts_with = "no_gitignore"
    )]
    pub(crate) gitignore: Option<bool>,

    /// Read the files git ignores: the same as `--gitignore=false`.
    #[arg(long)]
    pub(crate) no_gitignore: bool,

    /// What to do with a file named on the command line that no glob selects.
    /// Defaults to the config's `on_unmatched`, then to `warn`.
    #[arg(long, value_enum)]
    pub(crate) on_unmatched: Option<OnUnmatched>,
}

impl CheckArgs {
    /// Whether a walk skips what git ignores: `--no-gitignore`, else
    /// `--gitignore`, which also reads its variable, else the config's value,
    /// else yes.
    pub(crate) fn gitignore(&self, config: Option<bool>) -> bool {
        if self.no_gitignore {
            false
        } else {
            self.gitignore.or(config).unwrap_or(true)
        }
    }
}

/// What a run does with a file named on the command line that no glob selects.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub(crate) enum OnUnmatched {
    /// Print a warning and go on.
    #[default]
    Warn,
    /// Fail the run: exit 2.
    Error,
    /// Say nothing.
    Ignore,
}

/// The levels `--fail-level` takes, and the config's `fail_level`.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub(crate) enum FailLevel {
    /// Any alert fails the run.
    Suggestion,
    /// A warning or an error fails the run.
    Warning,
    /// Only an error fails the run.
    #[default]
    Error,
}

impl From<FailLevel> for Level {
    fn from(level: FailLevel) -> Self {
        match level {
            FailLevel::Suggestion => Self::Suggestion,
            FailLevel::Warning => Self::Warning,
            FailLevel::Error => Self::Error,
        }
    }
}
