//! Rule parameters: the scenarios of the rule-parameters change.
//!
//! `tests/fixtures/rule-parameters` holds one package (`styles`, with the
//! style `Team`: one rule of each check type that has parameters), two
//! Markdown files (`text`, with `.fixture` after the extension), and one config
//! for each tool and scenario: `vale/<name>.ini` and `.ctl/<name>.yaml`.
//! `vale.tsv` holds one row for each alert Vale gave on the files under each
//! config. A row has the config, the file, the line, the first and last column,
//! the level, the rule, the message and the matched text, with a line feed
//! written `\n` and a tab `\t`. The rows come from Vale at `66bbb85` run from
//! the fixture folder as `vale --config=vale/<name>.ini --output=JSON text`,
//! with `Vale.Spelling` turned off, which hakkiri has no rule for yet.
//!
//! Vale applies a parameter to every file, whichever section sets it, and
//! hakkiri applies it to the files of the glob that sets it. Each config of the
//! fixture has one glob that holds every file, where the two agree. The other
//! tests hold what hakkiri does where they differ.

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

/// A file, a line, the first and last column, the level, the rule, the
/// message and the matched text.
type Row = (String, usize, usize, usize, String, String, String, String);

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/rule-parameters")
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

/// The names of the configs of the fixture, in order.
fn configs() -> Result<Vec<String>, Box<dyn Error>> {
    let mut names = Vec::new();
    for entry in fs::read_dir(fixture().join(".ctl"))? {
        let name = entry?.file_name().to_string_lossy().into_owned();
        let name = name.trim_end_matches(".yaml");
        if !name.starts_with("fault-") {
            names.push(name.to_owned());
        }
    }
    names.sort();
    Ok(names)
}

/// The rows Vale gave, by config.
fn vale_rows() -> Result<Vec<(String, Row)>, Box<dyn Error>> {
    let mut rows = Vec::new();
    for line in fs::read_to_string(fixture().join("vale.tsv"))?.lines() {
        let cells: Vec<&str> = line.split('\t').collect();
        let [
            config,
            file,
            line,
            first,
            last,
            level,
            rule,
            message,
            matched,
        ] = cells[..]
        else {
            return Err(format!("a row of vale.tsv has {} cells", cells.len()).into());
        };
        rows.push((
            config.to_owned(),
            (
                file.to_owned(),
                line.parse()?,
                first.parse()?,
                last.parse()?,
                level.to_owned(),
                rule.to_owned(),
                message.to_owned(),
                matched.replace("\\n", "\n").replace("\\t", "\t"),
            ),
        ));
    }
    Ok(rows)
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

/// A scenario folder that holds the fixture's styles, text and configs.
fn project<'a>(suite: &'a Suite, name: &str) -> Result<Root<'a>, Box<dyn Error>> {
    let root = suite.scenario(name);
    copy_into(&root, &fixture().join("styles"), Path::new("styles"))?;
    copy_into(&root, &fixture().join("text"), Path::new("text"))?;
    copy_into(&root, &fixture().join(".ctl"), Path::new(".ctl"))?;
    Ok(root)
}

#[test]
fn every_config_gives_the_alerts_vale_gives() -> TestResult {
    let suite = Suite::new("rule-parameters-parity");
    let root = project(&suite, "parity")?;
    let vale = vale_rows()?;
    let names = configs()?;
    let recorded: BTreeSet<&str> = vale.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(
        recorded,
        names.iter().map(String::as_str).collect::<BTreeSet<_>>(),
        "vale.tsv and .ctl hold different configs"
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
        let theirs: BTreeSet<Row> = vale
            .iter()
            .filter(|(config, _)| config == name)
            .map(|(_, row)| row.clone())
            .collect();
        let only_vale: Vec<_> = theirs.difference(&ours).collect();
        let only_ours: Vec<_> = ours.difference(&theirs).collect();
        if !(only_vale.is_empty() && only_ours.is_empty()) {
            failed.push(format!(
                "{name}: only Vale gave {only_vale:#?}, only hakkiri gave {only_ours:#?}"
            ));
        }
    }
    assert!(failed.is_empty(), "{}", failed.join("\n"));
    Ok(())
}

