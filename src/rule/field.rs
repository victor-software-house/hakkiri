//! The value types a rule file's fields read into.
//!
//! A field that holds the wrong kind of value does not fail the parse.
//! It reads with a problem, and validation refuses it under the field's name
//! and line, which a parse error does not carry.

use std::fmt;

use serde::Deserialize;
use serde::de::{self, Deserializer, IgnoredAny, MapAccess, SeqAccess, Visitor};

/// A value that can read with a problem.
pub(super) trait Checked {
    /// What is wrong with the value, if anything.
    fn problem(&self) -> Option<&str>;
}

/// The garde rule every field of this module takes: a value that read with a
/// problem is refused.
#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "garde calls a custom rule with a reference to its context."
)]
pub(super) fn valid<T: Checked>(value: &T, _: &()) -> garde::Result {
    match value.problem() {
        Some(problem) => Err(garde::Error::new(problem)),
        None => Ok(()),
    }
}

/// The text of a scalar, as a rule file wrote it.
///
/// A number or a boolean reads as its text, so `message: 5` is `5`.
#[derive(Debug, Default)]
pub(super) struct Text {
    value: String,
    problem: Option<String>,
}

impl Text {
    /// The text, empty when the field read with a problem.
    pub(super) fn value(&self) -> &str {
        &self.value
    }

    fn found(found: &str) -> Self {
        Self {
            value: String::new(),
            problem: Some(format!("must be text, found {found}")),
        }
    }
}

impl Checked for Text {
    fn problem(&self) -> Option<&str> {
        self.problem.as_deref()
    }
}

impl<'de> Deserialize<'de> for Text {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct TextVisitor;

        impl<'de> Visitor<'de> for TextVisitor {
            type Value = Text;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("text")
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<Text, E> {
                Ok(Text {
                    value: value.to_owned(),
                    problem: None,
                })
            }

            fn visit_bool<E: de::Error>(self, value: bool) -> Result<Text, E> {
                self.visit_str(&value.to_string())
            }

            fn visit_i64<E: de::Error>(self, value: i64) -> Result<Text, E> {
                self.visit_str(&value.to_string())
            }

            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Text, E> {
                self.visit_str(&value.to_string())
            }

            fn visit_f64<E: de::Error>(self, value: f64) -> Result<Text, E> {
                self.visit_str(&value.to_string())
            }

            fn visit_unit<E: de::Error>(self) -> Result<Text, E> {
                Ok(Text::default())
            }

            fn visit_none<E: de::Error>(self) -> Result<Text, E> {
                Ok(Text::default())
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Text, A::Error> {
                while seq.next_element::<IgnoredAny>()?.is_some() {}
                Ok(Text::found("a list"))
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Text, A::Error> {
                while map.next_entry::<IgnoredAny, IgnoredAny>()?.is_some() {}
                Ok(Text::found("a mapping"))
            }
        }

        deserializer.deserialize_any(TextVisitor)
    }
}

/// A list of texts. One text reads as a list of one, as Vale reads it.
#[derive(Debug, Default)]
pub(super) struct Texts {
    values: Vec<String>,
    problem: Option<String>,
}

impl Texts {
    /// The texts, empty when the field read with a problem.
    pub(super) fn values(&self) -> &[String] {
        &self.values
    }
}

impl Checked for Texts {
    fn problem(&self) -> Option<&str> {
        self.problem.as_deref()
    }
}

impl<'de> Deserialize<'de> for Texts {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct TextsVisitor;

        impl<'de> Visitor<'de> for TextsVisitor {
            type Value = Texts;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a list of text")
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Texts, A::Error> {
                let mut values = Vec::new();
                let mut problem = None;
                while let Some(text) = seq.next_element::<Text>()? {
                    match text.problem {
                        Some(found) => problem = problem.or(Some(found)),
                        None => values.push(text.value),
                    }
                }
                Ok(if problem.is_some() {
                    Texts {
                        values: Vec::new(),
                        problem,
                    }
                } else {
                    Texts {
                        values,
                        problem: None,
                    }
                })
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Texts, A::Error> {
                while map.next_entry::<IgnoredAny, IgnoredAny>()?.is_some() {}
                Ok(Texts {
                    values: Vec::new(),
                    problem: Some("must be a list of text, found a mapping".to_owned()),
                })
            }

            fn visit_unit<E: de::Error>(self) -> Result<Texts, E> {
                Ok(Texts::default())
            }

            fn visit_none<E: de::Error>(self) -> Result<Texts, E> {
                Ok(Texts::default())
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<Texts, E> {
                Ok(Texts {
                    values: vec![value.to_owned()],
                    problem: None,
                })
            }

            fn visit_bool<E: de::Error>(self, value: bool) -> Result<Texts, E> {
                self.visit_str(&value.to_string())
            }

            fn visit_i64<E: de::Error>(self, value: i64) -> Result<Texts, E> {
                self.visit_str(&value.to_string())
            }

            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Texts, E> {
                self.visit_str(&value.to_string())
            }

            fn visit_f64<E: de::Error>(self, value: f64) -> Result<Texts, E> {
                self.visit_str(&value.to_string())
            }
        }

        deserializer.deserialize_any(TextsVisitor)
    }
}

/// A whole number of 0 or more. A number in quotes reads as the number.
#[derive(Debug, Default)]
pub(super) struct Count {
    value: usize,
    problem: Option<String>,
}

impl Count {
    /// The number, 0 when the field read with a problem.
    pub(super) const fn value(&self) -> usize {
        self.value
    }

