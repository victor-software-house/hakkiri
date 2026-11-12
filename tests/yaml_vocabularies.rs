//! YAML vocabularies: the scenarios of the yaml-vocabularies change.
//!
//! `tests/fixtures/yaml-vocabularies` holds one package of rules and
//! vocabularies (`styles`), two Markdown files (`text`, with `.fixture` after
//! `.md`), and a config for each tool: `vale/<name>.ini` and
//! `.ctl/<name>.yaml`.
//!
//! `vale.tsv` holds one row for each alert Vale gave on the files under each
//! of the first four configs: the config, the file, the line, the first and
//! last column, the level, the rule, the message and the matched text.
//! `errors.tsv` holds one row for each of the other configs, whose vocabulary
//! has one fault: the config, and the error line Vale gave. The rows come from
//! Vale at `66bbb85` run from the fixture folder as `vale
//! --config=vale/<name>.ini --output=JSON text` and `--output=line text`, with
//! `Vale.Spelling` and `Vale.Repetition` turned off in the first four, as the
//! hakkiri configs turn them off.

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

/// The configs whose alerts Vale gave.
const CONFIGS: [&str; 4] = ["none", "all", "bool-yes", "empty-entry"];

/// A file, a line, the first and last column, the level, the rule, the
/// message and the matched text.
type Row = (String, usize, usize, usize, String, String, String, String);

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/yaml-vocabularies")
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
    let suite = Suite::new("yaml-vocabularies-parity");
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

/// A faulty config, the vocabulary file Vale named for it, and the line.
type Fault = (String, String, usize);

/// The fault of each faulty config.
fn vale_errors() -> Result<Vec<Fault>, Box<dyn Error>> {
    let mut errors = Vec::new();
    for row in fs::read_to_string(fixture().join("errors.tsv"))?.lines() {
        let (case, error) = row
            .split_once('\t')
            .ok_or("a row of errors.tsv has no tab")?;
        let mut parts = error.splitn(3, ':');
        let (Some(file), Some(line)) = (parts.next(), parts.next()) else {
            return Err(format!("the row of `{case}` holds no file and line").into());
        };
        errors.push((case.to_owned(), file.to_owned(), line.parse()?));
    }
    Ok(errors)
}

#[test]
fn every_fault_is_refused_on_the_file_and_line_vale_names() -> TestResult {
    let suite = Suite::new("yaml-vocabularies-errors");
    let root = project(&suite, "errors")?;
    let errors = vale_errors()?;
    assert_eq!(errors.len(), 12, "one row for each fault of the fixture");
    for (case, file, line) in errors {
        let config = format!(".ctl/{case}.yaml");
        let result = run(root.path(), &["check", "--config", &config, "text"], &[])?;
        assert_eq!(result.code, 2, "{case}: {}{}", result.out, result.err);
        // The unfinished list opens on line 3. Vale names line 2, which holds
        // a complete entry, so hakkiri names the line of the bracket instead.
        let line = if case == "yaml-syntax" { 3 } else { line };
        let wanted = format!("failed {file}: line {line}: ");
        assert!(result.err.contains(&wanted), "{case}: {}", result.err);
    }
    Ok(())
}

/// A project with one vocabulary file `body` and the text `text`, checked
/// with a rule that finds `foo`, `foozz` and `fooqq`.
fn small<'a>(suite: &'a Suite, name: &str, body: &str, text: &str) -> Root<'a> {
    let root = suite.scenario(name);
    root.file("styles/config/vocabularies/Team/vocab.yml", body);
    root.file(
        "styles/Team/Probe.yml",
        "extends: existence\nmessage: \"Found '%s'.\"\nlevel: warning\ntokens: ['foo(?:zz|qq)?', Beta, Gamma]\n",
    );
    root.file(
        ".ctl/hakkiri.yaml",
        "packages:\n  - path: styles\nvocab: [Team]\nfiles:\n  'a.md': {styles: [Team]}\n",
    );
    root.file("a.md", text);
    root
}

#[test]
fn every_faulty_entry_of_a_file_is_reported_at_once() -> TestResult {
    let suite = Suite::new("yaml-vocabularies-many");
    let root = small(
        &suite,
        "many",
        "accept:\n  - ' '\n  - a:\n      regx: true\n  - 'b(':\n      regex: true\n  - Beta\n",
        "Fine.\n",
    );
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 2, "{}{}", result.out, result.err);
    assert!(result.lines().is_empty(), "{}", result.out);
    let file = "styles/config/vocabularies/Team/vocab.yml";
    for line in [2, 4, 5] {
        assert!(
            result
                .err
                .contains(&format!("failed {file}: line {line}: ")),
            "line {line}: {}",
            result.err
        );
    }
    assert_eq!(result.err.matches("failed ").count(), 3, "{}", result.err);
    Ok(())
}

#[test]
fn a_key_the_file_does_not_define_is_refused_on_its_line() -> TestResult {
    let suite = Suite::new("yaml-vocabularies-unknown");
    let root = small(
        &suite,
        "unknown",
        "accept:\n  - Beta\naccpt:\n  - Gamma\n",
        "Fine.\n",
    );
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 2, "{}{}", result.out, result.err);
    assert!(
        result
            .err
            .contains("failed styles/config/vocabularies/Team/vocab.yml: line 3: "),
        "{}",
        result.err
    );
    Ok(())
}

#[test]
fn the_affix_file_is_looked_up_in_the_holder_first_then_the_other_packages() -> TestResult {
    let suite = Suite::new("yaml-vocabularies-affix");
    let root = small(
        &suite,
        "affix",
        "affix: custom\naccept:\n  - foo:\n      flags: X\n",
        "foo foozz fooqq.\n",
    );
    let aff = |suffix: &str| format!("SET UTF-8\nSFX X Y 1\nSFX X 0 {suffix} .\n");
    root.file(
        ".ctl/hakkiri.yaml",
        "packages:\n  - path: other\n  - path: styles\nvocab: [Team]\nfiles:\n  'a.md': {styles: [Team]}\n",
    );
    root.file("other/config/dictionaries/custom.aff", aff("qq"));
    // The package that holds the vocabulary comes first, whatever the order of
    // the packages.
    root.file("styles/config/dictionaries/custom.aff", aff("zz"));
    let result = run(root.path(), &["check", "--format", "json"], &[])?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    let matched: Vec<String> = rows_of(&result.out)?.into_iter().map(|row| row.7).collect();
    assert_eq!(matched, ["fooqq"], "{}", result.out);
    fs::remove_file(root.path().join("styles/config/dictionaries/custom.aff"))?;
    let result = run(root.path(), &["check", "--format", "json"], &[])?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    let matched: Vec<String> = rows_of(&result.out)?.into_iter().map(|row| row.7).collect();
    assert_eq!(matched, ["foozz"], "{}", result.out);
    Ok(())
}

#[test]
fn an_affix_name_is_a_file_name() -> TestResult {
    let suite = Suite::new("yaml-vocabularies-affix-name");
    let root = small(
        &suite,
        "name",
        "affix: ../outside\naccept:\n  - Beta\n",
        "Fine.\n",
    );
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 2, "{}{}", result.out, result.err);
    assert!(
        result
            .err
            .contains("failed styles/config/vocabularies/Team/vocab.yml: line 1: affix: `../outside` is not a file name"),
        "{}",
        result.err
    );
    Ok(())
}
