//! Comment controls: the scenarios of the comment-controls change.
//!
//! `tests/fixtures/comment-controls` holds one package of four rules
//! (`styles`), nine Markdown files and one MDX file (`text`, with `.fixture`
//! after the extension), and one config for each tool: `vale/all.ini` and
//! `.ctl/all.yaml`.
//! Each paragraph of a file holds `very`, `really` and `good`, which three
//! rules flag, so the alerts that a comment leaves show which of them it
//! turned off.
//! `vale.tsv` holds one row for each alert Vale gave on the files: the config,
//! the file, the line, the first and last column, the level, the rule, the
//! message and the matched text.
//! The rows come from Vale at `66bbb85`, run from the fixture folder as
//! `vale --config=vale/all.ini --output=JSON text`.

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
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/comment-controls")
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

/// The rows Vale gave.
fn vale_rows() -> Result<BTreeSet<Row>, Box<dyn Error>> {
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
        if config == "all" {
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

/// The fixture in a scenario folder, with the Vale comments written as hakkiri
/// comments when `hakkiri` is set.
fn project<'a>(suite: &'a Suite, name: &str, hakkiri: bool) -> Result<Root<'a>, Box<dyn Error>> {
    let root = suite.scenario(name);
    copy_into(&root, &fixture().join("styles"), Path::new("styles"))?;
    copy_into(&root, &fixture().join(".ctl"), Path::new(".ctl"))?;
    if hakkiri {
        for entry in fs::read_dir(fixture().join("text"))? {
            let entry = entry?;
            let file = entry.file_name().to_string_lossy().into_owned();
            // The matches of a rule and the list of styles have no hakkiri
            // form.
            if file.starts_with("c6.") {
                continue;
            }
            let written = fs::read_to_string(entry.path())?;
            root.file(
                Path::new("text").join(file.trim_end_matches(".fixture")),
                hakkiri_form(&written),
            );
        }
    } else {
        copy_into(&root, &fixture().join("text"), Path::new("text"))?;
    }
    Ok(root)
}

/// `text` with each comment that Vale reads written as hakkiri writes it, in
/// the HTML form of a Markdown file and the `{/* */}` form of an MDX file.
fn hakkiri_form(text: &str) -> String {
    let mut lines: Vec<String> = text
        .lines()
        .map(|line| {
            let mut line = line.to_owned();
            for (open, close) in [("<!-- ", " -->"), ("{/* ", " */}")] {
                for word in ["off", "on"] {
                    line = line.replace(
                        &format!("{open}vale {word}{close}"),
                        &format!("{open}hakkiri {word}{close}"),
                    );
                }
                for (word, ours) in [
                    (" = NO", ": off"),
                    (" = YES", ": on"),
                    (" = off", ": off"),
                    (" = on", ": on"),
                ] {
                    let start = format!("{open}vale ");
                    let end = format!("{word}{close}");
                    if line.starts_with(&start) && line.ends_with(&end) && !line.contains('[') {
                        let key = &line[start.len()..line.len() - end.len()];
                        line = format!("{open}hakkiri {key}{ours}{close}");
                    }
                }
            }
            line
        })
        .collect();
    lines.push(String::new());
    lines.join("\n")
}

#[test]
fn every_file_gives_the_alerts_vale_gives() -> TestResult {
    let suite = Suite::new("comment-controls-parity");
    let root = project(&suite, "parity", false)?;
    let result = run(
        root.path(),
        &[
            "check",
            "--config",
            ".ctl/all.yaml",
            "--format",
            "json",
            "text",
        ],
        &[],
    )?;
    assert!(
        result.code == 0 || result.code == 1,
        "{}{}",
        result.out,
        result.err
    );
    let ours = rows_of(&result.out)?;
    let vale = vale_rows()?;
    let only_vale: Vec<_> = vale.difference(&ours).collect();
    let only_ours: Vec<_> = ours.difference(&vale).collect();
    assert!(
        only_vale.is_empty() && only_ours.is_empty(),
        "only Vale gave {only_vale:#?}, only hakkiri gave {only_ours:#?}"
    );
    Ok(())
}

/// The line, the rule and the matched text of each alert, with a repeat kept.
fn spots(rows: &BTreeSet<Row>) -> Vec<(String, usize, String, String)> {
    rows.iter()
        .map(|(file, line, _, _, _, rule, _, matched)| {
            (file.clone(), *line, rule.clone(), matched.clone())
        })
        .collect()
}

