//! Corrections to the tags of one sentence, made after the model has tagged it.
//!
//! The model reads UD English EWT, a corpus of web text.
//! On the prose of our own documents it is wrong in a few cases that repeat.
//! Each correction below is one rule, with the reason for it, and none is a
//! second model. The rules read the words and the tags of the sentence only, so
//! a word the model decides is never fed back into its context.

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

/// The forms of the verbs of saying and thinking that prose never uses as a
/// noun, so a noun tag on one is the model reading a compound.
/// On UD English GUM, dev and test, each of them is a verb at every use.
const NEVER_NOUNS: [&str; 16] = [
    "say",
    "says",
    "tell",
    "tells",
    "know",
    "knows",
    "think",
    "thinks",
    "believe",
    "believes",
    "understand",
    "understands",
    "insist",
    "insists",
    "agree",
    "agrees",
];

/// The verb of asking, which a noun phrase can also name, as in `the asks that
/// wait`.
const ALSO_NOUNS: [&str; 2] = ["ask", "asks"];

/// The past forms of the verbs of asking and telling, which a noun phrase can
/// also hold as a participle: `the hook told of the file`.
const PAST_SAYING: [&str; 2] = ["asked", "told"];

/// The stems of the verbs that take a clause with no `that`, after which a noun
/// phrase can start a clause: `says the flag asks for rows`.
const COMPLEMENT_STEMS: [&str; 35] = [
    "say", "said", "tell", "told", "know", "knew", "think", "thought", "believ", "show", "check",
    "mean", "ensur", "note", "find", "found", "see", "saw", "hope", "guess", "assum", "expect",
    "suppos", "imagin", "prov", "confirm", "report", "explain", "decid", "learn", "notic",
    "rememb", "feel", "felt", "verif",
];

/// The prepositions that take a noun phrase as their object and no clause.
const PREPOSITIONS: [&str; 33] = [
    "for", "of", "from", "with", "by", "in", "on", "at", "into", "onto", "about", "under", "over",
    "through", "without", "within", "between", "during", "per", "across", "against", "among",
    "via", "upon", "near", "toward", "towards", "beyond", "behind", "beside", "off", "out", "to",
];

/// The tags of the words a noun phrase holds between its determiner and its
/// head.
const MODIFIERS: [&str; 8] = ["NN", "NNS", "NNP", "NNPS", "JJ", "CD", "POS", "VBG"];

/// The tags of the words that can be the subject of a verb of saying.
const SUBJECTS: [&str; 5] = ["NN", "NNS", "NNP", "NNPS", "JJ"];

/// Words that technical prose uses as nouns where the model reads one other
/// part of speech: `check`, `split`, `round`, `prompt`, `fade` and `thinking`
/// name a thing in `the id check`, `the split` and `the thinking mode`.
const TECHNICAL_NOUNS: [&str; 6] = ["check", "split", "round", "prompt", "fade", "thinking"];

/// The tags of the words after the corrections.
///
/// `tags` holds the model's tag of each word, and a correction replaces it.
/// A rule that reads the tag before a word reads the corrected tag.
pub(super) fn apply(words: &[&str], tags: &mut [&str]) {
    let mut before = "";
    for (word, tag) in words.iter().zip(tags.iter_mut()) {
        if let Some(corrected) = corrected(word, before, tag) {
            *tag = corrected;
        }
        before = *tag;
    }
    verbs_after_noun_phrases(words, tags);
    verb_at_start(words, tags);
    nouns_after_possessives(words, tags);
    lowercase_name_at_start(words, tags);
    verbs_of_saying(words, tags);
    subjects_of_verbs_of_saying(words, tags);
    pull_requests(words, tags);
    nouns_of_technical_prose(words, tags);
    asking_in_noun_phrases(words, tags);
}

/// Reads a plural noun that starts a sentence before a determiner as the verb.
///
/// The model reads `Takes` in `Takes every waiting ask` as a plural noun.
/// A noun that starts a sentence is not followed by a determiner, and the doc
/// comment of a function starts with the verb that has no subject.
fn verb_at_start(words: &[&str], tags: &mut [&str]) {
    let relative = words
        .get(1)
        .is_some_and(|word| word.eq_ignore_ascii_case("that"));
    if tags.first() == Some(&"NNS")
        && matches!(tags.get(1), Some(&("DT" | "PRP$")))
        && !relative
        && let Some(tag) = tags.first_mut()
    {
        *tag = "VBZ";
    }
}

