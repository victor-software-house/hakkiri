//! Vocabularies: the scenarios of the vocabularies change.
//!
//! `tests/fixtures/vocabularies` holds one package of rules and vocabularies
//! (`styles`), eight Markdown files (`text`, with `.fixture` after `.md`), and
//! five configs for each tool: `vale/<name>.ini` and `.ctl/<name>.yaml`.
//! `vale.tsv` holds one row for each alert Vale gave on the files under each
//! config: the config, the file, the line, the first and last column, the
//! level, the rule, the message and the matched text. The rows come from Vale
//! at `66bbb85` run from the fixture folder as `vale --config=vale/<name>.ini
//! --output=JSON text`.

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
const CONFIGS: [&str; 5] = ["none", "top", "glob", "union", "two"];

/// A file, a line, the first and last column, the level, the rule, the
/// message and the matched text.
type Row = (String, usize, usize, usize, String, String, String, String);

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vocabularies")
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
            rows.insert((
                file.to_owned(),
                line.parse()?,
                first.parse()?,
                last.parse()?,
                level.to_owned(),
                rule.to_owned(),
                message.to_owned(),
                matched.replace("\\n", "\n"),
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
    let suite = Suite::new("vocabularies-parity");
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

/// A project with the fixture's styles, `text/a.md` holding `text`, and a
/// config that holds `top` before `files`.
fn small<'a>(
    suite: &'a Suite,
    name: &str,
    top: &str,
    text: &str,
) -> Result<Root<'a>, Box<dyn Error>> {
    let root = project(suite, name)?;
    root.file(
        ".ctl/hakkiri.yaml",
        format!("packages:\n  - path: styles\n{top}files:\n  'text/*.md': {{styles: [Team]}}\n"),
    );
    root.file("text/a.md", text);
    Ok(root)
}

#[test]
fn an_unknown_vocabulary_is_an_error_on_the_key_line() -> TestResult {
    let suite = Suite::new("vocabularies-unknown");
    let root = small(&suite, "unknown", "vocab: [Nope]\n", "Fine.\n")?;
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 2, "{}{}", result.out, result.err);
    assert!(
        result
            .err
            .contains("line 3: vocab[0]: `Nope` is not a vocabulary in any package"),
        "{}",
        result.err
    );
    Ok(())
}

#[test]
fn two_packages_that_hold_one_vocabulary_are_refused() -> TestResult {
    let suite = Suite::new("vocabularies-twice");
    let root = suite.scenario("twice");
    for package in ["one", "two"] {
        root.file(
            format!("{package}/config/vocabularies/Team/accept.txt"),
            "GitHub\n",
        );
    }
    root.file(
        ".ctl/hakkiri.yaml",
        "packages:\n  - path: one\n  - path: two\nvocab: [Team]\n",
    );
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 2, "{}{}", result.out, result.err);
    assert!(
        result
            .err
            .contains("the vocabulary `Team` is in both `one/config/vocabularies/Team` and `two/config/vocabularies/Team`"),
        "{}",
        result.err
    );
    Ok(())
}

#[test]
fn a_term_that_is_not_a_pattern_names_its_file_and_line() -> TestResult {
    let suite = Suite::new("vocabularies-pattern");
    let root = suite.scenario("pattern");
    root.file(
        "styles/config/vocabularies/Team/accept.txt",
        "GitHub\n\nC(\n",
    );
    root.file(
        ".ctl/hakkiri.yaml",
        "packages:\n  - path: styles\nvocab: [Team]\n",
    );
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 2, "{}{}", result.out, result.err);
    assert!(
        result.err.contains("accept.txt")
            && result.err.contains("line 3")
            && result.err.contains("`C(`"),
        "{}",
        result.err
    );
    Ok(())
}

#[test]
fn the_lists_of_a_vocabulary_are_read_as_vale_reads_them() -> TestResult {
    let suite = Suite::new("vocabularies-lists");
    let root = suite.scenario("lists");
    let folder = "styles/config/vocabularies/Team";
    root.file(
        format!("{folder}/accept.txt"),
        "# a comment\n#nocomment\n  padded  \r\n\nAlpha\r\n",
    );
    root.file(format!("{folder}/sub/accept.txt"), "Beta\n");
    root.file(format!("{folder}/words.txt"), "Gamma\n");
    root.file(
        "styles/Team/Words.yml",
        "extends: existence\nmessage: \"Remove '%s'.\"\nlevel: warning\ntokens: [Alpha, Beta, Gamma, padded]\n",
    );
    root.file(
        ".ctl/hakkiri.yaml",
        "packages:\n  - path: styles\nvocab: [Team]\nfiles:\n  '*.md': {styles: [Team]}\n",
    );
    root.file("a.md", "Alpha Beta Gamma padded #nocomment.\n");
    let result = run(root.path(), &["check", "--format", "json"], &[])?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    let matched: Vec<String> = rows_of(&result.out)?.into_iter().map(|row| row.7).collect();
    assert_eq!(matched, ["Gamma"], "{}", result.out);
    Ok(())
}

