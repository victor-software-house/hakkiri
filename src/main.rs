//! The `hakkiri` command: it parses the arguments, runs one verb, and prints
//! its result. ctl-core's `App` owns parsing, help and output; `hakkiri
//! instructions` prints the operator contract built into the binary.

mod cli;
mod config;
mod files;
mod report;
mod run;
mod suite;

use ctl_core::prelude::{App, ExitCode};

use crate::cli::{Cli, Command};
use crate::report::{Instructions, Output};

/// The operator contract, read at build time.
const INSTRUCTIONS: &str = include_str!("instructions.md");

fn main() -> ExitCode {
    App::<Cli>::new("hakkiri")
        .view(|cli| cli.format.view(cli.color.color()))
        .run(|cli| {
            Ok::<_, anyhow::Error>(match cli.command {
                Command::Check(args) => Output::Check(run::check(&args)),
                Command::Test(args) => Output::Test(suite::test(&args)),
                Command::Instructions => Output::Instructions(Instructions {
                    markdown: INSTRUCTIONS.to_owned(),
                }),
            })
        })
}
