//! Front matter: the scenarios of the front-matter-values change.
//!
//! `tests/fixtures/front-matter` holds one package of nine rules (`styles`),
//! six Markdown files, one MDX file and one probe of YAML scalars (`text`, with
//! `.fixture` after the extension), and one config for each tool:
//! `vale/all.ini` and `.ctl/all.yaml`.
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
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/front-matter")
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

/// The file, the line, the first column and the matched text of an alert.
type Place = (&'static str, usize, usize, &'static str);

/// The alerts that Vale places in the wrong part of a value, and where hakkiri
/// places them: the file, the line, the first column, and the matched text.
///
/// Vale puts the words of the second line of a folded block scalar on its
/// first line.
/// It finds a TOML value by searching the front matter, so `title` in
/// `"A very toml title"` lands on the key.
/// It starts the text of a value with an explicit tag or an anchor at the tag
/// or the anchor, whichever comes first.
const MISPLACED: [(Place, Place); 9] = [
    (("text/f1.md", 4, 15, "text"), ("text/f1.md", 5, 3, "text")),
    (("text/f1.md", 4, 20, "here"), ("text/f1.md", 5, 8, "here")),
    (
        ("text/f2.md", 2, 1, "title"),
        ("text/f2.md", 2, 22, "title"),
    ),
    (
        ("text/f7.md", 36, 6, "tagged"),
        ("text/f7.md", 36, 12, "tagged"),
    ),
    (
        ("text/f11.md", 3, 8, "very"),
        ("text/f11.md", 3, 11, "very"),
    ),
    (
        ("text/f11.md", 3, 13, "anchored"),
        ("text/f11.md", 3, 16, "anchored"),
    ),
    (
        ("text/f11.md", 3, 22, "text"),
        ("text/f11.md", 3, 25, "text"),
    ),
    (
        ("text/f11.md", 5, 9, "very"),
        ("text/f11.md", 5, 18, "very"),
    ),
    (
        ("text/f11.md", 5, 14, "tagged"),
        ("text/f11.md", 5, 23, "tagged"),
    ),
];

/// Takes the misplaced alerts out of both sets, and asserts that each side
/// holds them for the same rules.
/// A `raw` rule reads the file as written, so Vale places its alerts right.
fn leave_out_the_misplaced(vale: &mut BTreeSet<Row>, ours: &mut BTreeSet<Row>) {
    let is = |row: &Row, (file, line, column, text): Place| {
        row.0 == file && row.1 == line && row.2 == column && row.7 == text && row.5 != "Team.Raw"
    };
    for (theirs, mine) in MISPLACED {
        let rules = |set: &mut BTreeSet<Row>, place: Place| {
            let found: BTreeSet<String> = set
                .iter()
                .filter(|row| is(row, place))
                .map(|row| row.5.clone())
                .collect();
            set.retain(|row| !is(row, place));
            found
        };
        let (from_vale, from_ours) = (rules(vale, theirs), rules(ours, mine));
        assert!(!from_vale.is_empty(), "Vale gave no alert at {theirs:?}");
        assert_eq!(from_vale, from_ours, "the rules at {theirs:?} and {mine:?}");
    }
}

/// The alerts that Vale gives on the lines of a `;;;` block and hakkiri does
/// not, each as a file, a line, a column and a rule.
///
/// Vale lints the lines of a `;;;` block as a paragraph too (design decision 3
/// of `front-matter-crate`), where hakkiri cuts every block from the body.
/// The `Team.Text` alert at line 2, column 14 of `text/f8.md` stays, because
/// the title is a value of the front matter, which that rule reads.
const BLOCK_AS_PARAGRAPH: [(&str, usize, usize, &str); 7] = [
    ("text/f8.md", 2, 14, "Team.Para"),
    ("text/f8.md", 2, 54, "Team.Para"),
    ("text/f8.md", 2, 79, "Team.Para"),
    ("text/f8.md", 2, 54, "Team.Text"),
    ("text/f8.md", 2, 79, "Team.Text"),
    ("text/f9.md", 3, 15, "Team.Para"),
    ("text/f9.md", 4, 20, "Team.Para"),
];