#[test]
fn a_vocab_that_is_not_a_list_is_an_error() -> TestResult {
    let suite = Suite::new("vocabularies-word");
    let top = small(&suite, "top", "vocab: Team\n", "Fine.\n")?;
    let result = run(top.path(), &["check"], &[])?;
    assert_eq!(result.code, 2, "{}{}", result.out, result.err);
    assert!(
        result.err.contains("line 3: expected sequence start"),
        "{}",
        result.err
    );
    let glob = project(&suite, "glob")?;
    glob.file(
        ".ctl/hakkiri.yaml",
        "packages:\n  - path: styles\nfiles:\n  'text/*.md':\n    styles: [Team]\n    vocab: Team\n",
    );
    let result = run(glob.path(), &["check"], &[])?;
    assert_eq!(result.code, 2, "{}{}", result.out, result.err);
    assert!(
        result
            .err
            .contains("line 6: vocab: is a list of vocabulary names"),
        "{}",
        result.err
    );
    Ok(())
}

#[test]
fn a_config_with_no_vocab_loads_as_before() -> TestResult {
    let suite = Suite::new("vocabularies-none");
    let root = small(&suite, "none", "", "We use GitHub and harness.\n")?;
    let result = run(root.path(), &["check", "text/a.md"], &[])?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    let lines = result.lines();
    for line in [
        "text/a.md:1:8: warning Team.Filler: Remove 'GitHub'.",
        "text/a.md:1:19: warning Team.Filler: Remove 'harness'.",
    ] {
        assert!(lines.contains(&line), "{line} is missing from {lines:#?}");
    }
    Ok(())
}

#[test]
fn a_bad_top_level_vocab_is_placed_on_its_own_line() -> TestResult {
    let suite = Suite::new("vocabularies-line");
    let root = project(&suite, "line")?;
    root.file(
        ".ctl/hakkiri.yaml",
        "packages:\n  - path: styles\nfiles:\n  'text/*.md':\n    styles: [Team]\n    vocab: [Team]\nvocab: [Team, Nope]\n",
    );
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 2, "{}{}", result.out, result.err);
    assert!(
        result
            .err
            .contains("line 7: vocab[1]: `Nope` is not a vocabulary in any package"),
        "{}",
        result.err
    );
    Ok(())
}

/// The words a `spelling` rule reports in `text`, with `terms` accepted.
fn unaccepted(
    suite: &Suite,
    name: &str,
    terms: &str,
    text: &str,
) -> Result<BTreeSet<String>, Box<dyn Error>> {
    let root = suite.scenario(name);
    root.file("styles/config/vocabularies/Team/accept.txt", terms);
    root.file(
        "styles/Team/Spell.yml",
        "extends: spelling\nmessage: \"Did you really mean '%s'?\"\nlevel: error\n",
    );
    root.file(
        ".ctl/hakkiri.yaml",
        "packages:\n  - path: styles\nvocab: [Team]\nfiles:\n  '*.md': {styles: [Team]}\n",
    );
    root.file("a.md", text);
    let result = run(root.path(), &["check", "--format", "json"], &[])?;
    assert_eq!(result.code, 1, "{}{}", result.out, result.err);
    Ok(rows_of(&result.out)?.into_iter().map(|row| row.7).collect())
}

#[test]
fn each_accepted_term_matches_only_itself_in_spelling() -> TestResult {
    let suite = Suite::new("vocabularies-grouped");
    let text = "blorft Blorft zibble zibbled azobble zobble snorp Snorp.\n";
    let flagged = BTreeSet::from([
        "Blorft".to_owned(),
        "zibbled".to_owned(),
        "azobble".to_owned(),
    ]);
    for (name, terms) in [
        ("flag-first", "(?-i)blorft\nzibble|zobble\nsnorp\n"),
        ("flag-last", "snorp\nzibble|zobble\n(?-i)blorft\n"),
    ] {
        assert_eq!(unaccepted(&suite, name, terms, text)?, flagged, "{name}");
    }
    Ok(())
}

#[test]
fn an_unknown_vocabulary_of_a_glob_names_the_line_of_that_name() -> TestResult {
    let suite = Suite::new("vocabularies-later-name");
    let root = project(&suite, "later-name")?;
    root.file(
        ".ctl/hakkiri.yaml",
        "packages:\n  - path: styles\nfiles:\n  'text/*.md':\n    styles: [Team]\n    vocab:\n      - Team\n      - Nope\n",
    );
    root.file("text/a.md", "Fine.\n");
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 2, "{}{}", result.out, result.err);
    assert!(
        result
            .err
            .contains("line 8: vocab: `Nope` is not a vocabulary in any package"),
        "{}",
        result.err
    );
    Ok(())
}
