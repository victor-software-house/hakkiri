//! The built-in rule `Vale.Spelling`: the scenarios of the vale-spelling
//! change.
//!
//! `tests/fixtures/vale-spelling` holds three packages (`plain`, with a
//! vocabulary `Team`; `lists`, with word lists under `config/ignore`; and
//! `dicts`, with a dictionary under `config/dictionaries`), three Markdown
//! files and one Rust file (`text`, with `.fixture` after the extension), and
//! one config for each tool and scenario: `vale/<name>.ini` and
//! `.ctl/<name>.yaml`.
//! `vale.tsv` holds one row for each alert Vale gave on the files under each
//! config: the config, the file, the line, the first and last column, the
//! level, the rule, the message and the matched text, with a line feed written
//! `\n` and a tab `\t`.
//! The rows come from Vale at `66bbb85` run from the fixture folder as `vale
//! --config=vale/<name>.ini --output=JSON text`, with `Vale.Repetition`
//! turned off, which `tests/builtin_repetition.rs` compares.
//!
//! `departures.tsv` holds one row for each alert that only one side gave,
//! led by the config and `vale` or `hakkiri`.
//! The test fails on any difference that file does not list, and on any row
//! of it that no longer differs.

#![expect(
    clippy::panic_in_result_fn,
    reason = "A test returns Result for `?` and fails through an assertion."
)]

mod common;

use std::collections::BTreeSet;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use common::run;
use ctl_core::fixture::{Root, Suite};
use serde_json::Value;

type TestResult = Result<(), Box<dyn Error>>;

/// The rows of one table, each led by its first cell and holding the rest.
type Table = Vec<(String, Vec<String>)>;

/// A file, a line, the first and last column, the level, the rule, the
/// message and the matched text.
type Row = (String, usize, usize, usize, String, String, String, String);

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vale-spelling")
}

/// Copies `from` into `root` at `to`, dropping `.fixture` from each name.
fn copy_into(root: &Root<'_>, from: &Path, to: &Path) -> TestResult {
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let target = to.join(name.strip_suffix(".fixture").unwrap_or(&name));
        if entry.file_type()?.is_dir() {
            copy_into(root, &entry.path(), &target)?;
        } else {
            root.file(target, fs::read(entry.path())?);
        }
    }
    Ok(())
}

/// A scenario folder that holds the fixture's packages, text and configs.
fn project<'a>(suite: &'a Suite, name: &str) -> Result<Root<'a>, Box<dyn Error>> {
    let root = suite.scenario(name);
    for folder in ["plain", "lists", "dicts", "text", ".ctl"] {
        copy_into(&root, &fixture().join(folder), Path::new(folder))?;
    }
    Ok(root)
}

/// The names of the configs of the fixture, in order.
fn configs() -> Result<Vec<String>, Box<dyn Error>> {
    let mut names = Vec::new();
    for entry in fs::read_dir(fixture().join(".ctl"))? {
        let name = entry?.file_name().to_string_lossy().into_owned();
        names.push(name.trim_end_matches(".yaml").to_owned());
    }
    names.sort();
    Ok(names)
}

/// The rows of one file of the fixture, led by a config name, by config.
fn table(file: &str) -> Result<Table, Box<dyn Error>> {
    let mut rows = Vec::new();
    for line in fs::read_to_string(fixture().join(file))?.lines() {
        let mut cells: Vec<String> = line.split('\t').map(str::to_owned).collect();
        let lead = cells.remove(0);
        rows.push((lead, cells));
    }
    Ok(rows)
}

fn row_of(cells: &[String]) -> Result<Row, Box<dyn Error>> {
    let [file, line, first, last, level, rule, message, matched] = cells else {
        return Err(format!("a row has {} cells: {cells:?}", cells.len()).into());
    };
    Ok((
        file.clone(),
        line.parse()?,
        first.parse()?,
        last.parse()?,
        level.clone(),
        rule.clone(),
        message.clone(),
        matched.replace("\\n", "\n").replace("\\t", "\t"),
    ))
}

/// The rows of a `check --format json` run, whose matches all end on the line
/// they start on.
fn rows_of(out: &str) -> Result<BTreeSet<Row>, Box<dyn Error>> {
    let json: Value = serde_json::from_str(out)?;
    let mut rows = BTreeSet::new();
    for alert in json["alerts"].as_array().ok_or("no alerts array")? {
        let text = |key: &str| alert[key].as_str().unwrap_or_default().to_owned();
        let number = |key: &str| {
            alert[key]
                .as_u64()
                .and_then(|value| usize::try_from(value).ok())
                .unwrap_or_default()
        };
        assert_eq!(
            number("line"),
            number("end_line"),
            "a match over two lines needs its last column mapped: {alert}"
        );
        rows.insert((
            text("path"),
            number("line"),
            number("column"),
            number("end_column"),
            text("level"),
            text("rule"),
            text("message"),
            text("match"),
        ));
    }
    Ok(rows)
}