/// Takes the alerts of the `;;;` blocks as a paragraph out of Vale's set,
/// after asserting that Vale holds each, and asserts that hakkiri holds none.
fn leave_out_the_block_as_paragraph(vale: &mut BTreeSet<Row>, ours: &BTreeSet<Row>) {
    for (file, line, column, rule) in BLOCK_AS_PARAGRAPH {
        let is = |row: &Row| row.0 == file && row.1 == line && row.2 == column && row.5 == rule;
        assert!(
            vale.iter().any(is),
            "Vale gave no {rule} at {file}:{line}:{column}"
        );
        vale.retain(|row| !is(row));
        assert!(
            !ours.iter().any(is),
            "hakkiri gave {rule} at {file}:{line}:{column}"
        );
    }
}

/// The configs of the fixture: no pattern, a token pattern with no group and a
/// token pattern with a group. Vale reads the metadata before it rewrites the
/// body, so the patterns change no value of the front matter.
const CONFIGS: [&str; 3] = ["all", "nogroup", "group"];

#[test]
fn every_config_gives_the_alerts_vale_gives() -> TestResult {
    let suite = Suite::new("front-matter-parity");
    let root = suite.scenario("parity");
    for dir in ["styles", "text", ".ctl"] {
        copy_into(&root, &fixture().join(dir), Path::new(dir))?;
    }
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
        assert!(!result.lines().is_empty(), "{}", result.out);
        let mut ours = rows_of(&result.out)?;
        let mut vale = vale_rows(name)?;
        leave_out_the_misplaced(&mut vale, &mut ours);
        leave_out_the_block_as_paragraph(&mut vale, &ours);
        let only_vale: Vec<_> = vale.difference(&ours).collect();
        let only_ours: Vec<_> = ours.difference(&vale).collect();
        assert!(
            only_vale.is_empty() && only_ours.is_empty(),
            "{name}: only Vale gave {only_vale:#?}, only hakkiri gave {only_ours:#?}"
        );
    }
    Ok(())
}

/// Front matter that no parser reads, and a sequence key that one reads.
///
/// Vale fails each file of the first three with E201 on line 1 and lints none
/// of it, and it fails the sequence key too.
/// hakkiri fails the file with the line of the parser's problem and lints the
/// rest of it, and lints the rest of the sequence key's block, which a parser
/// reads.
#[test]
fn front_matter_no_parser_reads_fails_the_file_and_leaves_the_rest_linted() -> TestResult {
    let suite = Suite::new("front-matter-broken");
    let root = suite.scenario("broken");
    for dir in ["styles", ".ctl"] {
        copy_into(&root, &fixture().join(dir), Path::new(dir))?;
    }
    root.file(
        "text/ok.md",
        "---\ntitle: A very title\n---\n\nBody very.\n",
    );
    root.file(
        "text/yaml.md",
        "---\ntitle: very\nother: [open\n---\n\nBody very.\n",
    );
    root.file("text/toml.md", "+++\ntitle = \"open\n+++\n\nBody very.\n");
    root.file(
        "text/json.md",
        ";;;\n{\"title\": \"very\", \n;;;\n\nBody very.\n",
    );
    root.file(
        "text/seq.md",
        "---\n? [a, b]\n: very seq\ntitle: very after\n---\n\nBody very.\n",
    );
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
    assert_eq!(result.code, 2, "{}{}", result.out, result.err);
    let json: Value = serde_json::from_str(&result.out)?;
    let failed: BTreeSet<(&str, bool)> = json["failures"]
        .as_array()
        .ok_or("no failures array")?
        .iter()
        .map(|failure| {
            (
                failure["path"].as_str().unwrap_or_default(),
                failure["message"]
                    .as_str()
                    .is_some_and(|message| message.starts_with("front matter: line ")),
            )
        })
        .collect();
    assert_eq!(
        failed,
        BTreeSet::from([
            ("text/json.md", true),
            ("text/toml.md", true),
            ("text/yaml.md", true)
        ])
    );
    let rows = rows_of(&result.out)?;
    let body = |file: &str, line: usize| {
        (
            file.to_owned(),
            line,
            6,
            9,
            "warning".to_owned(),
            "Team.Para".to_owned(),
            "Para 'very'".to_owned(),
            "very".to_owned(),
        )
    };
    for (file, line) in [
        ("text/ok.md", 5),
        ("text/yaml.md", 6),
        ("text/toml.md", 5),
        ("text/json.md", 5),
        ("text/seq.md", 7),
    ] {
        assert!(rows.contains(&body(file, line)), "{file}: {rows:#?}");
    }
    let after = (
        "text/seq.md".to_owned(),
        4,
        8,
        11,
        "warning".to_owned(),
        "Team.Lit".to_owned(),
        "Lit 'very'".to_owned(),
        "very".to_owned(),
    );
    assert!(rows.contains(&after), "{rows:#?}");
    Ok(())
}

