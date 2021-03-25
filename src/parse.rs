use super::{Tag, TagSet, TagsBuilder};
use fst::Map;
use regex::Regex;
use ustr::{ustr, UstrMap};

const TAG_ALIASES: &[(&Tag, &[&str])] = &[
    (&Tag::Adjective, &["en-adj", "en-adjective"]),
    (&Tag::Adverb, &["en-adv", "en-adverb"]),
    (&Tag::Conjunction, &["en-con", "en-conjunction"]),
    (&Tag::Determiner, &["en-det"]),
    (
        &Tag::Interjection,
        &["en-interj", "en-interjection", "en-intj"],
    ),
    (&Tag::Noun, &["en-noun"]),
    (&Tag::Numeral, &["en-num"]),
    (&Tag::Particle, &["en-part"]),
    (&Tag::Postposition, &["en-postp"]),
    (&Tag::Preposition, &["en-prep"]),
    (&Tag::Pronoun, &["en-pron"]),
    (&Tag::ProperNoun, &["en-proper noun"]),
    (&Tag::Verb, &["en-verb"]),
];

pub struct ParserRegexes {
    tag_regex: Regex,
    title_regex: Regex,
    opening_text_regex: Regex,
    alias_lookup: Map<Vec<u8>>,
}

impl std::default::Default for ParserRegexes {
    fn default() -> Self {
        let mut tags_builder = TagsBuilder::in_memory();

        for (tag, aliases) in TAG_ALIASES.iter() {
            for alias in aliases.iter() {
                tags_builder.insert_tag(alias, tag);
            }
        }

        ParserRegexes {
            alias_lookup: Map::new(tags_builder.into_inner()).unwrap(),
            tag_regex: Regex::new(
                r#"(?x)
                    \{\{\s*
                    ((?:en)\-[^\\|{}\d\.&]+)
                "#,
            )
            .unwrap(),
            title_regex: Regex::new(
                r#"(?x)
                    <title>
                    ([^<]+)
                "#,
            )
            .unwrap(),
            opening_text_regex: Regex::new(r#"<text"#).unwrap(),
        }
    }
}

#[derive(Debug)]
pub struct PageInfo {
    pub title: String,
    pub tags: TagSet,
}

pub fn parse_page(
    regexes: &ParserRegexes,
    tag_counter: &mut UstrMap<usize>,
    add_to: &mut Vec<PageInfo>,
    page_contents: &str,
) -> Result<(), String> {
    regexes
        .title_regex
        .captures(&page_contents)
        .ok_or_else(|| format!("Failed to find title for page"))
        .map(|title| {
            let mut tags = TagSet::default();
            if let Some(m) = regexes.opening_text_regex.find(&page_contents) {
                for wiki_tag in regexes
                    .tag_regex
                    .captures_iter(&page_contents[m.end()..])
                    .map(|cap| {
                        let handle = ustr(&cap[1].trim());
                        *tag_counter.entry(handle).or_default() += 1;
                        handle
                    })
                {
                    if let Some(existing_tag_mask) = regexes.alias_lookup.get(wiki_tag.as_str()) {
                        tags.insert_tag_mask(existing_tag_mask as u32);
                    }
                }
                add_to.push(PageInfo {
                    title: String::from(&title[1]),
                    tags,
                });
            }
        })
}
