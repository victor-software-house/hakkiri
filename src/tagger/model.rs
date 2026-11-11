//! The model file: the flat format of prose's averaged perceptron.
//!
//! The file starts with the magic `PROSETAG` and a version of 1 or 2.
//! Every number is little-endian.
//! The tables follow in this order:
//!
//! | Table | Contents |
//! |:--|:--|
//! | classes | A count, then each class name as a `u16` length and bytes. |
//! | sizes | Feature count, pair count and blob length, as three `u32`. |
//! | blob | Every feature name, sorted bytewise and concatenated. |
//! | feature offsets | One `u32` more than there are features, into the blob. |
//! | pair offsets | One `u32` more than there are features, into the pairs. |
//! | pairs | A class id (`u8`) for each pair, then a weight (`f32`) for each. |
//! | tag map | A count, then a word and a class id for each word with one tag. |
//! | metadata | Version 2 only: a count, then a key and a value for each. |

use std::collections::HashMap;
use std::{error, fmt};

/// The first bytes of every model file.
const MAGIC: &[u8] = b"PROSETAG";

/// The newest format version this reader knows.
const VERSION: u32 = 2;

/// The most classes a model may name.
const MAX_CLASSES: usize = 256;

/// Why a model file could not be read.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FormatError {
    /// The file does not start with the magic bytes.
    Magic,
    /// The version is one this reader does not know.
    Version(u32),
    /// The file ends before a table does.
    Truncated {
        /// The byte offset where the read began.
        at: usize,
        /// The bytes the table needed.
        wanted: usize,
    },
    /// A name in the file is not UTF-8.
    Text {
        /// The byte offset of the name.
        at: usize,
    },
    /// The tables conflict with each other.
    Invalid(&'static str),
    /// The model is not the English Penn Treebank model this tagger reads.
    Unsupported(&'static str),
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Magic => write!(f, "the file does not start with `PROSETAG`"),
            Self::Version(version) => write!(
                f,
                "format version {version} is not one this reader knows (1 to {VERSION})"
            ),
            Self::Truncated { at, wanted } => {
                write!(
                    f,
                    "the file ends early: {wanted} bytes wanted at offset {at}"
                )
            }
            Self::Text { at } => write!(f, "the name at offset {at} is not UTF-8"),
            Self::Invalid(problem) | Self::Unsupported(problem) => f.write_str(problem),
        }
    }
}

impl error::Error for FormatError {}

/// The tables of a model, as read from a file.
#[derive(Debug)]
pub(super) struct Model {
    /// The name of each class, by id.
    classes: Vec<String>,
    /// Every feature name, concatenated.
    blob: Vec<u8>,
    /// Where each feature starts in the blob, and a last offset at its end.
    feature_offsets: Vec<usize>,
    /// Where the pairs of each feature start, and a last offset at their end.
    pair_offsets: Vec<usize>,
    /// The class of each pair.
    class_ids: Vec<u8>,
    /// The weight of each pair.
    weights: Vec<f32>,
    /// The class of each word that has only one.
    tag_map: HashMap<String, u8>,
    /// The metadata of a version 2 file.
    meta: HashMap<String, String>,
}