/// Every form of front matter is metadata: its values are lints of the
/// `frontmatter` scope, and no line of the block is a paragraph.
///
/// Each file holds a title and the body `Body very.`, after a blank line.
/// The last file holds a JSON key `very` and no string value.
#[test]
fn every_form_of_block_is_front_matter_and_no_body_text() -> TestResult {
    let suite = Suite::new("front-matter-forms");
    let root = suite.scenario("forms");
    for dir in ["styles", ".ctl"] {
        copy_into(&root, &fixture().join(dir), Path::new(dir))?;
    }
    let yaml = "title: A very title";
    let toml = "title = \"A very title\"";
    let json = "{\"title\": \"A very title\"}";
    let forms = [
        ("fence", format!("---\n{yaml}\n---"), 2),
        ("yaml", format!("---yaml\n{yaml}\n---"), 2),
        ("toml", format!("---toml\n{toml}\n---"), 2),
        ("json", format!("---json\n{json}\n---"), 2),
        ("semicolons", format!(";;;\n{json}\n;;;"), 2),
        ("plus", format!("+++\n{toml}\n+++"), 2),
        ("bare", json.to_owned(), 1),
        (
            "bare-lines",
            "{\n\"title\": \"A very title\"\n}".to_owned(),
            2,
        ),
        ("dots", format!("---\n{yaml}\n..."), 2),
    ];
    let mut expected = Vec::new();
    for (name, block, title) in forms {
        let file = format!("text/{name}.md");
        let body = block.lines().count() + 2;
        root.file(&file, format!("{block}\n\nBody very.\n"));
        expected.push((
            file,
            BTreeSet::from([(title, "Team.FmTitle"), (body, "Team.Para")]),
        ));
    }
    root.file("text/key.md", ";;;\n{\"very\": 1}\n;;;\n\nBody very.\n");
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
    let rows = rows_of(&result.out)?;
    let lines_of = |file: &str, rules: &[&str]| -> BTreeSet<(usize, String)> {
        rows.iter()
            .filter(|row| row.0 == file && rules.contains(&row.5.as_str()))
            .map(|row| (row.1, row.5.clone()))
            .collect()
    };
    for (file, lines) in expected {
        let wanted = lines
            .into_iter()
            .map(|(line, rule)| (line, rule.to_owned()))
            .collect();
        assert_eq!(
            lines_of(&file, &["Team.Para", "Team.FmTitle"]),
            wanted,
            "{file}: {rows:#?}"
        );
    }
    assert_eq!(
        lines_of("text/key.md", &["Team.Para", "Team.Text"]),
        BTreeSet::from([(5, "Team.Para".to_owned()), (5, "Team.Text".to_owned())]),
        "{rows:#?}"
    );
    Ok(())
}
