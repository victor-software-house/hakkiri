//! The operator contract: `hakkiri instructions` prints `src/instructions.md`,
//! and the instructions and the usage skill name every flag, variable and exit
//! code of `check`.

#![expect(
    clippy::panic_in_result_fn,
    reason = "A test returns Result for `?` and fails through an assertion."
)]

use std::collections::BTreeSet;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::Command;

use regex::Regex;

type TestResult = Result<(), Box<dyn Error>>;

/// The text of `relative`, a file of the crate.
fn crate_file(relative: &str) -> Result<String, Box<dyn Error>> {
    Ok(fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join(relative),
    )?)
}

/// The standard output of `hakkiri` with `args`, with colour variables removed.
fn output(args: &[&str]) -> Result<String, Box<dyn Error>> {
    let mut command = Command::new(env!("CARGO_BIN_EXE_hakkiri"));
    for name in [
        "CLICOLOR",
        "CLICOLOR_FORCE",
        "FORCE_COLOR",
        "NO_COLOR",
        "COLORTERM",
    ] {
        command.env_remove(name);
    }
    let output = command.args(args).env("COLUMNS", "1000").output()?;
    Ok(String::from_utf8(output.stdout)?)
}

#[test]
fn the_command_prints_the_instructions_file() -> TestResult {
    let printed = output(&["instructions"])?;
    assert_eq!(
        printed.trim_end(),
        crate_file("src/instructions.md")?.trim_end()
    );
    Ok(())
}

#[test]
fn both_documents_name_every_flag_variable_and_exit_code() -> TestResult {
    let help = output(&["check", "--help"])?;
    let names = Regex::new(r"--[a-z][a-z-]*|HAKKIRI_[A-Z_]+")?;
    let wanted: BTreeSet<&str> = names
        .find_iter(&help)
        .map(|found| found.as_str())
        .filter(|name| !matches!(*name, "--help" | "--version"))
        .collect();
    assert!(wanted.contains("--fail-level"), "{help}");
    assert!(wanted.contains("HAKKIRI_CONFIG"), "{help}");
    for file in ["src/instructions.md", "skills/hakkiri/SKILL.md"] {
        let text = crate_file(file)?;
        for name in &wanted {
            assert!(text.contains(name), "{file} does not name {name}");
        }
        for code in ["| 0 |", "| 1 |", "| 2 |"] {
            assert!(text.contains(code), "{file} has no exit row {code}");
        }
    }
    Ok(())
}

#[test]
fn the_skill_names_the_crate_version() -> TestResult {
    let skill = crate_file("skills/hakkiri/SKILL.md")?;
    let line = format!("version: {}\n", env!("CARGO_PKG_VERSION"));
    assert!(skill.contains(&line), "the skill's version is not {line}");
    Ok(())
}