/// A cursor over the bytes of a file.
struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, wanted: usize) -> Result<&'a [u8], FormatError> {
        let truncated = FormatError::Truncated {
            at: self.at,
            wanted,
        };
        let end = self.at.checked_add(wanted).ok_or(truncated.clone())?;
        let slice = self.bytes.get(self.at..end).ok_or(truncated)?;
        self.at = end;
        Ok(slice)
    }

    /// The next `N` bytes as an array.
    fn array<const N: usize>(&mut self) -> Result<[u8; N], FormatError> {
        self.take(N)?
            .first_chunk::<N>()
            .copied()
            .ok_or(FormatError::Invalid("a short read"))
    }

    fn u8(&mut self) -> Result<u8, FormatError> {
        Ok(u8::from_le_bytes(self.array::<1>()?))
    }

    fn u16(&mut self) -> Result<u16, FormatError> {
        Ok(u16::from_le_bytes(self.array::<2>()?))
    }

    fn u32(&mut self) -> Result<u32, FormatError> {
        Ok(u32::from_le_bytes(self.array::<4>()?))
    }

    /// A `u32` that counts or locates something, as a `usize`.
    ///
    /// A value over `usize` reads as the largest `usize`, which no table holds.
    fn size(&mut self) -> Result<usize, FormatError> {
        Ok(usize::try_from(self.u32()?).unwrap_or(usize::MAX))
    }

    fn string(&mut self) -> Result<String, FormatError> {
        let at = self.at;
        let length = usize::from(self.u16()?);
        String::from_utf8(self.take(length)?.to_vec()).or(Err(FormatError::Text { at }))
    }

    /// The bytes of a table of `count` four-byte values.
    fn table(&mut self, count: usize) -> Result<&'a [u8], FormatError> {
        let length = count.checked_mul(4).ok_or(FormatError::Invalid(
            "a table longer than the address space",
        ))?;
        self.take(length)
    }

    fn sizes(&mut self, count: usize) -> Result<Vec<usize>, FormatError> {
        let (values, _) = self.table(count)?.as_chunks::<4>();
        Ok(values
            .iter()
            .map(|value| usize::try_from(u32::from_le_bytes(*value)).unwrap_or(usize::MAX))
            .collect())
    }

    fn weights(&mut self, count: usize) -> Result<Vec<f32>, FormatError> {
        let (values, _) = self.table(count)?.as_chunks::<4>();
        Ok(values
            .iter()
            .map(|value| f32::from_le_bytes(*value))
            .collect())
    }
}

impl Model {
    /// Reads a model from the bytes of a model file.
    ///
    /// # Errors
    ///
    /// Returns a [`FormatError`] when the magic or version is wrong, a table
    /// is cut short, or the tables conflict.
    pub(super) fn parse(data: &[u8]) -> Result<Self, FormatError> {
        let mut reader = Reader { bytes: data, at: 0 };
        if reader.take(MAGIC.len())? != MAGIC {
            return Err(FormatError::Magic);
        }
        let version = reader.u32()?;
        if !(1..=VERSION).contains(&version) {
            return Err(FormatError::Version(version));
        }
        let class_count = reader.size()?;
        if class_count > MAX_CLASSES {
            return Err(FormatError::Invalid("the model names too many classes"));
        }
        let classes = (0..class_count)
            .map(|_| reader.string())
            .collect::<Result<Vec<_>, _>>()?;
        let feature_count = reader.size()?;
        let pair_count = reader.size()?;
        let blob_length = reader.size()?;
        let blob = reader.take(blob_length)?.to_vec();
        let feature_offsets = reader.sizes(feature_count.saturating_add(1))?;
        let pair_offsets = reader.sizes(feature_count.saturating_add(1))?;
        let class_ids = reader.take(pair_count)?.to_vec();
        let weights = reader.weights(pair_count)?;
        let word_count = reader.size()?;
        // Each word takes at least three bytes: a length and a class.
        let room = data.len().saturating_sub(reader.at) / 3;
        let mut tag_map = HashMap::with_capacity(word_count.min(room));
        for _ in 0..word_count {
            let word = reader.string()?;
            let id = reader.u8()?;
            if usize::from(id) >= classes.len() {
                return Err(FormatError::Invalid(
                    "a word names a class that is not there",
                ));
            }
            tag_map.insert(word, id);
        }
        let mut meta = HashMap::new();
        if version >= 2 {
            for _ in 0..reader.size()? {
                let key = reader.string()?;
                let value = reader.string()?;
                meta.insert(key, value);
            }
        }
        let model = Self {
            classes,
            blob,
            feature_offsets,
            pair_offsets,
            class_ids,
            weights,
            tag_map,
            meta,
        };
        model.validate()?;
        Ok(model)
    }

