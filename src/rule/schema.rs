//! What a rule file may hold, one type per check.
//!
//! Each type refuses a field its check does not define, and validation then
//! refuses a field of the wrong kind, a `level` Vale does not define, and the
//! Vale features hakkiri does not run.

use garde::Validate;
use serde::Deserialize;

use super::field::{Count, Flag, Pairs, Text, Texts, valid};

/// The check types of Vale 3.26.0 that hakkiri runs.
pub(super) const RUNNABLE: [&str; 4] = ["existence", "substitution", "occurrence", "metric"];

/// The check types of Vale 3.26.0 that a later change adds.
pub(super) const PLANNED: [&str; 7] = [
    "capitalization",
    "consistency",
    "conditional",
    "repetition",
    "sequence",
    "spelling",
    "readability",
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

#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "garde calls a custom rule with a reference to its context."
)]
fn extends_ok(value: &Text, ctx: &()) -> garde::Result {
    valid(value, ctx)?;
    let name = value.value();
    if RUNNABLE.contains(&name) {
        Ok(())
    } else if name == "script" {
        Err(garde::Error::new(
            "`script` rules are not supported: hakkiri runs no scripts",
        ))
    } else if PLANNED.contains(&name) {
        Err(garde::Error::new(format!(
            "`{name}` is a check type hakkiri does not run yet"
        )))
    } else {
        Err(garde::Error::new(format!(
            "`{name}` is not a check type: use one of {}",
            [RUNNABLE.as_slice(), PLANNED.as_slice()]
                .concat()
                .join(", ")
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

#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "garde calls a custom rule with a reference to its context."
)]
fn scope_ok(value: &Texts, ctx: &()) -> garde::Result {
    valid(value, ctx)?;
    match value
        .values()
        .iter()
        .find(|scope| scope.trim_start_matches('~').starts_with("doc("))
    {
        Some(scope) => Err(garde::Error::new(format!(
            "`{scope}` is a doc selector, which hakkiri does not support"
        ))),
        None => Ok(()),
    }
}

#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "garde calls a custom rule with a reference to its context."
)]
fn affix_ok(value: &Text, ctx: &()) -> garde::Result {
    valid(value, ctx)?;
    if value.value().is_empty() {
        Ok(())
    } else {
        Err(garde::Error::new(
            "`affix` is not supported yet: no affix files load",
        ))
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

/// A metric without `tokens` or `raw` counts words only. One that lists
/// `tokens` holding nothing counts nothing, which is a mistake.
#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "garde calls a custom rule with a reference to its context."
)]
fn metric_ok(fields: &MetricFields, _: &()) -> garde::Result {
    if fields.tokens.values().is_empty() || has_pattern(&fields.tokens, &fields.raw) {
        Ok(())
    } else {
        Err(garde::Error::new(NO_PATTERN))
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
            #[garde(custom(scope_ok))]
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
        #[garde(custom(affix_ok))]
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
        #[garde(custom(affix_ok))]
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
