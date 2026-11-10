//! The character set of a dictionary's two files.
//!
//! A port of `encoding.go` in [Vale]'s `spell` package at `66bbb85`, for UTF-8
//! and ISO 8859-1 only.
//!
//! [Vale]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/spell/encoding.go

use super::DictionaryError;

/// The byte order mark some dictionaries start with.
const BOM: &[u8] = b"\xef\xbb\xbf";

/// The value of the `SET` directive of an affix file, if it has one.
fn declared(aff: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(aff);
    text.lines().find_map(|line| {
        let mut fields = line.split_whitespace();
        (fields.next() == Some("SET")).then(|| fields.next().map(str::to_owned))?
    })
}

/// The text of the two files in UTF-8.
///
/// Hunspell's default is ISO 8859-1, but a file with no `SET` that is valid
/// UTF-8 is read as UTF-8.
///
/// # Errors
///
/// Returns a [`DictionaryError`] when a file is not valid UTF-8 and declares
/// UTF-8, or when it declares another character set than ISO 8859-1.
pub(super) fn decode(aff: &[u8], dic: &[u8]) -> Result<(String, String), DictionaryError> {
    let aff = aff.strip_prefix(BOM).unwrap_or(aff);
    let dic = dic.strip_prefix(BOM).unwrap_or(dic);
    let utf8 = |bytes: &[u8]| String::from_utf8(bytes.to_vec()).or(Err(DictionaryError::NotUtf8));
    let latin1 = |bytes: &[u8]| {
        bytes
            .iter()
            .map(|byte| char::from(*byte))
            .collect::<String>()
    };
    match declared(aff) {
        None if std::str::from_utf8(aff).is_ok() && std::str::from_utf8(dic).is_ok() => {
            Ok((utf8(aff)?, utf8(dic)?))
        }
        None => Ok((latin1(aff), latin1(dic))),
        Some(set) => {
            let name = set.to_ascii_uppercase().replace("ISO8859-", "ISO-8859-");
            match name.as_str() {
                "UTF-8" | "UTF8" => Ok((utf8(aff)?, utf8(dic)?)),
                "ISO-8859-1" | "LATIN1" => Ok((latin1(aff), latin1(dic))),
                _ => Err(DictionaryError::Charset { set }),
            }
        }
    }
}