    /// Checks that the offsets end where the tables do, that every class id is
    /// in range, and that the features are sorted, which a lookup relies on.
    fn validate(&self) -> Result<(), FormatError> {
        if self.class_ids.len() != self.weights.len() {
            return Err(FormatError::Invalid(
                "the pairs and the weights differ in number",
            ));
        }
        if self.feature_offsets.len() != self.pair_offsets.len() {
            return Err(FormatError::Invalid("the offset tables conflict"));
        }
        if self.feature_offsets.len() > 1
            && (self.feature_offsets.last() != Some(&self.blob.len())
                || self.pair_offsets.last() != Some(&self.weights.len()))
        {
            return Err(FormatError::Invalid(
                "the offsets do not end with their tables",
            ));
        }
        if self
            .class_ids
            .iter()
            .any(|id| usize::from(*id) >= self.classes.len())
        {
            return Err(FormatError::Invalid(
                "a pair names a class that is not there",
            ));
        }
        let names = (0..self.feature_offsets.len().saturating_sub(1))
            .map(|index| self.feature(index))
            .collect::<Option<Vec<&[u8]>>>()
            .ok_or(FormatError::Invalid("a feature runs outside the blob"))?;
        if names.windows(2).any(|pair| pair.first() >= pair.get(1)) {
            return Err(FormatError::Invalid("the features are not sorted"));
        }
        Ok(())
    }

    /// The name of the feature with index `index`.
    fn feature(&self, index: usize) -> Option<&[u8]> {
        let start = self.feature_offsets.get(index).copied()?;
        let end = self.feature_offsets.get(index.checked_add(1)?).copied()?;
        self.blob.get(start..end)
    }

    /// What the model's metadata holds under `key`.
    #[must_use]
    pub(super) fn meta(&self, key: &str) -> Option<&str> {
        self.meta.get(key).map(String::as_str)
    }

    /// The number of classes.
    #[must_use]
    pub(super) fn class_count(&self) -> usize {
        self.classes.len()
    }

    /// The tag of a word that has only one, as the model's own list holds it.
    #[must_use]
    pub(super) fn tag_for(&self, word: &str) -> Option<&str> {
        let id = self.tag_map.get(word)?;
        self.classes.get(usize::from(*id)).map(String::as_str)
    }

    /// Adds the weights of feature `name` to `scores`, one per class.
    pub(super) fn accumulate(&self, name: &[u8], scores: &mut [f32]) {
        let count = self.feature_offsets.len().saturating_sub(1);
        let (mut low, mut high) = (0, count);
        while low < high {
            let middle = low + (high - low) / 2;
            match self.feature(middle) {
                Some(found) if found < name => low = middle + 1,
                Some(found) if found == name => {
                    self.add_pairs(middle, scores);
                    return;
                }
                _ => high = middle,
            }
        }
    }

    /// Adds the pairs of the feature at `index`.
    fn add_pairs(&self, index: usize, scores: &mut [f32]) {
        let (Some(start), Some(end)) = (
            self.pair_offsets.get(index).copied(),
            index
                .checked_add(1)
                .and_then(|next| self.pair_offsets.get(next).copied()),
        ) else {
            return;
        };
        let ids = self.class_ids.get(start..end).unwrap_or_default();
        let weights = self.weights.get(start..end).unwrap_or_default();
        for (id, weight) in ids.iter().zip(weights) {
            if let Some(score) = scores.get_mut(usize::from(*id)) {
                *score += *weight;
            }
        }
    }

    /// The class with the highest score, the first of equals.
    #[must_use]
    pub(super) fn best(&self, scores: &[f32]) -> Option<&str> {
        let mut best: Option<(usize, f32)> = None;
        for (id, score) in scores.iter().enumerate().take(self.classes.len()) {
            if best.is_none_or(|(_, top)| *score > top) {
                best = Some((id, *score));
            }
        }
        best.and_then(|(id, _)| self.classes.get(id).map(String::as_str))
    }
}
