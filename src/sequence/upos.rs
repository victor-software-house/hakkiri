//! Universal part-of-speech tags, as the Penn Treebank tags Vale maps them to.
//!
//! Penn is the finer tagset, so most universal tags widen to a list of Penn
//! tags. Three are not a list of Penn tags, because what Penn leaves out is
//! syntactic: Penn has no `AUX`, `IN` covers both `ADP` and `SCONJ`, and `TO`
//! is `PART` in "to eat" but `ADP` in "to the store". Vale approximates `AUX`
//! and `SCONJ` with a list of words beside the tag, and so does this module
//! ([upos.go]).
//!
//! [upos.go]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/upos.go#L10-L127

/// Each universal tag, sorted, with the Penn tags it stands for as a pattern.
const TAGS: [(&str, &str); 17] = [
    ("ADJ", "JJ|JJR|JJS"),
    ("ADP", "IN|RP"),
    ("ADV", "RB|RBR|RBS|WRB"),
    ("AUX", "MD|VB|VBD|VBG|VBN|VBP|VBZ"),
    ("CCONJ", "CC"),
    ("DET", r"DT|PDT|WDT|WP\$"),
    ("INTJ", "UH"),
    ("NOUN", "NN|NNS"),
    ("NUM", "CD"),
    ("PART", "TO|RP|POS"),
    ("PRON", r"PRP|PRP\$|WP|EX"),
    ("PROPN", "NNP|NNPS"),
    ("PUNCT", r"[.,:]|''|``|-LRB-|-RRB-|\(|\)"),
    ("SCONJ", "IN|WRB"),
    ("SYM", r"SYM|\$|#"),
    ("VERB", "VB|VBD|VBG|VBN|VBP|VBZ"),
    ("X", "FW|LS"),
];

/// The words a Penn tag that is not enough narrows a universal tag to.
const WORDS: [(&str, &[&str]); 2] = [
    (
        "AUX",
        &[
            "am", "is", "are", "was", "were", "be", "been", "being", "have", "has", "had",
            "having", "do", "does", "did", "doing", "will", "would", "shall", "should", "can",
            "could", "may", "might", "must", "ought", "'s", "'re", "'m", "'ve", "'ll", "'d",
        ],
    ),
    (
        "SCONJ",
        &[
            "after", "although", "as", "because", "before", "even", "if", "since", "so", "than",
            "that", "though", "unless", "until", "when", "whenever", "where", "whereas",
            "wherever", "whether", "while", "why", "how",
        ],
    ),
];

/// The names of the universal tags, as an error lists them.
pub(crate) fn names() -> String {
    TAGS.map(|(name, _)| name).join(", ")
}

/// The pattern for the Penn tags of `name`: one universal tag, or several
/// joined by `|`.
///
/// The pattern also accepts each universal tag's own name, which a tagger of
/// another language gives.
///
/// # Errors
///
/// Returns the part of `name` that is not a universal tag.
pub(crate) fn tag_pattern(name: &str) -> Result<String, String> {
    let mut alternatives = Vec::new();
    for part in name.split('|') {
        let part = part.trim();
        let upper = part.to_uppercase();
        let (_, penn) = TAGS
            .iter()
            .find(|(tag, _)| *tag == upper)
            .ok_or_else(|| part.to_owned())?;
        alternatives.push((*penn).to_owned());
        alternatives.push(upper);
    }
    Ok(format!("^(?:{})$", alternatives.join("|")))
}

/// The pattern for the words a token of `name` is limited to, or `None` when
/// a tag in `name` holds any word.
pub(crate) fn word_pattern(name: &str) -> Option<String> {
    let mut words: Vec<String> = Vec::new();
    for part in name.split('|') {
        let upper = part.trim().to_uppercase();
        let (_, listed) = WORDS.iter().find(|(tag, _)| *tag == upper)?;
        words.extend(listed.iter().map(|word| regex::escape(word)));
    }
    Some(format!("^(?:{})$", words.join("|")))
}