/// Reads a base form after a possessive as the noun it continues.
///
/// The model reads `resolver` in `the client's resolver asks` as a verb, as it
/// reads the word after a `'s` that is a contraction.
/// A possessive is followed by the noun it owns, so a base form of a verb
/// after it is a noun, or `own`, an adjective.
fn nouns_after_possessives(words: &[&str], tags: &mut [&str]) {
    for index in 1..tags.len() {
        let contraction = index >= 2
            && words
                .get(index - 2)
                .is_some_and(|word| word.eq_ignore_ascii_case("let"));
        let own = words
            .get(index)
            .is_some_and(|word| word.eq_ignore_ascii_case("own"));
        if tags.get(index - 1) == Some(&"POS")
            && !contraction
            && let Some(tag) = tags.get_mut(index)
            && matches!(*tag, "VB" | "VBP")
        {
            *tag = if own { "JJ" } else { "NN" };
        }
    }
}

/// Reads a lowercase word that starts a sentence before a verb of saying as a
/// name.
///
/// The model reads `cargo` in `cargo asks again` as an imperative, as it reads
/// every first word.
/// A sentence of prose starts with a capital, so a lowercase first word is a
/// name, such as a command, and a verb follows it.
fn lowercase_name_at_start(words: &[&str], tags: &mut [&str]) {
    let lowercase = words
        .first()
        .is_some_and(|word| word.starts_with(|first: char| first.is_ascii_lowercase()));
    let verb = words.get(1).is_some_and(|word| says(word) || asks(word));
    if lowercase
        && verb
        && let Some(tag) = tags.first_mut()
        && *tag == "VB"
    {
        *tag = "NN";
    }
}

/// Reads `pull` before `request` as the noun of the compound.
///
/// The model reads `pull` in `the queue load split pull request` as a verb,
/// which breaks the run of nouns that a pull request ends.
fn pull_requests(words: &[&str], tags: &mut [&str]) {
    for index in 0..tags.len() {
        let request = words.get(index + 1).is_some_and(|word| {
            matches!(word.to_ascii_lowercase().as_str(), "request" | "requests")
        });
        if request
            && words.get(index).is_some_and(|word| *word == "pull")
            && let Some(tag) = tags.get_mut(index)
            && matches!(*tag, "VB" | "VBP")
        {
            *tag = "NN";
        }
    }
}

/// Reads a word of [`TECHNICAL_NOUNS`] as the noun it is when a noun phrase
/// holds it.
///
/// The model reads `split` in `the load split pull request` as a past tense,
/// `round` in `the mask round pull request` as an adverb, and `check` in `the
/// peer user id check` as a verb, since the corpus it learned from uses these
/// words as one other part of speech.
/// A determiner, a possessive or an adjective before the word makes it a noun,
/// unless it is a participle after an adjective, as in `so stupid thinking`.
/// So does a noun before it, when the nouns after it run up to `pull request`,
/// the head of the title of a pull request.
/// A base form after two singular nouns, with no noun after it, is a noun too,
/// as no singular noun takes it as its verb.
fn nouns_of_technical_prose(words: &[&str], tags: &mut [&str]) {
    let noun_at = |tags: &[&str], index: Option<usize>| {
        index
            .and_then(|index| tags.get(index))
            .is_some_and(|tag| matches!(*tag, "NN" | "NNS"))
    };
    for index in 1..tags.len() {
        let (Some(&word), Some(&tag)) = (words.get(index), tags.get(index)) else {
            continue;
        };
        if !TECHNICAL_NOUNS.contains(&word) || matches!(tag, "NN" | "NNS") {
            continue;
        }
        let before = index.checked_sub(1);
        let before_tag = before.and_then(|at| tags.get(at)).copied();
        let after_determiner = matches!(before_tag, Some("DT" | "PRP$" | "POS"))
            || (before_tag == Some("JJ") && tag != "VBG");
        let in_title = noun_at(tags, before) && heads_pull_request(words, tags, index);
        let ends_compound = before_tag == Some("NN")
            && noun_at(tags, index.checked_sub(2))
            && matches!(tag, "VB" | "VBP")
            && matches!(
                tags.get(index.saturating_add(1)),
                Some(&("IN" | "." | "," | "CC" | "TO"))
            );
        if (after_determiner || in_title || ends_compound)
            && let Some(slot) = tags.get_mut(index)
        {
            *slot = "NN";
        }
    }
}