#[test]
fn every_config_gives_the_alerts_vale_gives() -> TestResult {
    let suite = Suite::new("vale-spelling-parity");
    let root = project(&suite, "parity")?;
    let vale = table("vale.tsv")?;
    let departures = table("departures.tsv")?;
    let names = configs()?;
    let recorded: BTreeSet<&str> = vale.iter().map(|(name, _)| name.as_str()).collect();
    let known: BTreeSet<&str> = names.iter().map(String::as_str).collect();
    assert!(
        recorded.is_subset(&known),
        "vale.tsv holds a config that .ctl does not: {recorded:?}"
    );
    let mut failed = Vec::new();
    for name in &names {
        let config = format!(".ctl/{name}.yaml");
        let result = run(
            root.path(),
            &["check", "--config", &config, "--format", "json", "text"],
            &[],
        )?;
        assert!(
            result.code == 0 || result.code == 1,
            "{name}: {}{}",
            result.out,
            result.err
        );
        let ours = rows_of(&result.out)?;
        let theirs = vale
            .iter()
            .filter(|(config, _)| config == name)
            .map(|(_, cells)| row_of(cells))
            .collect::<Result<BTreeSet<Row>, _>>()?;
        let listed = |side: &str| -> Result<BTreeSet<Row>, Box<dyn Error>> {
            departures
                .iter()
                .filter(|(lead, cells)| lead == name && cells.first().is_some_and(|s| s == side))
                .map(|(_, cells)| row_of(&cells[1..]))
                .collect()
        };
        let (only_vale, only_ours) = (listed("vale")?, listed("hakkiri")?);
        let got_vale: BTreeSet<Row> = theirs.difference(&ours).cloned().collect();
        let got_ours: BTreeSet<Row> = ours.difference(&theirs).cloned().collect();
        if got_vale != only_vale || got_ours != only_ours {
            failed.push(format!(
                "{name}: only Vale gave {got_vale:#?}, listed {only_vale:#?}; \
                 only hakkiri gave {got_ours:#?}, listed {only_ours:#?}"
            ));
        }
    }
    assert!(failed.is_empty(), "{}", failed.join("\n"));
    Ok(())
}

#[test]
fn the_pretty_output_names_the_rule_and_its_message() -> TestResult {
    let suite = Suite::new("vale-spelling-pretty");
    let root = project(&suite, "pretty")?;
    let result = run(
        root.path(),
        &["check", "--config", ".ctl/default.yaml", "text/prose.md"],
        &[],
    )?;
    assert_eq!(result.code, 1, "{}{}", result.out, result.err);
    assert!(
        result
            .lines()
            .iter()
            .any(|line| line.contains("Vale.Spelling")
                && line.contains("Did you really mean 'foxs'?")),
        "{}",
        result.out
    );
    Ok(())
}

/// The words that a `check` of `config` under `files` reports as misspelt.
fn misspelt(
    root: &Root<'_>,
    config: &str,
    files: &str,
) -> Result<BTreeSet<String>, Box<dyn Error>> {
    let result = run(
        root.path(),
        &["check", "--config", config, "--format", "json", files],
        &[],
    )?;
    assert!(
        result.code == 0 || result.code == 1,
        "{}{}",
        result.out,
        result.err
    );
    Ok(rows_of(&result.out)?
        .into_iter()
        .filter(|row| row.5 == "Vale.Spelling")
        .map(|row| row.7)
        .collect())
}

#[test]
fn each_package_adds_its_dictionaries_and_word_lists() -> TestResult {
    let suite = Suite::new("vale-spelling-packages");
    let root = project(&suite, "packages")?;
    root.file(
        ".ctl/both.yaml",
        "packages:\n  - path: lists\n  - path: dicts\nfiles:\n  \"text/*.md\":\n    styles: [Vale]\n    Vale.Repetition: off\n",
    );
    let words = misspelt(&root, ".ctl/both.yaml", "text/prose.md")?;
    for held in ["blorft", "xyz", "foxs", "hakkiri", "Hakkiri"] {
        assert!(!words.contains(held), "{held} was reported: {words:?}");
    }
    assert!(words.contains("vSphere"), "{words:?}");
    Ok(())
}

#[test]
fn a_project_with_no_package_uses_the_built_in_dictionary_alone() -> TestResult {
    let suite = Suite::new("vale-spelling-none");
    let root = project(&suite, "none")?;
    root.file(
        ".ctl/none.yaml",
        "files:\n  \"text/*.md\":\n    styles: [Vale]\n    Vale.Repetition: off\n",
    );
    let words = misspelt(&root, ".ctl/none.yaml", "text/prose.md")?;
    for reported in ["blorft", "vSphere", "foxs", "hakkiri", "xyz"] {
        assert!(
            words.contains(reported) || reported == "GitHub",
            "{reported} was not reported: {words:?}"
        );
    }
    assert!(!words.contains("quick"), "{words:?}");
    Ok(())
}

#[test]
fn a_dictionary_no_package_holds_is_an_error_on_the_line_of_the_parameter() -> TestResult {
    let suite = Suite::new("vale-spelling-fault");
    let root = project(&suite, "fault")?;
    root.file(
        ".ctl/fault.yaml",
        "packages:\n  - path: dicts\nfiles:\n  \"text/*.md\":\n    styles: [Vale]\n    params:\n      Vale.Spelling:\n        dictionaries: nope\n",
    );
    let result = run(
        root.path(),
        &["check", "--config", ".ctl/fault.yaml", "text/prose.md"],
        &[],
    )?;
    assert_eq!(result.code, 2, "{}{}", result.out, result.err);
    let shown = format!("{}{}", result.out, result.err);
    assert!(
        shown.contains(".ctl/fault.yaml: line 8: dictionaries: rule `Vale.Spelling`: dictionary `nope`: `nope.aff` is not in `config/dictionaries`"),
        "{shown}"
    );
    Ok(())
}
