//!
//! ```bash
//! cargo run regenerate --release enwiktionary-pages-*.xml # regenerate "words.fst" binary
//! cargo publish # publish lib including "words.fst" binary
//! ```
//!
//! ## Usage
//!
//! ```
//! use wiktionary_part_of_speech_extract::{ENGLISH_TAG_LOOKUP, TagSet, Tag};
//!
//! assert_eq!(Some(TagSet::of(&[Tag::Noun, Tag::Verb])), ENGLISH_TAG_LOOKUP.get("harbor"));
//! ```

mod tags;

use once_cell::sync::Lazy;

pub use fst::Map;
pub use tags::{Tag, TagSet, TagsBuilder, TagsLookup};

pub static ENGLISH_TAG_LOOKUP: Lazy<TagsLookup<&[u8]>> = Lazy::new(|| {
    tags::TagsLookup::new(include_bytes!("../dist/english-word-tags.fst").as_ref())
        .expect("File was not found")
});