/// Whether the nouns after the word at `index` run up to `pull request`.
fn heads_pull_request(words: &[&str], tags: &[&str], index: usize) -> bool {
    (1..=3).any(|offset| {
        let at = index.saturating_add(offset);
        words.get(at).is_some_and(|word| *word == "pull")
            && words.get(at.saturating_add(1)).is_some_and(|word| {
                matches!(word.to_ascii_lowercase().as_str(), "request" | "requests")
            })
            && (1..offset).all(|between| {
                tags.get(index.saturating_add(between))
                    .is_some_and(|tag| matches!(*tag, "NN" | "NNS"))
            })
    })
}

/// Reads a verb of saying or thinking as the verb it is.
///
/// The model reads `say` in `The docs say so.` as a noun, as it reads a verb
/// after a compound.
/// These forms are no nouns, so a noun tag on one is wrong, unless a
/// determiner, a number, a possessive or a preposition comes before it.
fn verbs_of_saying(words: &[&str], tags: &mut [&str]) {
    for index in 1..tags.len() {
        let (Some(word), Some(&before)) = (words.get(index), tags.get(index - 1)) else {
            continue;
        };
        if !says(word) || matches!(before, "DT" | "PRP$" | "CD" | "IN" | "POS") {
            continue;
        }
        if let Some(tag) = tags.get_mut(index)
            && (matches!(*tag, "NN" | "NNS" | "JJ") || (*tag == "UH" && NOUNS.contains(&before)))
        {
            *tag = if matches!(before, "TO" | "MD") {
                "VB"
            } else if word.to_ascii_lowercase().ends_with('s') {
                "VBZ"
            } else {
                "VBP"
            };
        }
    }
}

/// Reads the word before a verb of saying as the noun that is its subject.
///
/// The model reads `builds` in `The CI builds agree` as a verb and `report`
/// in `the report agree` as an adjective.
/// A present verb of saying has a noun for its subject, and a verb cannot
/// come before it.
fn subjects_of_verbs_of_saying(words: &[&str], tags: &mut [&str]) {
    for index in 1..tags.len() {
        let (Some(word), Some(&tag)) = (words.get(index), tags.get(index)) else {
            continue;
        };
        let (Some(next), Some(&next_tag)) = (words.get(index + 1), tags.get(index + 1)) else {
            continue;
        };
        let before = tags.get(index - 1).copied().unwrap_or_default();
        if !says(next) || !matches!(next_tag, "VB" | "VBP" | "VBZ") {
            continue;
        }
        let noun = if tag == "VBZ" && NOUNS.contains(&before) && !word_is_auxiliary(word) {
            Some("NNS")
        } else if tag == "JJ" && matches!(before, "DT" | "PRP$" | "CC" | "POS") {
            Some("NN")
        } else {
            None
        };
        if let Some(noun) = noun
            && let Some(slot) = tags.get_mut(index)
        {
            *slot = noun;
        }
    }
}

/// Reads a verb of asking or wanting, and the past tense of one of saying, as
/// the noun, the verb or the participle its place makes it.
///
/// The model reads `asks` in `A request line asks for the rows` and in `the
/// asks that wait` the same way, as a noun or as a verb by the words around
/// it alone, and `asked` in `a batch a request asked for` as a participle.
/// A clause that follows `asks` like a relative clause makes it a noun.
/// A noun phrase that starts with a determiner and is the object of a
/// preposition, of a gerund or of a first-word imperative ends in a noun, so
/// a verb tag on `asks` there is wrong, and a past tense is a participle.
/// A noun after a noun in any other place is the verb of the clause, and a
/// participle there is a past tense.
fn asking_in_noun_phrases(words: &[&str], tags: &mut [&str]) {
    for index in 1..tags.len() {
        let Some(word) = words.get(index) else {
            continue;
        };
        let new = if asks(word) {
            reading_of_ask(words, tags, index)
        } else if PAST_SAYING.contains(&word.to_ascii_lowercase().as_str()) {
            reading_of_past(words, tags, index)
        } else {
            None
        };
        if let Some(new) = new
            && let Some(slot) = tags.get_mut(index)
        {
            *slot = new;
        }
    }
}

