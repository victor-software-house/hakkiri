//! The unit tests of the readers and their langdex languages.

use std::path::Path;

use super::Language;

#[test]
fn every_reader_maps_to_a_langdex_language_and_back() {
    for reader in Language::ALL {
        let language = reader.langdex();
        assert_eq!(
            language.map(|language| language.name),
            Some(reader.name()),
            "{reader:?}"
        );
        assert_eq!(language.and_then(Language::of), Some(reader), "{reader:?}");
    }
}

#[test]
fn a_language_hakkiri_has_no_reader_for_maps_to_none() {
    let python = langdex::by_name("python");
    assert!(python.is_some());
    assert_eq!(python.and_then(Language::of), None);
}

#[test]
fn extension_gives_the_first_extension_of_each_reader() {
    let got: Vec<&str> = Language::ALL.into_iter().map(Language::extension).collect();
    assert_eq!(
        got,
        [
            "md", "mdx", "rs", "ts", "tsx", "js", "go", "swift", "yml", "toml", "sh", "txt"
        ]
    );
}

#[test]
fn from_path_reads_the_langdex_language_of_the_path() {
    let of = |path: &str| Language::from_path(Path::new(path));
    assert_eq!(of("a.mts"), Some(Language::TypeScript));
    assert_eq!(of("a.markdown"), Some(Language::Markdown));
    assert_eq!(of("docs/a.md"), Some(Language::Markdown));
    assert_eq!(of("a.jsx"), Some(Language::JavaScript));
    assert_eq!(of("a.tsx"), Some(Language::Tsx));
    assert_eq!(of("LICENSE"), Some(Language::Text));
    assert_eq!(of("notes.txt"), Some(Language::Text));
    assert_eq!(of("Cargo.toml"), Some(Language::Toml));
    assert_eq!(of("README"), None);
}

#[test]
fn from_reader_reads_any_name_of_a_reader() {
    let of = Language::from_reader;
    for name in ["md", "markdown", "Markdown"] {
        assert_eq!(of(name), Some(Language::Markdown), "{name}");
    }
    assert_eq!(of("text"), Some(Language::Text));
    assert_eq!(of("txt"), Some(Language::Text));
    assert_eq!(of("ts"), Some(Language::TypeScript));
    assert_eq!(of("tsx"), Some(Language::Tsx));
    assert_eq!(of("python"), None);
    assert_eq!(of("loud"), None);
}
