//! The unit tests of the `formats` entries.

#![expect(
    clippy::panic_in_result_fn,
    reason = "A test returns Result for `?` and fails through an assertion."
)]

use hakkiri::Language;

use super::Formats;

#[test]
fn a_leading_slash_ties_a_formats_glob_to_the_root() -> Result<(), String> {
    let mut formats = Formats::default();
    formats.insert("/a.md", "text")?;
    assert_eq!(formats.language_of("a.md"), Some(Language::Text));
    assert_eq!(formats.language_of("docs/a.md"), None);
    Ok(())
}

#[test]
fn a_formats_glob_with_no_slash_matches_the_file_name_in_any_folder() -> Result<(), String> {
    let mut formats = Formats::default();
    formats.insert("*.cfg", "md")?;
    assert_eq!(formats.language_of("conf/x.cfg"), Some(Language::Markdown));
    Ok(())
}

#[test]
fn each_alternative_of_a_formats_glob_reads_on_its_own() -> Result<(), String> {
    let mut formats = Formats::default();
    formats.insert("{*.cfg,conf/x}", "md")?;
    assert_eq!(formats.language_of("a/b.cfg"), Some(Language::Markdown));
    assert_eq!(formats.language_of("conf/x"), Some(Language::Markdown));
    assert_eq!(formats.language_of("other/conf/x"), None);
    Ok(())
}