#[test]
fn hakkiri_comments_reach_as_vale_comments_do() -> TestResult {
    let suite = Suite::new("comment-controls-hakkiri");
    let root = project(&suite, "hakkiri", true)?;
    let result = run(
        root.path(),
        &[
            "check",
            "--config",
            ".ctl/all.yaml",
            "--format",
            "json",
            "text",
        ],
        &[],
    )?;
    assert!(
        result.code == 0 || result.code == 1,
        "{}{}",
        result.out,
        result.err
    );
    let ours: BTreeSet<Row> = rows_of(&result.out)?;
    let vale: BTreeSet<Row> = vale_rows()?
        .into_iter()
        .filter(|row| !row.0.ends_with("c6.md"))
        .collect();
    let (ours, vale) = (spots(&ours), spots(&vale));
    let only_vale: Vec<_> = vale.iter().filter(|spot| !ours.contains(spot)).collect();
    let only_ours: Vec<_> = ours.iter().filter(|spot| !vale.contains(spot)).collect();
    assert!(
        only_vale.is_empty() && only_ours.is_empty(),
        "only the Vale comments gave {only_vale:#?}, only the hakkiri comments gave {only_ours:#?}"
    );
    Ok(())
}

/// A rule that flags `very`, and one that flags `good`.
const FILLER: &str = "extends: existence\nmessage: \"Remove '%s'.\"\nlevel: warning\nscope: text\nignorecase: true\ntokens:\n  - very\n";

/// A project with the style `Team` that holds the rule `Filler`, and `a.md`.
fn small<'a>(suite: &'a Suite, name: &str, text: &str) -> Root<'a> {
    let root = suite.scenario(name);
    root.file("styles/Team/Filler.yml", FILLER);
    root.file(
        ".ctl/hakkiri.yaml",
        "packages:\n  - path: styles\nfiles:\n  '*.md':\n    styles: [Team]\n  '*.rs':\n    styles: [Team]\n",
    );
    root.file("a.md", text);
    root
}

/// The alert lines of a run, without its summary line.
fn alerts(result: &common::Run) -> Vec<&str> {
    result
        .lines()
        .into_iter()
        .filter(|line| line.contains(" Team.Filler: "))
        .collect()
}

#[test]
fn a_level_comment_sets_the_level_from_there_until_the_rule_is_named_again() -> TestResult {
    let suite = Suite::new("comment-controls-level");
    let root = small(
        &suite,
        "level",
        "A very one.\n\n<!-- hakkiri Team.Filler: error -->\n\nA very two.\n\n<!-- hakkiri Team.Filler: suggestion -->\n\nA very three.\n\n<!-- hakkiri Team.Filler: on -->\n\nA very four.\n",
    );
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 1, "{}{}", result.out, result.err);
    assert_eq!(
        alerts(&result),
        [
            "a.md:1:3: warning Team.Filler: Remove 'very'.",
            "a.md:5:3: error Team.Filler: Remove 'very'.",
            "a.md:9:3: suggestion Team.Filler: Remove 'very'.",
            "a.md:13:3: warning Team.Filler: Remove 'very'.",
        ],
        "{}",
        result.out
    );
    Ok(())
}

#[test]
fn a_level_comment_wins_over_the_level_the_config_sets() -> TestResult {
    let suite = Suite::new("comment-controls-config-level");
    let root = small(
        &suite,
        "config-level",
        "<!-- hakkiri Team.Filler: error -->\n\nA very one.\n",
    );
    root.file(
        ".ctl/hakkiri.yaml",
        "packages:\n  - path: styles\nfiles:\n  '*.md':\n    styles: [Team]\n    Team.Filler: suggestion\n",
    );
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(
        alerts(&result),
        ["a.md:3:3: error Team.Filler: Remove 'very'."],
        "{}",
        result.out
    );
    Ok(())
}

#[test]
fn a_comment_in_a_source_file_turns_nothing_off() -> TestResult {
    let suite = Suite::new("comment-controls-source");
    let root = small(&suite, "source", "Fine.\n");
    root.file(
        "a.rs",
        "//! A very one.\n//!\n//! <!-- hakkiri off -->\n//!\n//! A very two.\n\nfn main() {}\n",
    );
    let result = run(root.path(), &["check", "a.rs"], &[])?;
    let lines: Vec<&str> = alerts(&result);
    assert_eq!(lines.len(), 2, "{}", result.out);
    Ok(())
}
