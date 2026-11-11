//! What a rule file may hold, one type per check.
//!
//! Each type rejects a field its check does not define, and validation then
//! refuses a field of the wrong kind, a `level` Vale does not define, and the
//! Vale features hakkiri does not run.

use garde::Validate;
use serde::Deserialize;

use super::cases::{CaseFields, unique};
use super::field::{Count, Flag, Number, Pairs, Text, Texts, valid};

/// The check types of Vale 3.26.0 that hakkiri runs.
pub(super) const RUNNABLE: [&str; 11] = [
    "existence",
    "substitution",
    "occurrence",
    "metric",
    "consistency",
    "conditional",
    "repetition",
    "readability",
    "capitalization",
    "sequence",
    "spelling",
];

/// The levels of an alert.
pub(super) const LEVELS: [&str; 3] = ["suggestion", "warning", "error"];

/// The `extends` of a rule file, read before the rest so the rest is read as
/// the type it names.
#[derive(Debug, Deserialize, Validate)]
#[garde(allow_unvalidated)]
pub(super) struct Probe {
    #[garde(custom(extends_ok))]
    pub(super) extends: Text,
}

/// The `scope` of a rule file, read apart to tell a doc selector from the
/// rest.
#[derive(Debug, Deserialize, Validate)]
#[garde(allow_unvalidated)]
pub(super) struct ScopeProbe {
    #[serde(default)]
    pub(super) scope: Texts,
}

#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "garde calls a custom rule with a reference to its context."
)]
fn extends_ok(value: &Text, ctx: &()) -> garde::Result {
    valid(value, ctx)?;
    let name = value.value();
    // A rule name is an inheritance, read before the rule is. A `script` is
    // valid Vale that hakkiri cannot run, which the loader reports apart.
    if RUNNABLE.contains(&name) || name == "script" || super::inherit::is_rule_ref(name) {
        Ok(())
    } else {
        Err(garde::Error::new(format!(
            "`{name}` is not a check type: use one of {}",
            RUNNABLE.join(", ")
        )))
    }
}

#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "garde calls a custom rule with a reference to its context."
)]
fn level_ok(value: &Text, ctx: &()) -> garde::Result {
    valid(value, ctx)?;
    let name = value.value();
    if name.is_empty() || LEVELS.contains(&name) {
        Ok(())
    } else {
        Err(garde::Error::new(format!(
            "`{name}` is not a level: use one of {}",
            LEVELS.join(", ")
        )))
    }
}

/// Whether `tokens` and `raw` hold a pattern to match: a pattern of nothing
/// matches at every position.
fn has_pattern(tokens: &Texts, raw: &Texts) -> bool {
    raw.values().iter().any(|part| !part.is_empty())
        || tokens.values().iter().any(|token| !token.trim().is_empty())
}

const NO_PATTERN: &str = "`tokens` and `raw` hold nothing to match, so the pattern would \
     match at every position: list a token or a raw pattern";

#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "garde calls a custom rule with a reference to its context."
)]
fn existence_ok(fields: &ExistenceFields, _: &()) -> garde::Result {
    if has_pattern(&fields.tokens, &fields.raw) {
        Ok(())
    } else {
        Err(garde::Error::new(NO_PATTERN))
    }
}

/// A metric that lists neither `tokens` nor `raw` counts words only. One
/// that lists either of them, holding nothing, counts nothing, which is a
/// mistake.
#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "garde calls a custom rule with a reference to its context."
)]
fn metric_ok(fields: &MetricFields, _: &()) -> garde::Result {
    let listed = !fields.tokens.values().is_empty() || !fields.raw.values().is_empty();
    if !listed || has_pattern(&fields.tokens, &fields.raw) {
        Ok(())
    } else {
        Err(garde::Error::new(NO_PATTERN))
    }
}

#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "garde calls a custom rule with a reference to its context."
)]
fn consistency_ok(fields: &ConsistencyFields, _: &()) -> garde::Result {
    let entries = fields.either.entries();
    if entries.is_empty() {
        return Err(garde::Error::new(
            "`either` holds no pair: list at least one `form: other form`",
        ));
    }
    if entries
        .iter()
        .any(|(first, second)| first.trim().is_empty() || second.trim().is_empty())
    {
        return Err(garde::Error::new(
            "`either` has a form with no text, which would match at every position",
        ));
    }
    Ok(())
}

/// Whether `tokens` holds a pattern to match.
fn has_tokens(tokens: &Texts) -> bool {
    tokens.values().iter().any(|token| !token.trim().is_empty())
}

