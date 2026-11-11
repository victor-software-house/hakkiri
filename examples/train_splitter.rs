//! Trains the English sentence-splitter model, and writes the model and the
//! list of its sources.
//!
//! ```text
//! cargo run --release --example train_splitter -- \
//!     --markdown corpus/repositories.txt \
//!     --text corpus/issues.txt --text corpus/books.txt \
//!     --sources data/splitter \
//!     --model data/splitter/en.tsv --report data/splitter/SOURCES.md
//! ```
//!
//! Each corpus file holds documents separated by a form feed (U+000C).
//! A `--markdown` file holds Markdown, and the trainer reads the text of its
//! paragraphs. A `--text` file holds plain text.
//! `mise run splitter:train` runs this after `mise run splitter:fetch`.

use std::error::Error;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use hakkiri::splitter::{Summary, train};
use hakkiri::{Language, Node, Scope, Tree, parse};

type Outcome<T> = Result<T, Box<dyn Error>>;

/// A corpus file and how to read its documents.
struct Corpus {
    path: PathBuf,
    markdown: bool,
}

/// What the command line sets.
struct Arguments {
    corpora: Vec<Corpus>,
    sources: PathBuf,
    model: PathBuf,
    report: PathBuf,
}

/// What one corpus file held.
struct Counts {
    name: String,
    documents: usize,
    skipped: usize,
    tokens: usize,
}

fn arguments() -> Outcome<Arguments> {
    let mut corpora = Vec::new();
    let (mut sources, mut model, mut report) = (None, None, None);
    let mut args = std::env::args().skip(1);
    while let Some(flag) = args.next() {
        let value = args.next().ok_or_else(|| format!("{flag} needs a value"))?;
        match flag.as_str() {
            "--markdown" => corpora.push(Corpus {
                path: value.into(),
                markdown: true,
            }),
            "--text" => corpora.push(Corpus {
                path: value.into(),
                markdown: false,
            }),
            "--sources" => sources = Some(PathBuf::from(value)),
            "--model" => model = Some(PathBuf::from(value)),
            "--report" => report = Some(PathBuf::from(value)),
            _ => return Err(format!("unknown flag {flag}").into()),
        }
    }
    Ok(Arguments {
        corpora,
        sources: sources.ok_or("--sources is required")?,
        model: model.ok_or("--model is required")?,
        report: report.ok_or("--report is required")?,
    })
}

/// The text of every paragraph of `tree`, in file order, one blank line after
/// each.
fn paragraphs(source: &str, tree: &Tree) -> String {
    let mut text = String::new();
    let mut pending: Vec<&Node> = tree.roots.iter().rev().collect();
    while let Some(node) = pending.pop() {
        if node.scope == Scope::Paragraph
            && let Some(paragraph) = node.text(source)
        {
            text.push_str(&paragraph);
            text.push_str("\n\n");
        }
        pending.extend(node.children.iter().rev());
    }
    text
}

/// Reads the documents of one corpus file.
///
/// Bytes that are not UTF-8 read as U+FFFD.
fn read(corpus: &Corpus) -> Outcome<(Vec<String>, Counts)> {
    let bytes = fs::read(&corpus.path)?;
    let text = String::from_utf8_lossy(&bytes);
    let mut documents = Vec::new();
    let mut skipped = 0;
    for document in text.split('\u{c}').filter(|part| !part.trim().is_empty()) {
        if corpus.markdown {
            match parse(Language::Markdown, document) {
                Ok(tree) => documents.push(paragraphs(document, &tree)),
                Err(_) => skipped += 1,
            }
        } else {
            documents.push(document.to_owned());
        }
    }
    let counts = Counts {
        name: corpus
            .path
            .file_stem()
            .map_or_else(String::new, |stem| stem.to_string_lossy().into_owned()),
        documents: documents.len(),
        skipped,
        tokens: documents
            .iter()
            .map(|document| document.split_whitespace().count())
            .sum(),
    };
    Ok((documents, counts))
}

/// The rows of a tab-separated file, each split into its fields.
fn rows(path: &Path) -> Outcome<Vec<Vec<String>>> {
    Ok(fs::read_to_string(path)?
        .lines()
        .map(|line| line.split('\t').map(str::to_owned).collect())
        .collect())
}

