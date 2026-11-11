//! The rules a vocabulary builds: the scenarios of the vocabulary-rules change.
//!
//! `tests/fixtures/vocabulary-rules` holds one package of vocabularies
//! (`styles`), seven Markdown files (`text`, with `.fixture` after `.md`), and
//! nine configs for each tool: `vale/<name>.ini` and `.ctl/<name>.yaml`.
//! `vale.tsv` holds one row for each alert Vale gave on the files under each
//! config: the config, the file, the line, the first and last column, the
//! level, the rule, the message and the matched text. The rows come from Vale
//! at `66bbb85` run from the fixture folder as `vale --config=vale/<name>.ini
//! --output=JSON text`, with `Vale.Spelling` and `Vale.Repetition` turned off,
//! which hakkiri has no rule for yet.
//!
//! A match that spans lines ends on its first line in hakkiri, where Vale
//! counts every character of it from the first column. The last column of such
//! a row is left out of the comparison.

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
const CONFIGS: [&str; 9] = [
    "rules", "teamonly", "levels", "off", "both", "glob", "globvale", "globoff", "twoglobs",
];

/// A file, a line, the first and last column, the level, the rule, the
/// message and the matched text.
type Row = (String, usize, usize, usize, String, String, String, String);

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vocabulary-rules")
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

/// A scenario folder that holds the fixture's styles and text.
fn project<'a>(suite: &'a Suite, name: &str) -> Result<Root<'a>, Box<dyn Error>> {
    let root = suite.scenario(name);
    copy_into(&root, &fixture().join("styles"), Path::new("styles"))?;
    copy_into(&root, &fixture().join("text"), Path::new("text"))?;
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
            let matched = matched.replace("\\n", "\n");
            let last = if matched.contains('\n') {
                0
            } else {
                last.parse()?
            };
            rows.insert((
                file.to_owned(),
                line.parse()?,
                first.parse()?,
                last,
                level.to_owned(),
                rule.to_owned(),
                message.to_owned(),
                matched,
            ));
        }
    }
    Ok(rows)
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
        let matched = text("match");
        let last = if matched.contains('\n') {
            0
        } else {
            number("end_column")
        };
        rows.insert((
            text("path"),
            number("line"),
            number("column"),
            last,
            text("level"),
            text("rule"),
            text("message"),
            matched,
        ));
    }
    Ok(rows)
}

#[test]
fn every_config_gives_the_alerts_vale_gives() -> TestResult {
    let suite = Suite::new("vocabulary-rules-parity");
    let root = project(&suite, "parity")?;
    copy_into(&root, &fixture().join(".ctl"), Path::new(".ctl"))?;
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

/// A project with one package that holds `files`, a config that holds `config`,
/// and `a.md` holding `text`.
fn small<'a>(
    suite: &'a Suite,
    name: &str,
    files: &[(&str, &str)],
    config: &str,
    text: &str,
) -> Root<'a> {
    let root = suite.scenario(name);
    for (path, body) in files {
        root.file(format!("styles/{path}"), body);
    }
    root.file(".ctl/hakkiri.yaml", config);
    root.file("a.md", text);
    root
}

const ACCEPT_ONLY: &[(&str, &str)] = &[("config/vocabularies/Team/accept.txt", "GitHub\n")];

#[test]
fn the_style_vale_is_refused_when_no_vocabulary_holds_a_term() -> TestResult {
    let suite = Suite::new("vocabulary-rules-none");
    let root = small(
        &suite,
        "none",
        &[("config/vocabularies/Team/accept.txt", "# nothing\n")],
        "packages:\n  - path: styles\nvocab: [Team]\nfiles:\n  '*.md': {styles: [Vale]}\n",
        "Fine.\n",
    );
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 2, "{}{}", result.out, result.err);
    assert!(result.lines().is_empty(), "{}", result.out);
    assert!(
        result.err.contains("`Vale` is not a style in any package"),
        "{}",
        result.err
    );
    Ok(())
}

#[test]
fn a_level_for_a_rule_no_term_builds_is_an_error() -> TestResult {
    let suite = Suite::new("vocabulary-rules-level");
    let root = small(
        &suite,
        "level",
        ACCEPT_ONLY,
        "packages:\n  - path: styles\nvocab: [Team]\nfiles:\n  '*.md': {Vale.Avoid: warning}\n",
        "Fine.\n",
    );
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 2, "{}{}", result.out, result.err);
    assert!(
        result
            .err
            .contains("Vale.Avoid: names no rule in any style the config loads"),
        "{}",
        result.err
    );
    Ok(())
}

#[test]
fn a_package_style_named_vale_is_refused_beside_the_built_in_one() -> TestResult {
    let suite = Suite::new("vocabulary-rules-collision");
    let root = small(
        &suite,
        "collision",
        &[
            ("config/vocabularies/Team/accept.txt", "GitHub\n"),
            (
                "Vale/Own.yml",
                "extends: existence\nmessage: \"x '%s'\"\ntokens: [x]\n",
            ),
        ],
        "packages:\n  - path: styles\nvocab: [Team]\nfiles:\n  '*.md': {styles: [Vale]}\n",
        "Fine.\n",
    );
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 2, "{}{}", result.out, result.err);
    assert!(
        result
            .err
            .contains("the style `Vale` is built in from a vocabulary"),
        "{}",
        result.err
    );
    Ok(())
}