/// The faults of the fixture: a config, the line of its fault, the message
/// hakkiri gives there, and the message Vale gives. The unknown-field faults
/// list the fields of the check, and their row stops before the list.
fn faults() -> Result<Vec<[String; 4]>, Box<dyn Error>> {
    let mut rows = Vec::new();
    for line in fs::read_to_string(fixture().join("faults.tsv"))?.lines() {
        let cells: Vec<&str> = line.split('\t').collect();
        let [name, at, ours, vale] = cells[..] else {
            return Err(format!("a row of faults.tsv has {} cells", cells.len()).into());
        };
        rows.push([name, at, ours, vale].map(str::to_owned));
    }
    Ok(rows)
}

#[test]
fn every_fault_is_refused_on_the_line_of_the_config_that_holds_it() -> TestResult {
    let suite = Suite::new("rule-parameters-faults");
    let root = project(&suite, "faults")?;
    let rows = faults()?;
    assert_eq!(rows.len(), 16, "one row for each fault of the fixture");
    for [name, line, ours, vale] in rows {
        assert!(!vale.is_empty(), "{name}: the row holds what Vale says");
        let config = format!(".ctl/fault-{name}.yaml");
        let result = run(root.path(), &["check", "--config", &config, "text"], &[])?;
        assert_eq!(result.code, 2, "{name}: {}{}", result.out, result.err);
        assert!(result.lines().is_empty(), "{name}: {}", result.out);
        let wanted = format!("failed {config}: line {line}: {ours}");
        assert!(result.err.contains(&wanted), "{name}: {}", result.err);
        assert_eq!(
            result.err.matches("failed ").count(),
            1,
            "{name}: {}",
            result.err
        );
    }
    Ok(())
}

/// A project with the text `the the the` in `a.md` and `b.md`, and the config
/// `config`.
fn repeated<'a>(suite: &'a Suite, name: &str, config: &str) -> Root<'a> {
    let root = suite.scenario(name);
    root.file(".ctl/hakkiri.yaml", config);
    root.file("a.md", "the the the\n");
    root.file("b.md", "the the the\n");
    root
}

/// The alert of a repeat of `the` at `column` of the first line of `file`.
fn repeat(file: &str, column: usize) -> String {
    format!("{file}:1:{column}: error Vale.Repetition: 'the' is repeated!")
}

#[test]
fn a_parameter_reaches_only_the_files_its_glob_matches() -> TestResult {
    let suite = Suite::new("rule-parameters-glob");
    let root = repeated(
        &suite,
        "glob",
        "files:\n  '*.md': {styles: [Vale]}\n  'a.md':\n    params:\n      Vale.Repetition: {max: 1}\n",
    );
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 1, "{}{}", result.out, result.err);
    assert_eq!(
        result.lines(),
        [
            repeat("a.md", 5),
            repeat("b.md", 1),
            repeat("b.md", 5),
            "3 alerts in 2 files".to_owned()
        ]
    );
    Ok(())
}

#[test]
fn a_later_glob_wins_for_the_same_parameter() -> TestResult {
    let suite = Suite::new("rule-parameters-later");
    let root = repeated(
        &suite,
        "later",
        "files:\n  '*.md':\n    styles: [Vale]\n    params:\n      Vale.Repetition: {max: 1}\n  'a.md':\n    params:\n      Vale.Repetition: {max: 2}\n",
    );
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 1, "{}{}", result.out, result.err);
    assert_eq!(
        result.lines(),
        [repeat("b.md", 5), "1 alert in 2 files".to_owned()]
    );
    Ok(())
}

#[test]
fn a_rule_written_twice_under_params_is_refused() -> TestResult {
    let suite = Suite::new("rule-parameters-twice");
    let root = repeated(
        &suite,
        "twice",
        "files:\n  '*.md':\n    styles: [Vale]\n    params:\n      Vale.Repetition: {max: 1}\n      Vale.Repetition: {max: 2}\n",
    );
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 2, "{}{}", result.out, result.err);
    assert!(result.lines().is_empty(), "{}", result.out);
    assert!(
        result
            .err
            .contains("failed .ctl/hakkiri.yaml: line 6: duplicate mapping key"),
        "{}",
        result.err
    );
    Ok(())
}