/// The text of `SOURCES.md`.
fn report(sources: &Path, counts: &[Counts], summary: &Summary) -> Outcome<String> {
    let repositories = rows(&sources.join("repositories.tsv"))?;
    let datasets = rows(&sources.join("datasets.tsv"))?;
    let licenses = fs::read_to_string(sources.join("licenses.txt"))?
        .lines()
        .count();
    let mut text = String::from(
        "# Splitter model sources\n\n\
         `en.tsv` is the output of `mise run splitter:train` on the sources below.  \n\
         Two runs on these sources write byte-identical files.\n\n\
         ## Corpus\n\n\
         | File | Documents | Tokens |\n|:--|--:|--:|\n",
    );
    for count in counts {
        writeln!(
            text,
            "| `{}` | {} | {} |",
            count.name, count.documents, count.tokens
        )?;
    }
    text.push_str(
        "\n## Repositories\n\n\
         `repositories.tsv` lists the Markdown sources: public repositories of the owner, each at the commit below.  \n\
         A repository is listed when it is public, is not a fork, is not archived, and is at most 10 MB.  \n\
         A license of `none` means the repository declares no license.  \n\
         The corpus holds every tracked `.md` file except `CHANGELOG.md`, and the trainer reads the text of its paragraphs.\n\n\
         | Repository | Commit | License |\n|:--|:--|:--|\n",
    );
    for row in &repositories {
        if let [name, commit, license] = row.as_slice() {
            writeln!(text, "| {name} | `{commit}` | {license} |")?;
        }
    }
    text.push_str(
        "\n## Datasets\n\n\
         [Common Pile](https://arxiv.org/abs/2506.05209) datasets on Hugging Face, each at a pinned revision.  \n\
         `datasets.tsv` lists the files read, and how many bytes of each; 0 means the whole file.\n\n\
         | Dataset | Revision | File | Bytes | License |\n|:--|:--|:--|--:|:--|\n",
    );
    for row in &datasets {
        if let [name, revision, file, bytes, license] = row.as_slice() {
            writeln!(
                text,
                "| {name} | `{revision}` | `{file}` | {bytes} | {license} |"
            )?;
        }
    }
    write!(
        text,
        "\n### Filters\n\n\
         - `common-pile/stackv2` ([dataset card](https://huggingface.co/datasets/common-pile/stackv2)) holds code files from GitHub repositories whose detected licenses are all on the Blue Oak list.  \n  \
         A record is kept when its `metadata.path` ends in `.md` or `.markdown`, in any case, and every license in its `metadata.license`, a comma-separated list, is one of the {licenses} ids in `licenses.txt`.  \n  \
         That file is the [Blue Oak Council list](https://blueoakcouncil.org/list), version 16.\n\
         - `common-pile/python_enhancement_proposals` ([dataset card](https://huggingface.co/datasets/common-pile/python_enhancement_proposals)) holds the Python Enhancement Proposals in the public domain, as plain text.  \n  \
         A record is kept when `metadata.license` is `Public Domain`.\n\
         - `common-pile/project_gutenberg` ([dataset card](https://huggingface.co/datasets/common-pile/project_gutenberg)) holds books.  \n  \
         A record is kept when `metadata.license` is `Public Domain` and `metadata.language` is `en`.\n\n\
         ## Result\n\n\
         | Count | Value |\n|:--|--:|\n\
         | Tokens read | {} |\n\
         | Word types | {} |\n\
         | Tokens that end in a period | {} |\n\
         | Possible sentence ends | {} |\n\
         | Abbreviations | {} |\n\
         | Collocations | {} |\n\
         | Sentence starters | {} |\n\
         | Capitalization entries seen | {} |\n\
         | Capitalization entries kept | {} |\n",
        summary.tokens,
        summary.types,
        summary.period_tokens,
        summary.sentence_breaks,
        summary.abbreviations,
        summary.collocations,
        summary.starters,
        summary.orthography_seen,
        summary.orthography,
    )?;
    Ok(text)
}

fn main() -> Outcome<()> {
    let arguments = arguments()?;
    let started = Instant::now();
    let mut corpora = Vec::new();
    let mut counts = Vec::new();
    for corpus in &arguments.corpora {
        let (documents, count) = read(corpus)?;
        eprintln!(
            "{}: {} documents, {} skipped, {} tokens",
            count.name, count.documents, count.skipped, count.tokens
        );
        corpora.push(documents);
        counts.push(count);
    }
    let documents: Vec<&str> = corpora.iter().flatten().map(String::as_str).collect();
    eprintln!("read in {:?}", started.elapsed());
    let started = Instant::now();
    let trained = train(&documents);
    eprintln!("trained in {:?}: {:?}", started.elapsed(), trained.summary);
    fs::write(&arguments.model, trained.model.to_tsv())?;
    fs::write(
        &arguments.report,
        report(&arguments.sources, &counts, &trained.summary)?,
    )?;
    Ok(())
}