/// The tag of the verb of asking or wanting at `index`, when the model's tag
/// is wrong.
fn reading_of_ask(words: &[&str], tags: &[&str], index: usize) -> Option<&'static str> {
    let word = words.get(index)?;
    let tag = *tags.get(index)?;
    let before = *tags.get(index.checked_sub(1)?)?;
    let plural = word.to_ascii_lowercase().ends_with('s');
    let (noun, verb) = if plural {
        ("NNS", "VBZ")
    } else {
        ("NN", "VBP")
    };
    let is_noun = matches!(tag, "NN" | "NNS");
    let is_verb = matches!(tag, "VB" | "VBP" | "VBZ");
    if !is_noun && !is_verb {
        return None;
    }
    if after_determiner(tags, index) {
        return is_verb.then_some(noun);
    }
    if !SUBJECTS.contains(&before) {
        return None;
    }
    let place = place_of(words, tags, index);
    if followed_by_clause_of_noun(words, tags, index) {
        Some(noun)
    } else if stranded_preposition(words, tags, index) {
        is_noun.then_some(verb)
    } else if is_verb && matches!(place, Place::Object | Place::VerbObject) {
        Some(noun)
    } else if is_noun && matches!(place, Place::None | Place::Subject | Place::AfterNoun) {
        Some(verb)
    } else {
        None
    }
}

/// Whether the word at `index` follows a determiner directly, or a gerund that
/// follows one, as `each ask` and `every waiting ask` do.
///
/// A verb cannot stand there.
fn after_determiner(tags: &[&str], index: usize) -> bool {
    let tag_at = |back: usize| {
        index
            .checked_sub(back)
            .and_then(|at| tags.get(at))
            .copied()
            .unwrap_or_default()
    };
    matches!(tag_at(1), "DT" | "PRP$") || (tag_at(1) == "VBG" && matches!(tag_at(2), "DT" | "PRP$"))
}

/// The tag of the past tense of a verb of saying at `index`, when the model's
/// tag is wrong.
fn reading_of_past(words: &[&str], tags: &[&str], index: usize) -> Option<&'static str> {
    let tag = *tags.get(index)?;
    let before = *tags.get(index.checked_sub(1)?)?;
    if !SUBJECTS.contains(&before) {
        return None;
    }
    let place = place_of(words, tags, index);
    let next_takes_object = words
        .get(index + 1)
        .is_some_and(|next| takes_object_of_preposition(next));
    let after_one = words
        .get(index - 1)
        .is_some_and(|word| matches!(word.to_ascii_lowercase().as_str(), "one" | "ones"));
    let participle = (place == Place::Object && next_takes_object)
        || (after_one && stranded_preposition(words, tags, index));
    if tag == "VBD" && participle {
        Some("VBN")
    } else if tag == "VBN" && matches!(place, Place::None | Place::AfterNoun) {
        Some("VBD")
    } else {
        None
    }
}

/// Whether what follows the word at `index` is what follows a noun: a relative
/// clause with or without `that`, a possessive ending, or a noun phrase joined
/// by `and`.
///
/// `The asks that wait` and `the asks it leaves` hold a clause that a verb of
/// asking, which takes an object or a `that` clause with a subject, never has.
fn followed_by_clause_of_noun(words: &[&str], tags: &[&str], index: usize) -> bool {
    let at = |offset: usize| tags.get(index + offset).copied().unwrap_or_default();
    let word_at = |offset: usize| words.get(index + offset).copied().unwrap_or_default();
    let finite = |offset: usize| {
        matches!(at(offset), "VB" | "VBP" | "VBZ" | "VBD" | "MD")
            || (matches!(at(offset), "RB" | "NNS")
                && word_at(offset).ends_with('s')
                && matches!(at(offset + 1), "," | "." | ":" | ";" | "CC"))
    };
    let relative_that = word_at(1).eq_ignore_ascii_case("that") && finite(2);
    let pronoun_clause = at(1) == "PRP" && finite(2);
    let noun_clause = matches!(at(1), "DT" | "CD") && at(2) == "NN" && finite(3);
    let possessive = at(1) == "POS"
        || (matches!(at(1), "''" | "``") && word_at(1) == "'" && matches!(at(2), "NN" | "NNS"));
    let joined = at(1) == "CC" && at(2) == "DT";
    let subject_of_verb = matches!(at(1), "VBZ" | "VBD" | "MD")
        || (at(1) == "NNS" && word_at(1).ends_with('s') && matches!(at(2), "." | "," | ":" | ";"));
    relative_that || pronoun_clause || noun_clause || possessive || joined || subject_of_verb
}

