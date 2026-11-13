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
//! as the hakkiri configs turn them off.
//!
//! A match that spans lines ends where its last character is in hakkiri, as
//! `end_line` and `end_column`. Vale prints the first column plus the number of
//! characters of the match, line breaks included, which can pass the end of the
//! line. The comparison maps hakkiri's end to Vale's count for such a match,
//! and compares the single-line ends as they are.

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
            rows.insert((
                file.to_owned(),
                line.parse()?,
                first.parse()?,
                last.parse()?,
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
        let (line, first) = (number("line"), number("column"));
        let last = if number("end_line") == line {
            number("end_column")
        } else {
            (first + matched.chars().count()).saturating_sub(1)
        };
        rows.insert((
            text("path"),
            line,
            first,
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
        let mut ours = rows_of(&result.out)?;
        let mut vale = vale_rows(name)?;
        leave_out_the_escape_difference(&mut vale, &mut ours);
        let only_vale: Vec<_> = vale.difference(&ours).collect();
        let only_ours: Vec<_> = ours.difference(&vale).collect();
        assert!(
            only_vale.is_empty() && only_ours.is_empty(),
            "{name}: only Vale gave {only_vale:#?}, only hakkiri gave {only_ours:#?}"
        );
    }
    Ok(())
}

/// The message for `DATA SET` under the accepted term `data\\Wset`.
const ESCAPE_MESSAGE: &str = "Use 'data\\Wset' instead of 'DATA SET'.";

/// Takes out the rows that differ because of an escape in a term.
///
/// Vale lowercases the whole pattern of `data\\Wset`, so `\\W` turns into
/// `\\w`: it flags `dataxset` and leaves `DATA SET` alone.
/// hakkiri lowercases only the literal text, so `\\W` stays a non-word
/// character: it flags `DATA SET`, which is not written as the term is, and
/// leaves `dataxset` alone.
/// Each config that gives one of the two must give the other.
fn leave_out_the_escape_difference(vale: &mut BTreeSet<Row>, ours: &mut BTreeSet<Row>) {
    let before = (vale.len(), ours.len());
    vale.retain(|row| row.7 != "dataxset");
    ours.retain(|row| row.6 != ESCAPE_MESSAGE);
    let removed = (before.0 - vale.len(), before.1 - ours.len());
    assert_eq!(
        removed.0, removed.1,
        "Vale and hakkiri differ by one row each"
    );
}

/// `text/t5.md` wraps `mea Culpa` and `Bad Word` across a line break, and each
/// alert names the line and column of the match's last character.
#[test]
fn a_match_across_lines_ends_at_its_last_character() -> TestResult {
    let suite = Suite::new("vocabulary-rules-end");
    let root = project(&suite, "end")?;
    copy_into(&root, &fixture().join(".ctl"), Path::new(".ctl"))?;
    let result = run(
        root.path(),
        &[
            "check",
            "--config",
            ".ctl/rules.yaml",
            "--format",
            "json",
            "text/t5.md",
        ],
        &[],
    )?;
    let json: Value = serde_json::from_str(&result.out)?;
    let ends: Vec<[u64; 4]> = json["alerts"]
        .as_array()
        .ok_or("no alerts array")?
        .iter()
        .filter(|alert| {
            alert["match"]
                .as_str()
                .is_some_and(|text| text.contains('\n'))
        })
        .map(|alert| {
            ["line", "column", "end_line", "end_column"].map(|key| alert[key].as_u64().unwrap_or(0))
        })
        .collect();
    assert_eq!(ends, [[1, 5, 2, 5], [2, 17, 3, 4]], "{}", result.out);
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
fn the_style_vale_holds_the_repetition_rule_when_no_vocabulary_holds_a_term() -> TestResult {
    let suite = Suite::new("vocabulary-rules-none");
    let root = small(
        &suite,
        "none",
        &[("config/vocabularies/Team/accept.txt", "# nothing\n")],
        "packages:\n  - path: styles\nvocab: [Team]\nfiles:\n  '*.md': {styles: [Vale]}\n",
        "Fine, fine and the the.\n",
    );
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 1, "{}{}", result.out, result.err);
    assert_eq!(
        result.lines(),
        [
            "a.md:1:16: error Vale.Repetition: 'the' is repeated!",
            "1 alert in 1 file",
        ]
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
fn a_package_style_named_vale_is_skipped_with_a_warning() -> TestResult {
    let suite = Suite::new("vocabulary-rules-collision");
    let root = small(
        &suite,
        "collision",
        &[
            (
                "Acme/Own.yml",
                "extends: existence\nmessage: \"x '%s'\"\nlevel: error\ntokens: [x]\n",
            ),
            (
                "Vale/Own.yml",
                "extends: existence\nmessage: \"y '%s'\"\nlevel: error\ntokens: [y]\n",
            ),
        ],
        "packages:\n  - path: styles\nfiles:\n  '*.md': {styles: [Acme, Vale]}\n",
        "Here x and y.\n",
    );
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 1, "{}{}", result.out, result.err);
    assert!(result.out.contains("Acme.Own"), "{}", result.out);
    assert!(!result.out.contains("Vale.Own"), "{}", result.out);
    let warnings: Vec<&str> = result
        .err
        .lines()
        .filter(|line| line.starts_with("warning "))
        .collect();
    assert_eq!(warnings.len(), 1, "{}", result.err);
    assert!(
        warnings[0].contains("`styles/Vale` is skipped: the built-in style `Vale` wins"),
        "{}",
        result.err
    );
    Ok(())
}
