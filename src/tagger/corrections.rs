//! Corrections to the tags of one sentence, made after the model has tagged it.
//!
//! The model reads UD English EWT, a corpus of web text.
//! On the prose of our own documents it is wrong in a few cases that repeat.
//! Each correction below is a rule with its reason, not a second model.
//! The rules read the words and the tags of the sentence only, so a word the
//! model decides is never fed back into its context.

/// The tags of a noun, which a possessive `'s` or a verb can follow.
const NOUNS: [&str; 4] = ["NN", "NNS", "NNP", "NNPS"];

/// The tags of a verb, and of a modal, which make a clause.
const VERBS: [&str; 7] = ["VB", "VBD", "VBG", "VBN", "VBP", "VBZ", "MD"];

/// The tags of the words that start the object of a verb: a determiner, a
/// possessive pronoun, a number and a pronoun.
const OBJECT_STARTS: [&str; 4] = ["DT", "PRP$", "CD", "PRP"];

/// The tags of a word that is no noun or verb, so a capitalised heading word
/// such as `THE`, `NOT` or `ALL` keeps its ordinary tag.
const FUNCTION_WORDS: [&str; 18] = [
    "DT", "IN", "CC", "PRP", "PRP$", "TO", "WDT", "WP", "WP$", "WRB", "EX", "PDT", "RP", "CD",
    "RB", "RBR", "RBS", "UH",
];

/// The units of size that stay as the model tags them, `MB` among them.
const UNITS: [&str; 5] = ["KB", "MB", "GB", "TB", "PB"];

/// The modals that RFC 2119 states a requirement with, as the model never reads
/// them written in capitals: it reads `SHALL` as a noun, a verb or `TO`.
const MODALS: [&str; 6] = ["SHALL", "MUST", "MAY", "SHOULD", "WILL", "CAN"];

/// The tags of `words` after the corrections.
///
/// `tags` holds the model's tag of each word, and a correction replaces it.
/// A rule that reads the tag before a word reads the corrected tag.
pub(super) fn apply<'a>(words: &[&str], tags: &mut [&'a str]) {
    let mut before = "";
    for (word, tag) in words.iter().zip(tags.iter_mut()) {
        if let Some(corrected) = corrected(word, before, *tag) {
            *tag = corrected;
        }
        before = *tag;
    }
    verbs_after_noun_phrases(words, tags);
}

/// The tag a rule gives `word`, which follows a word tagged `before` and which
/// the model tagged `model`.
fn corrected(word: &str, before: &str, model: &str) -> Option<&'static str> {
    if word == ";" {
        // The model reads every semicolon as a comma, and the corpus tags it
        // `:`.
        Some(":")
    } else if MODALS.contains(&word) {
        Some("MD")
    } else if is_all_capitals(word) {
        // An acronym or a name in capitals, such as `CSS` or `SSH`, where the
        // model reads a noun, an adjective or a verb. A function word in a
        // heading, and a unit, keep the model's tag.
        (!FUNCTION_WORDS.contains(&model) && !UNITS.contains(&word)).then_some("NNP")
    } else if is_possessive(word) && NOUNS.contains(&before) {
        // The model reads the `'s` after a noun as `VBZ`, `NNS` or `NN`.
        Some("POS")
    } else {
        None
    }
}

/// Reads a plural noun as the verb of the clause when a noun comes before it
/// and an object or a preposition after it.
///
/// The model reads `reads` in `The freshness harness reads stdout.` as a plural
/// noun, as it reads a verb that ends in `s` after a compound of nouns.
/// The noun before it ends a noun phrase, and a determiner, a possessive
/// pronoun, a number or a pronoun after it starts the object.
/// A noun, an adjective or a preposition other than `of` after it starts the
/// object or a phrase only when no other word of the sentence is a verb, since
/// a compound noun with a plural head, such as `layout rules of the widgets`,
/// reads the same.
fn verbs_after_noun_phrases(words: &[&str], tags: &mut [&str]) {
    let others = tags.iter().filter(|tag| VERBS.contains(tag)).count();
    for index in 1..tags.len() {
        let (Some(&before), Some(&tag)) = (tags.get(index - 1), tags.get(index)) else {
            continue;
        };
        if tag != "NNS" || !matches!(before, "NN" | "NNP") {
            continue;
        }
        let after = tags.get(index + 1).copied().unwrap_or_default();
        let object = OBJECT_STARTS.contains(&after);
        let bare = others == 0
            && match after {
                "NN" | "NNP" | "JJ" => true,
                "IN" => !words
                    .get(index + 1)
                    .is_some_and(|next| next.eq_ignore_ascii_case("of")),
                _ => false,
            };
        if (object || bare)
            && let Some(slot) = tags.get_mut(index)
        {
            *slot = "VBZ";
        }
    }
}

/// Whether `word` is two or more letters, all capitals.
fn is_all_capitals(word: &str) -> bool {
    word.len() >= 2 && word.bytes().all(|byte| byte.is_ascii_uppercase())
}

/// Whether `word` is the ending `'s` that the tokenizer splits off a noun.
fn is_possessive(word: &str) -> bool {
    word == "'s" || word == "\u{2019}s"
}
