use super::{Tag, TagSet, TagsBuilder};
use fst::Map;
use regex::Regex;
use ustr::{ustr, UstrMap};

const TAG_ALIASES: &[(&Tag, &[&str])] = &[
    (
        &Tag::Adjective,
        &[
            "en-adj",
            "en-adjective",
            "en|head|adj",
            "en|head|adjective",
            "head|en|adjective",
        ],
    ),
    (
        &Tag::Adverb,
        &[
            "en-adv",
            "en-adverb",
            "en|head|adv",
            "en|head|adverb",
            "head|en|adverb",
        ],
    ),
    (
        &Tag::Conjunction,
        &[
            "en-con",
            "en-conj",
            "en-conjunction",
            "en-conj-simple",
            "en|head|con",
            "en|head|conj",
            "en|head|conjunction",
        ],
    ),
    (
        &Tag::Determiner,
        &["en-det", "en|head|det", "head|en|determiner"],
    ),
    (
        &Tag::Interjection,
        &[
            "en-interj",
            "en-interjection",
            "en-intj",
            "en|head|interj",
            "en|head|interjection",
            "head|en|interjection",
        ],
    ),
    (
        &Tag::Noun,
        &[
            "en-noun",
            "en|head|noun",
            "head|en|noun",
            "head|en|noun form",
            "en-plural noun",
        ],
    ),
    (&Tag::Numeral, &["en-num", "en|head|num"]),
    (&Tag::Particle, &["en-part", "en|head|part"]),
    (&Tag::Postposition, &["en-postp", "en|head|postp"]),
    (&Tag::Preposition, &["en-prep", "en|head|prep"]),
    (&Tag::Pronoun, &["en-pron", "en|head|pron"]),
    (&Tag::ProperNoun, &["en-proper noun", "en|head|proper noun"]),
    (
        &Tag::Verb,
        &["en-verb", "head|en|verb", "head|en|verb form"],
    ),
];

pub struct ParserRegexes {
    tag_regex: Regex,
    title_regex: Regex,
    opening_text_regex: Regex,
    pub alias_lookup: Map<Vec<u8>>,
}

impl std::default::Default for ParserRegexes {
    fn default() -> Self {
        let mut tags_builder = TagsBuilder::in_memory();

        let mut tag_aliases: Vec<_> = TAG_ALIASES
            .iter()
            .flat_map(|(tag, aliases)| aliases.into_iter().map(move |alias| (alias, tag.clone())))
            .collect();

        tag_aliases.sort_by(|(alias1, _), (alias2, _)| alias1.cmp(alias2));

        for (alias, tag) in tag_aliases {
            tags_builder.insert_tag(alias, tag);
        }

        ParserRegexes {
            alias_lookup: Map::new(tags_builder.into_inner()).unwrap(),
            tag_regex: Regex::new(
                r#"(?x)
                    \{\{\s*
                    ((?:en\-|head\|en\|)[^\|{}\d\.&]+)
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