#[test]
fn an_empty_styles_list_drops_the_parameters_before_it() -> TestResult {
    let suite = Suite::new("rule-parameters-empty");
    let root = repeated(
        &suite,
        "empty",
        "files:\n  '*.md':\n    styles: [Vale]\n    params:\n      Vale.Repetition: {max: 2}\n  'a.md': {styles: []}\n  'a.*': {styles: [Vale]}\n",
    );
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 1, "{}{}", result.out, result.err);
    assert_eq!(
        result.lines(),
        [
            repeat("a.md", 1),
            repeat("a.md", 5),
            "2 alerts in 2 files".to_owned()
        ]
    );
    Ok(())
}

#[test]
fn the_project_overrides_the_parameter_of_a_package() -> TestResult {
    let suite = Suite::new("rule-parameters-package");
    let settings =
        "files:\n  '*.md':\n    styles: [Vale]\n    params:\n      Vale.Repetition: {max: 2}\n";
    let root = repeated(
        &suite,
        "package",
        "packages:\n  - path: pkg/styles\nfiles:\n  'b.md':\n    params:\n      Vale.Repetition: {max: 1}\n",
    );
    root.file("pkg/.ctl/hakkiri.yaml", settings);
    root.file(
        "pkg/styles/Team/Filler.yml",
        "extends: existence\nmessage: \"Remove '%s'.\"\nlevel: warning\ntokens: [very]\n",
    );
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 1, "{}{}", result.out, result.err);
    assert_eq!(
        result.lines(),
        [repeat("b.md", 5), "1 alert in 2 files".to_owned()]
    );
    Ok(())
}

#[test]
fn a_rule_built_from_a_vocabulary_takes_no_parameter() -> TestResult {
    let suite = Suite::new("rule-parameters-vocabulary");
    let root = suite.scenario("vocabulary");
    root.file("styles/config/vocabularies/Team/accept.txt", "Foo\n");
    root.file(
        ".ctl/hakkiri.yaml",
        "packages:\n  - path: styles\nvocab: [Team]\nfiles:\n  'a.md':\n    styles: [Vale]\n    params:\n      Vale.Terms: {ignorecase: false}\n",
    );
    root.file("a.md", "A foo.\n");
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 2, "{}{}", result.out, result.err);
    assert!(
        result.err.contains(
            "failed .ctl/hakkiri.yaml: line 8: ignorecase: rule `Vale.Terms`: is not a parameter: this rule is built from a vocabulary and takes none"
        ),
        "{}",
        result.err
    );
    Ok(())
}

#[test]
fn a_parameter_of_a_rule_no_glob_runs_is_still_checked() -> TestResult {
    let suite = Suite::new("rule-parameters-unused");
    let root = project(&suite, "unused")?;
    root.file(
        ".ctl/unused.yaml",
        "packages:\n  - path: styles\nfiles:\n  'text/*.md':\n    styles: [Vale]\n    params:\n      Team.Exist: {max: 1}\n",
    );
    let result = run(root.path(), &["check", "--config", ".ctl/unused.yaml"], &[])?;
    assert_eq!(result.code, 2, "{}{}", result.out, result.err);
    assert!(
        result
            .err
            .contains("failed .ctl/unused.yaml: line 7: rule `Team.Exist`: unknown field `max`"),
        "{}",
        result.err
    );
    Ok(())
}

#[test]
fn every_fault_of_a_config_is_reported_at_once() -> TestResult {
    let suite = Suite::new("rule-parameters-many");
    let root = repeated(
        &suite,
        "many",
        "files:\n  '*.md':\n    styles: [Vale]\n    params:\n      Vale.Repetition:\n        max: abc\n        bogus: 1\n        tokens: x\n      Nope.Rule: {max: 1}\n",
    );
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 2, "{}{}", result.out, result.err);
    for line in [6, 7, 8, 9] {
        assert!(
            result
                .err
                .contains(&format!("failed .ctl/hakkiri.yaml: line {line}: ")),
            "line {line}: {}",
            result.err
        );
    }
    assert_eq!(result.err.matches("failed ").count(), 4, "{}", result.err);
    Ok(())
}
