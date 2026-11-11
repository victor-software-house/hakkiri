//! Block and token ignores: the scenarios of the block-token-ignores change.
//!
//! `tests/fixtures/ignores` holds one package of six rules (`styles`), eight
//! text files (`text`, with `.fixture` after the extension), and ten configs
//! for each tool: `vale/<name>.ini` and `.ctl/<name>.yaml`.
//! `vale.tsv` holds one row for each alert Vale gave on the files under each
//! config: the config, the file, the line, the first and last column, the
//! level, the rule, the message and the matched text.
//! The rows come from Vale at `66bbb85`, with the Rust grammar built from the
//! same commit, run from the fixture folder as `vale --config=vale/<name>.ini
//! --output=JSON text`.
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
const CONFIGS: [&str; 10] = [
    "plain", "heads", "tokens", "blocks", "group", "both", "rust", "mdx", "global", "twoglobs",
];

/// A file, a line, the first and last column, the level, the rule, the
/// message and the matched text.
type Row = (String, usize, usize, usize, String, String, String, String);

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/ignores")
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
    let suite = Suite::new("ignores-parity");
    let root = suite.scenario("parity");
    copy_into(&root, &fixture().join("styles"), Path::new("styles"))?;
    copy_into(&root, &fixture().join("text"), Path::new("text"))?;
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

/// A rule that flags `very`.
const FILLER: &str = "extends: existence\nmessage: \"Remove '%s'.\"\nlevel: warning\nscope: text\nignorecase: true\ntokens:\n  - very\n";

/// A rule that flags `good`.
const PLAIN: &str = "extends: existence\nmessage: \"Drop '%s'.\"\nlevel: warning\nscope: text\nignorecase: true\ntokens:\n  - good\n";

/// The alert of `very` and of `good` in `a.md`, the text of the scenarios
/// below: `very` starts at column 9 and `good` at 14.
const BOTH: [&str; 2] = [
    "a.md:1:9: warning Team.Filler: Remove 'very'.",
    "a.md:1:14: warning Team.Plain: Drop 'good'.",
];

/// A project with one style `Team` that holds both rules, the config `config`,
/// and `a.md` holding `This is very good.`
fn small<'a>(suite: &'a Suite, name: &str, config: &str) -> Root<'a> {
    let root = suite.scenario(name);
    root.file("styles/Team/Filler.yml", FILLER);
    root.file("styles/Team/Plain.yml", PLAIN);
    root.file(".ctl/hakkiri.yaml", config);
    root.file("a.md", "This is very good.\n");
    root
}

/// The alert lines of `a.md` in a run, without the summary line.
fn alerts(result: &common::Run) -> Vec<&str> {
    result
        .lines()
        .into_iter()
        .filter(|line| line.starts_with("a.md:"))
        .collect()
}

#[test]
fn an_invalid_pattern_is_an_error_on_its_key() -> TestResult {
    let suite = Suite::new("ignores-invalid");
    let root = small(
        &suite,
        "invalid",
        "packages:\n  - path: styles\nfiles:\n  '*.md':\n    styles: [Team]\n    token_ignores:\n      - '('\n",
    );
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 2, "{}{}", result.out, result.err);
    assert!(result.lines().is_empty(), "{}", result.out);
    assert!(
        result.err.contains("token_ignores") && result.err.contains("does not compile"),
        "{}",
        result.err
    );
    Ok(())
}

#[test]
fn a_word_where_a_list_of_patterns_belongs_is_an_error() -> TestResult {
    let suite = Suite::new("ignores-word");
    let root = small(
        &suite,
        "word",
        "packages:\n  - path: styles\nfiles:\n  '*.md':\n    styles: [Team]\n    block_ignores: BEGIN\n",
    );
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 2, "{}{}", result.out, result.err);
    assert!(
        result.err.contains("block_ignores: is a list of patterns"),
        "{}",
        result.err
    );
    Ok(())
}

#[test]
fn a_pattern_that_matches_empty_text_changes_nothing() -> TestResult {
    let suite = Suite::new("ignores-empty");
    let root = small(
        &suite,
        "empty",
        "packages:\n  - path: styles\nfiles:\n  '*.md':\n    styles: [Team]\n    block_ignores: ['(z*)']\n    token_ignores: ['(z*)']\n",
    );
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    assert_eq!(alerts(&result), BOTH, "{}", result.out);
    Ok(())
}

#[test]
fn a_packages_ignores_and_the_projects_add_up() -> TestResult {
    let suite = Suite::new("ignores-package");
    let root = small(
        &suite,
        "package",
        "packages:\n  - path: pkg/styles\nfiles:\n  '*.md':\n    styles: [Team]\n    token_ignores: ['(good)']\n",
    );
    root.file("pkg/styles/Team/Filler.yml", FILLER);
    root.file("pkg/styles/Team/Plain.yml", PLAIN);
    root.file(
        "pkg/.ctl/hakkiri.yaml",
        "files:\n  '*.md':\n    token_ignores: ['(very)']\n",
    );
    let alone = run(root.path(), &["check"], &[])?;
    assert_eq!(alone.code, 0, "{}{}", alone.out, alone.err);
    assert!(alerts(&alone).is_empty(), "{}", alone.out);
    Ok(())
}

#[test]
fn the_projects_ignores_alone_leave_what_the_package_would_hide() -> TestResult {
    let suite = Suite::new("ignores-project");
    let root = small(
        &suite,
        "project",
        "packages:\n  - path: pkg/styles\nfiles:\n  '*.md':\n    styles: [Team]\n    token_ignores: ['(good)']\n",
    );
    root.file("pkg/styles/Team/Filler.yml", FILLER);
    root.file("pkg/styles/Team/Plain.yml", PLAIN);
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    assert_eq!(alerts(&result), [BOTH[0]], "{}", result.out);
    Ok(())
}

/// A rule that flags `very` in a code span.
const CODE: &str =
    "extends: existence\nmessage: \"Code '%s'.\"\nlevel: warning\nscope: code\ntokens:\n  - very\n";

#[test]
fn a_code_span_after_a_block_with_no_group_is_placed_where_it_is() -> TestResult {
    let suite = Suite::new("ignores-no-group");
    let root = suite.scenario("no-group");
    root.file("styles/Team/Code.yml", CODE);
    root.file(
        ".ctl/hakkiri.yaml",
        "packages:\n  - path: styles\nfiles:\n  '*.md':\n    styles: [Team]\n    block_ignores: ['(?s)BEGIN.*?END']\n",
    );
    root.file(
        "a.md",
        "A paragraph line one very\nBEGIN\ninside very\nEND\nline three really.\n\nText with `very` code and more.\n",
    );
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    assert_eq!(
        result.lines().first(),
        Some(&"a.md:7:12: warning Team.Code: Code 'very'."),
        "{}",
        result.out
    );
    Ok(())
}