    fn found(found: &str) -> Self {
        Self {
            value: 0,
            problem: Some(format!(
                "must be a whole number of 0 or more, found {found}"
            )),
        }
    }
}

impl Checked for Count {
    fn problem(&self) -> Option<&str> {
        self.problem.as_deref()
    }
}

impl<'de> Deserialize<'de> for Count {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct CountVisitor;

        impl<'de> Visitor<'de> for CountVisitor {
            type Value = Count;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a whole number")
            }

            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Count, E> {
                Ok(usize::try_from(value).map_or_else(
                    |_| Count::found(&format!("`{value}`")),
                    |value| Count {
                        value,
                        problem: None,
                    },
                ))
            }

            fn visit_i64<E: de::Error>(self, value: i64) -> Result<Count, E> {
                Ok(u64::try_from(value).map_or_else(
                    |_| Count::found(&format!("`{value}`")),
                    |value| self.visit_u64::<E>(value).unwrap_or_default(),
                ))
            }

            fn visit_f64<E: de::Error>(self, value: f64) -> Result<Count, E> {
                Ok(Count::found(&format!("`{value}`")))
            }

            fn visit_bool<E: de::Error>(self, value: bool) -> Result<Count, E> {
                Ok(Count::found(&format!("`{value}`")))
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<Count, E> {
                Ok(value.trim().parse::<usize>().map_or_else(
                    |_| Count::found(&format!("`{value}`")),
                    |value| Count {
                        value,
                        problem: None,
                    },
                ))
            }

            fn visit_unit<E: de::Error>(self) -> Result<Count, E> {
                Ok(Count::default())
            }

            fn visit_none<E: de::Error>(self) -> Result<Count, E> {
                Ok(Count::default())
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Count, A::Error> {
                while seq.next_element::<IgnoredAny>()?.is_some() {}
                Ok(Count::found("a list"))
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Count, A::Error> {
                while map.next_entry::<IgnoredAny, IgnoredAny>()?.is_some() {}
                Ok(Count::found("a mapping"))
            }
        }

        deserializer.deserialize_any(CountVisitor)
    }
}

/// A number, whole or with decimals, and negative when it starts with a minus.
/// A number in quotes reads as the number.
#[derive(Debug, Default)]
pub(super) struct Number {
    value: f64,
    problem: Option<String>,
}

impl Number {
    /// The number, 0 when the field read with a problem.
    pub(super) const fn value(&self) -> f64 {
        self.value
    }

    fn found(found: &str) -> Self {
        Self {
            value: 0.0,
            problem: Some(format!("must be a number, found {found}")),
        }
    }

    fn of(value: f64) -> Self {
        if value.is_finite() {
            Self {
                value,
                problem: None,
            }
        } else {
            Self::found(&format!("`{value}`"))
        }
    }
}

impl Checked for Number {
    fn problem(&self) -> Option<&str> {
        self.problem.as_deref()
    }
}

impl<'de> Deserialize<'de> for Number {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct NumberVisitor;

        impl<'de> Visitor<'de> for NumberVisitor {
            type Value = Number;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a number")
            }

            fn visit_f64<E: de::Error>(self, value: f64) -> Result<Number, E> {
                Ok(Number::of(value))
            }

            fn visit_i64<E: de::Error>(self, value: i64) -> Result<Number, E> {
                Ok(i32::try_from(value).map_or_else(
                    |_| Number::found(&format!("`{value}`")),
                    |value| Number::of(f64::from(value)),
                ))
            }

            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Number, E> {
                Ok(u32::try_from(value).map_or_else(
                    |_| Number::found(&format!("`{value}`")),
                    |value| Number::of(f64::from(value)),
                ))
            }

            fn visit_bool<E: de::Error>(self, value: bool) -> Result<Number, E> {
                Ok(Number::found(&format!("`{value}`")))
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<Number, E> {
                Ok(value
                    .trim()
                    .parse::<f64>()
                    .map_or_else(|_| Number::found(&format!("`{value}`")), Number::of))
            }

            fn visit_unit<E: de::Error>(self) -> Result<Number, E> {
                Ok(Number::default())
            }

            fn visit_none<E: de::Error>(self) -> Result<Number, E> {
                Ok(Number::default())
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Number, A::Error> {
                while seq.next_element::<IgnoredAny>()?.is_some() {}
                Ok(Number::found("a list"))
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Number, A::Error> {
                while map.next_entry::<IgnoredAny, IgnoredAny>()?.is_some() {}
                Ok(Number::found("a mapping"))
            }
        }

        deserializer.deserialize_any(NumberVisitor)
    }
}

/// `true` or `false`. The words in quotes read the same.
#[derive(Debug, Default)]
pub(super) struct Flag {
    value: bool,
    problem: Option<String>,
}

impl Flag {
    /// The flag, `false` when the field read with a problem.
    pub(super) const fn value(&self) -> bool {
        self.value
    }