/// Whether the word at `index` is followed by a preposition with no object:
/// `what the flag asks for,` leaves the object of `for` behind, which only a
/// verb does.
fn stranded_preposition(words: &[&str], tags: &[&str], index: usize) -> bool {
    let preposition = words
        .get(index + 1)
        .is_some_and(|word| PREPOSITIONS.contains(&word.to_ascii_lowercase().as_str()));
    let after = tags.get(index + 2).copied().unwrap_or(".");
    preposition && matches!(after, "," | "." | ":" | "CC" | "IN")
}

/// Where the noun phrase that ends at a word stands in its clause.
#[derive(Clone, Copy, Eq, PartialEq)]
enum Place {
    /// No determiner starts the noun phrase.
    None,
    /// A determiner starts it at the start of a clause or after a conjunction.
    Subject,
    /// A determiner starts it, and a preposition, a gerund or an imperative
    /// before it takes it as its object.
    Object,
    /// A determiner starts it after a verb that has a subject.
    VerbObject,
    /// A determiner starts it after a verb that takes a clause with no `that`,
    /// so the noun phrase can also be the subject of that clause.
    Complement,
    /// A determiner starts it right after a noun, as an object relative clause
    /// does: `a batch a request asked for`.
    AfterNoun,
}

/// Where the noun phrase that ends at the word before `index` stands.
fn place_of(words: &[&str], tags: &[&str], index: usize) -> Place {
    let mut at = index;
    while let Some(before) = at.checked_sub(1) {
        let tag = tags.get(before).copied().unwrap_or_default();
        let determiner = matches!(tag, "DT" | "PRP$")
            && !words
                .get(before)
                .is_some_and(|word| word.eq_ignore_ascii_case("that"));
        if determiner {
            return place_of_start(words, tags, before);
        }
        let quote = tag == "''" && words.get(before) == Some(&"'");
        if !MODIFIERS.contains(&tag) && !quote {
            return Place::None;
        }
        at = before;
    }
    Place::None
}

/// Where the noun phrase that starts with the determiner at `start` stands.
fn place_of_start(words: &[&str], tags: &[&str], start: usize) -> Place {
    let Some(before) = start.checked_sub(1) else {
        return Place::Subject;
    };
    let tag = tags.get(before).copied().unwrap_or_default();
    let word = words.get(before).copied().unwrap_or_default();
    let preposition = tag == "IN" && PREPOSITIONS.contains(&word.to_ascii_lowercase().as_str());
    let coordinated = tag == "IN"
        && before >= 2
        && tags.get(before - 1) == Some(&"CC")
        && tags.get(before - 2) == Some(&"IN");
    let imperative = before == 0 && tag == "VB";
    if preposition || coordinated || tag == "TO" || tag == "VBG" || imperative {
        Place::Object
    } else if VERBS.contains(&tag) {
        if COMPLEMENT_STEMS
            .iter()
            .any(|stem| word.to_ascii_lowercase().starts_with(stem))
        {
            Place::Complement
        } else {
            Place::VerbObject
        }
    } else if SUBJECTS.contains(&tag) {
        Place::AfterNoun
    } else {
        Place::Subject
    }
}

/// Whether `word` is a preposition that a participle of saying or asking takes
/// as its complement, as `told of` does.
/// `to` starts an infinitive after `wanted` and `said`, so it is left out.
fn takes_object_of_preposition(word: &str) -> bool {
    let word = word.to_ascii_lowercase();
    word != "to" && PREPOSITIONS.contains(&word.as_str())
}

/// Whether `word` is a verb of saying or thinking that is no noun.
fn says(word: &str) -> bool {
    NEVER_NOUNS.contains(&word.to_ascii_lowercase().as_str())
}

/// Whether `word` is a verb of asking or wanting that a noun phrase can also
/// name.
fn asks(word: &str) -> bool {
    ALSO_NOUNS.contains(&word.to_ascii_lowercase().as_str())
}

/// Whether `word` is an auxiliary that a verb follows without a noun.
fn word_is_auxiliary(word: &str) -> bool {
    matches!(word.to_ascii_lowercase().as_str(), "does" | "has" | "is")
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
/// object or a phrase only when no other word of the sentence is a verb.
/// A plural noun that this rule turned into a verb counts as one.
/// A compound noun with a plural head, such as `layout rules of the widgets`,
/// reads the same.
fn verbs_after_noun_phrases(words: &[&str], tags: &mut [&str]) {
    let mut others = tags.iter().filter(|tag| VERBS.contains(tag)).count();
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
            others = others.saturating_add(1);
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
