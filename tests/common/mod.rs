//! What every integration test of the binary shares: the binary with a clean
//! environment, and a run that reads its status and both streams.

use std::env;
use std::error::Error;
use std::path::Path;
use std::process::Command;

/// The variables a caller's shell may set that change what hakkiri prints.
/// Agent shells export `CLICOLOR_FORCE=1` and `FORCE_COLOR=3`.
pub(crate) const CALLER_ENV: [&str; 5] = [
    "CLICOLOR",
    "CLICOLOR_FORCE",
    "FORCE_COLOR",
    "NO_COLOR",
    "COLORTERM",
];

/// The hakkiri binary with none of [`CALLER_ENV`] and no `HAKKIRI_*` variable,
/// so a test sees the same output in an agent shell, a terminal and CI. A test
/// about colour or a variable sets its own back on top.
pub(crate) fn binary() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_hakkiri"));
    for name in CALLER_ENV {
        command.env_remove(name);
    }
    for (name, _) in env::vars_os() {
        if name.to_string_lossy().starts_with("HAKKIRI_") {
            command.env_remove(name);
        }
    }
    command
}

/// What a run gave: its exit status and its two streams.
#[derive(Debug)]
pub(crate) struct Run {
    pub(crate) code: i32,
    pub(crate) out: String,
    pub(crate) err: String,
}

impl Run {
    /// The lines of standard output.
    pub(crate) fn lines(&self) -> Vec<&str> {
        self.out.lines().collect()
    }
}

/// Runs hakkiri in `dir` with `args` and the variables `envs`.
pub(crate) fn run(dir: &Path, args: &[&str], envs: &[(&str, &str)]) -> Result<Run, Box<dyn Error>> {
    let output = binary()
        .current_dir(dir)
        .args(args)
        .envs(envs.iter().copied())
        .output()?;
    Ok(Run {
        code: output.status.code().ok_or("killed by a signal")?,
        out: String::from_utf8(output.stdout)?,
        err: String::from_utf8(output.stderr)?,
    })
}
