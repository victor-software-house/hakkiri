//! The command-line grammar: one struct of arguments per verb. A flag's doc
//! comment is its `--help` text.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use ctl_core::{ColorLong, EnvPrefix, FormatLong};
use hakkiri::rule::Level;

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

    /// The lowest level of an alert that makes the run exit 1.
    #[arg(long, value_enum, default_value_t = FailLevel::Error)]
    pub(crate) fail_level: FailLevel,
}

/// The levels `--fail-level` takes.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum FailLevel {
    /// Any alert fails the run.
    Suggestion,
    /// A warning or an error fails the run.
    Warning,
    /// Only an error fails the run.
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