    /// A flag that is `true`, for a field that defaults to it when absent.
    pub(super) const fn on() -> Self {
        Self {
            value: true,
            problem: None,
        }
    }

    fn found(found: &str) -> Self {
        Self {
            value: false,
            problem: Some(format!("must be true or false, found {found}")),
        }
    }
}

impl Checked for Flag {
    fn problem(&self) -> Option<&str> {
        self.problem.as_deref()
    }
}

impl<'de> Deserialize<'de> for Flag {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct FlagVisitor;

        impl<'de> Visitor<'de> for FlagVisitor {
            type Value = Flag;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("true or false")
            }

            fn visit_bool<E: de::Error>(self, value: bool) -> Result<Flag, E> {
                Ok(Flag {
                    value,
                    problem: None,
                })
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<Flag, E> {
                Ok(match value.trim().to_ascii_lowercase().as_str() {
                    "true" => Flag {
                        value: true,
                        problem: None,
                    },
                    "false" => Flag::default(),
                    _ => Flag::found(&format!("`{value}`")),
                })
            }

            fn visit_i64<E: de::Error>(self, value: i64) -> Result<Flag, E> {
                Ok(Flag::found(&format!("`{value}`")))
            }

            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Flag, E> {
                Ok(Flag::found(&format!("`{value}`")))
            }

            fn visit_f64<E: de::Error>(self, value: f64) -> Result<Flag, E> {
                Ok(Flag::found(&format!("`{value}`")))
            }

            fn visit_unit<E: de::Error>(self) -> Result<Flag, E> {
                Ok(Flag::default())
            }

            fn visit_none<E: de::Error>(self) -> Result<Flag, E> {
                Ok(Flag::default())
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Flag, A::Error> {
                while seq.next_element::<IgnoredAny>()?.is_some() {}
                Ok(Flag::found("a list"))
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Flag, A::Error> {
                while map.next_entry::<IgnoredAny, IgnoredAny>()?.is_some() {}
                Ok(Flag::found("a mapping"))
            }
        }

        deserializer.deserialize_any(FlagVisitor)
    }
}

/// A mapping of text to text, in the order the file wrote it.
#[derive(Debug, Default)]
pub(super) struct Pairs {
    entries: Vec<(String, String)>,
    problem: Option<String>,
}

impl Pairs {
    /// The entries, empty when the field read with a problem.
    pub(super) fn entries(&self) -> &[(String, String)] {
        &self.entries
    }
}

impl Checked for Pairs {
    fn problem(&self) -> Option<&str> {
        self.problem.as_deref()
    }
}

impl<'de> Deserialize<'de> for Pairs {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct PairsVisitor;

        impl PairsVisitor {
            fn found(found: &str) -> Pairs {
                Pairs {
                    entries: Vec::new(),
                    problem: Some(format!("must be a mapping of text to text, found {found}")),
                }
            }
        }

        impl<'de> Visitor<'de> for PairsVisitor {
            type Value = Pairs;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a mapping of text to text")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Pairs, A::Error> {
                let mut entries = Vec::new();
                let mut problem = None;
                while let Some((key, value)) = map.next_entry::<Text, Text>()? {
                    match key.problem.or(value.problem) {
                        Some(found) => problem = problem.or(Some(found)),
                        None => entries.push((key.value, value.value)),
                    }
                }
                Ok(match problem {
                    Some(found) => Pairs {
                        entries: Vec::new(),
                        problem: Some(found),
                    },
                    None => Pairs {
                        entries,
                        problem: None,
                    },
                })
            }

            fn visit_unit<E: de::Error>(self) -> Result<Pairs, E> {
                Ok(Pairs::default())
            }

            fn visit_none<E: de::Error>(self) -> Result<Pairs, E> {
                Ok(Pairs::default())
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<Pairs, E> {
                Ok(Self::found(&format!("`{value}`")))
            }

            fn visit_bool<E: de::Error>(self, value: bool) -> Result<Pairs, E> {
                Ok(Self::found(&format!("`{value}`")))
            }

            fn visit_i64<E: de::Error>(self, value: i64) -> Result<Pairs, E> {
                Ok(Self::found(&format!("`{value}`")))
            }

            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Pairs, E> {
                Ok(Self::found(&format!("`{value}`")))
            }

            fn visit_f64<E: de::Error>(self, value: f64) -> Result<Pairs, E> {
                Ok(Self::found(&format!("`{value}`")))
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Pairs, A::Error> {
                while seq.next_element::<IgnoredAny>()?.is_some() {}
                Ok(Self::found("a list"))
            }
        }

        deserializer.deserialize_any(PairsVisitor)
    }
}
