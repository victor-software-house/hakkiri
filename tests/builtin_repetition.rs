//! The built-in rule `Vale.Repetition`: the scenarios of the vale-repetition
//! change.
//!
//! `tests/fixtures/builtin-repetition` holds one package (`styles`, with a
//! vocabulary `Rep`), three Markdown files and one Rust file (`text`, with
//! `.fixture` after the extension), and five configs for each tool:
//! `vale/<name>.ini` and `.ctl/<name>.yaml`. `vale.tsv` holds one row for each
//! alert Vale gave on the files under each config. A row has the config, the
//! file, the line, the first and last column, the level, the rule, the message
//! and the matched text, with a line feed written `\n` and a tab `\t`.
//! The rows come from Vale at `66bbb85` run from the fixture folder as `vale
//! --config=vale/<name>.ini --output=JSON text`, with `Vale.Spelling` turned
//! off, as the hakkiri configs turn it off.
//!
//! A repeat that spans two lines ends where its last character is in hakkiri,
//! as `end_line` and `end_column`. Vale prints the first column plus the
//! number of characters from the start to the end in the file, line breaks and
//! indentation included, which passes the end of the line. The comparison maps
//! hakkiri's end to that count for such a match.

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

/// The configs of the fixture, by name.
const CONFIGS: [&str; 5] = ["plain", "top", "glob", "warning", "off"];

/// A file, a line, the first and last column, the level, the rule, the
/// message and the matched text.
type Row = (String, usize, usize, usize, String, String, String, String);

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/builtin-repetition")
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

/// A scenario folder that holds the fixture's styles, text and configs.
fn project<'a>(suite: &'a Suite, name: &str) -> Result<Root<'a>, Box<dyn Error>> {
    let root = suite.scenario(name);
    copy_into(&root, &fixture().join("styles"), Path::new("styles"))?;
    copy_into(&root, &fixture().join("text"), Path::new("text"))?;
    copy_into(&root, &fixture().join(".ctl"), Path::new(".ctl"))?;
    Ok(root)
}

/// The rows Vale gave under the config `name`.
fn vale_rows(name: &str) -> Result<BTreeSet<Row>, Box<dyn Error>> {
    let mut rows = BTreeSet::new();
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
        if config == name {
            rows.insert((
                file.to_owned(),
                line.parse()?,
                first.parse()?,
                last.parse()?,
                level.to_owned(),
                rule.to_owned(),
                message.to_owned(),
                matched.replace("\\n", "\n").replace("\\t", "\t"),
            ));
        }
    }
    Ok(rows)
}

/// The last column Vale prints for a match that starts at `line` and `first`
/// and ends at `end_line` and `end_column` of `source`: the first column plus
/// the characters from the start to the end of the match, minus one.
fn vale_last(source: &str, line: usize, first: usize, end: (usize, usize)) -> usize {
    let lines: Vec<&str> = source.lines().collect();
    let length = |number: usize| lines.get(number - 1).map_or(0, |text| text.chars().count());
    let (end_line, end_column) = end;
    let mut count = length(line) + 1 - first;
    for between in line + 1..end_line {
        count += 1 + length(between);
    }
    count += 1 + end_column;
    first + count - 1
}

/// The rows of a `check --format json` run.
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
        let (path, line, first) = (text("path"), number("line"), number("column"));
        let last = if number("end_line") == line {
            number("end_column")
        } else {
            let file = fixture().join(format!("{path}.fixture"));
            let source = fs::read_to_string(file)?;
            vale_last(
                &source,
                line,
                first,
                (number("end_line"), number("end_column")),
            )
        };
        rows.insert((
            path,
            line,
            first,
            last,
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
    let suite = Suite::new("builtin-repetition-parity");
    let root = project(&suite, "parity")?;
    for name in CONFIGS {
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
        let vale = vale_rows(name)?;
        let only_vale: Vec<_> = vale.difference(&ours).collect();
        let only_ours: Vec<_> = ours.difference(&vale).collect();
        assert!(
            only_vale.is_empty() && only_ours.is_empty(),
            "{name}: only Vale gave {only_vale:#?}, only hakkiri gave {only_ours:#?}"
        );
    }
    Ok(())
}

#[test]
fn the_style_vale_runs_the_rule_with_no_vocabulary() -> TestResult {
    let suite = Suite::new("builtin-repetition-bare");
    let root = suite.scenario("bare");
    root.file(".ctl/hakkiri.yaml", "files:\n  '*.md': {styles: [Vale]}\n");
    root.file("a.md", "This is is a test.\n");
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 1, "{}{}", result.out, result.err);
    assert_eq!(
        result.lines(),
        [
            "a.md:1:6: error Vale.Repetition: 'is' is repeated!",
            "1 alert in 1 file",
        ]
    );
    Ok(())
}