#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "garde calls a custom rule with a reference to its context."
)]
fn repetition_ok(fields: &RepetitionFields, _: &()) -> garde::Result {
    if has_tokens(&fields.tokens) {
        Ok(())
    } else {
        Err(garde::Error::new(
            "`tokens` holds nothing to match, so the pattern would match at every position: \
             list a token",
        ))
    }
}

/// The action of a rule: what a fix would do. It loads and changes nothing.
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
#[garde(allow_unvalidated)]
pub(super) struct ActionFields {
    #[serde(default)]
    #[garde(custom(valid))]
    pub(super) name: Text,
    #[serde(default)]
    #[garde(custom(valid))]
    pub(super) params: Texts,
}

/// Defines the fields of one rule file type: the fields every check takes,
/// then the ones of its own. `check(f)` adds the rule `f`, which reads the
/// whole type.
macro_rules! rule_fields {
    (
        $(#[$meta:meta])*
        $name:ident $(check($check:ident))? {
            $(
                $(#[$field_meta:meta])*
                $field:ident: $ty:ty,
            )*
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Deserialize, Validate)]
        #[serde(deny_unknown_fields)]
        #[garde(allow_unvalidated)]
        $(#[garde(custom($check))])?
        pub(super) struct $name {
            #[garde(custom(valid))]
            pub(super) extends: Text,
            #[garde(custom(valid))]
            pub(super) message: Text,
            #[serde(default)]
            #[garde(custom(level_ok))]
            pub(super) level: Text,
            #[serde(default)]
            #[garde(custom(valid))]
            pub(super) scope: Texts,
            #[serde(default)]
            #[garde(custom(valid))]
            pub(super) link: Text,
            #[serde(default)]
            #[garde(custom(valid))]
            pub(super) description: Text,
            #[serde(default)]
            #[garde(custom(valid))]
            pub(super) limit: Count,
            #[serde(default)]
            #[garde(dive)]
            pub(super) action: Option<ActionFields>,
            #[serde(default)]
            #[garde(custom(valid))]
            pub(super) matchcase: Flag,
            #[serde(default)]
            #[garde(dive, custom(unique))]
            pub(super) tests: Vec<CaseFields>,
            $(
                $(#[$field_meta])*
                pub(super) $field: $ty,
            )*
        }
    };
}

rule_fields! {
    /// An `existence` rule.
    ExistenceFields check(existence_ok) {
        #[serde(default)]
        #[garde(custom(valid))]
        raw: Texts,
        #[serde(default)]
        #[garde(custom(valid))]
        tokens: Texts,
        #[serde(default)]
        #[garde(custom(valid))]
        affix: Text,
        #[serde(default)]
        #[garde(custom(valid))]
        exceptions: Texts,
        #[serde(default)]
        #[garde(custom(valid))]
        append: Flag,
        #[serde(default)]
        #[garde(custom(valid))]
        ignorecase: Flag,
        #[serde(default)]
        #[garde(custom(valid))]
        nonword: Flag,
        #[serde(default)]
        #[garde(custom(valid))]
        vocab: Flag,
    }
}

rule_fields! {
    /// A `substitution` rule.
    SubstitutionFields {
        #[garde(custom(valid))]
        swap: Pairs,
        #[serde(default)]
        #[garde(custom(valid))]
        exceptions: Texts,
        #[serde(default)]
        #[garde(custom(valid))]
        affix: Text,
        #[serde(default)]
        #[garde(custom(valid))]
        ignorecase: Flag,
        #[serde(default)]
        #[garde(custom(valid))]
        nonword: Flag,
        #[serde(default)]
        #[garde(custom(valid))]
        vocab: Flag,
        #[serde(default)]
        #[garde(custom(valid))]
        capitalize: Flag,
    }
}

rule_fields! {
    /// An `occurrence` rule.
    OccurrenceFields {
        #[garde(custom(valid))]
        token: Text,
        #[serde(default)]
        #[garde(custom(valid))]
        max: Count,
        #[serde(default)]
        #[garde(custom(valid))]
        min: Count,
        #[serde(default)]
        #[garde(custom(valid))]
        ignorecase: Flag,
    }
}

rule_fields! {
    /// A `metric` rule.
    MetricFields check(metric_ok) {
        #[garde(custom(valid))]
        formula: Text,
        #[garde(custom(valid))]
        condition: Text,
        #[serde(default)]
        #[garde(custom(valid))]
        tokens: Texts,
        #[serde(default)]
        #[garde(custom(valid))]
        raw: Texts,
        #[serde(default)]
        #[garde(custom(valid))]
        append: Flag,
        #[serde(default)]
        #[garde(custom(valid))]
        exceptions: Texts,
        #[serde(default)]
        #[garde(custom(valid))]
        ignorecase: Flag,
        #[serde(default)]
        #[garde(custom(valid))]
        nonword: Flag,
    }
}

rule_fields! {
    /// A `readability` rule.
    ReadabilityFields {
        #[serde(default)]
        #[garde(custom(valid))]
        metrics: Texts,
        #[serde(default)]
        #[garde(custom(valid))]
        grade: Number,
    }
}

rule_fields! {
    /// A `repetition` rule.
    RepetitionFields check(repetition_ok) {
        #[serde(default)]
        #[garde(custom(valid))]
        tokens: Texts,
        #[serde(default)]
        #[garde(custom(valid))]
        max: Count,
        #[serde(default)]
        #[garde(custom(valid))]
        ignorecase: Flag,
        #[serde(default)]
        #[garde(custom(valid))]
        alpha: Flag,
        #[serde(default)]
        #[garde(custom(valid))]
        vocab: Flag,
        #[serde(default)]
        #[garde(custom(valid))]
        exceptions: Texts,
    }
}

rule_fields! {
    /// A `consistency` rule.
    ConsistencyFields check(consistency_ok) {
        #[serde(default)]
        #[garde(custom(valid))]
        either: Pairs,
        #[serde(default)]
        #[garde(custom(valid))]
        nonword: Flag,
        #[serde(default)]
        #[garde(custom(valid))]
        ignorecase: Flag,
    }
}

rule_fields! {
    /// A `conditional` rule.
    ConditionalFields {
        #[garde(custom(valid))]
        first: Text,
        #[garde(custom(valid))]
        second: Text,
        #[serde(default, rename = "in")]
        #[garde(custom(valid))]
        within: Text,
        #[serde(default)]
        #[garde(custom(valid))]
        exceptions: Texts,
        #[serde(default)]
        #[garde(custom(valid))]
        ignorecase: Flag,
        #[serde(default)]
        #[garde(custom(valid))]
        vocab: Flag,
    }
}

rule_fields! {
    /// A `capitalization` rule.
    CapitalizationFields {
        #[serde(rename = "match")]
        #[garde(custom(valid))]
        pattern: Text,
        #[serde(default)]
        #[garde(custom(valid))]
        style: Text,
        #[serde(default)]
        #[garde(custom(valid))]
        exceptions: Texts,
        #[serde(default)]
        #[garde(custom(valid))]
        indicators: Texts,
        #[serde(default)]
        #[garde(custom(valid))]
        threshold: Number,
        #[serde(default)]
        #[garde(custom(valid))]
        vocab: Flag,
        #[serde(default)]
        #[garde(custom(valid))]
        prefix: Text,
    }
}

/// One token of a `sequence` rule.
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
#[garde(allow_unvalidated)]
pub(super) struct TokenFields {
    #[serde(default)]
    #[garde(custom(valid))]
    pub(super) pattern: Text,
    #[serde(default)]
    #[garde(custom(valid))]
    pub(super) tag: Text,
    #[serde(default)]
    #[garde(custom(valid))]
    pub(super) upos: Text,
    #[serde(default)]
    #[garde(custom(valid))]
    pub(super) skip: Count,
    #[serde(default)]
    #[garde(custom(valid))]
    pub(super) min: Count,
    #[serde(default)]
    #[garde(custom(valid))]
    pub(super) target: Flag,
    #[serde(default)]
    #[garde(custom(valid))]
    pub(super) negate: Flag,
}

rule_fields! {
    /// A `sequence` rule.
    SequenceFields {
        #[garde(dive)]
        tokens: Vec<TokenFields>,
        #[serde(default)]
        #[garde(custom(valid))]
        exceptions: Texts,
        #[serde(default)]
        #[garde(custom(valid))]
        ignorecase: Flag,
        #[serde(default)]
        #[garde(custom(valid))]
        model: Text,
    }
}

rule_fields! {
    /// A `spelling` rule.
    SpellingFields {
        #[serde(default)]
        #[garde(custom(valid))]
        filters: Texts,
        #[serde(default)]
        #[garde(custom(valid))]
        ignore: Texts,
        #[serde(default)]
        #[garde(custom(valid))]
        exceptions: Texts,
        #[serde(default)]
        #[garde(custom(valid))]
        dictionaries: Texts,
        #[serde(default)]
        #[garde(custom(valid))]
        aff: Text,
        #[serde(default)]
        #[garde(custom(valid))]
        dic: Text,
        #[serde(default)]
        #[garde(custom(valid))]
        dicpath: Text,
        #[serde(default)]
        #[garde(custom(valid))]
        threshold: Number,
        #[serde(default)]
        #[garde(custom(valid))]
        custom: Flag,
        #[serde(default)]
        #[garde(custom(valid))]
        append: Flag,
        #[serde(default)]
        #[garde(custom(valid))]
        split: Flag,
    }
}
