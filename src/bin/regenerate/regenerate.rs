mod parse;

use parse::PageInfo;
use std::{collections::HashMap, env, path::PathBuf, time::Instant};
use ustr::UstrMap;
use wiktionary_part_of_speech_extract::{Tag, TagSet, TagsBuilder};

static OPENING_PAGE: &str = "<page>";
static CLOSING_PAGE: &str = "</page>";

#[derive(Debug)]
enum MyError {
    Io(std::io::Error),
    InvalidPage(String),
}

impl From<std::io::Error> for MyError {
    fn from(err: std::io::Error) -> Self {
        MyError::Io(err)
    }
}

impl From<String> for MyError {
    fn from(err: String) -> Self {
        MyError::InvalidPage(err)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use parse::ParserRegexes;
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let parser_regexes = ParserRegexes::default();
    let mut tag_counter = UstrMap::default();
    let mut pages = Vec::new();
    // Prints each argument on a separate line
    for file_to_parse in env::args().skip(1) {
        eprintln!("{}", file_to_parse);

        let file = File::open(file_to_parse)?;
        let total_bytes = file.metadata().unwrap().len();
        let buffer = BufReader::new(file);
        let mut page = String::new();
        let mut is_inside_page = false;

        let mut time_since_last_report = Instant::now();
        let mut total_bytes_seen = 0;
        let mut report_percentage_after = 0.05f64;

        for line in buffer.lines() {
            let line = line?;
            total_bytes_seen += line.len();

            if !is_inside_page && line.contains(OPENING_PAGE) {
                is_inside_page = true;
            } else {
                if line.contains(CLOSING_PAGE) {
                    parse::parse_page(&parser_regexes, &mut tag_counter, &mut pages, &page)?;

                    page.clear();

                    is_inside_page = false;
                } else {
                    page.push_str(&line);
                    page.push('\n');
                }
            }

            let percentage_seen = (total_bytes_seen as f64) / (total_bytes as f64);
            if percentage_seen > report_percentage_after {
                let current_instant = Instant::now();
                eprintln!(
                    "{}% complete in {:?}",
                    (report_percentage_after * 100f64).round(),
                    current_instant.duration_since(time_since_last_report.clone())
                );
                report_percentage_after += 0.05;
                time_since_last_report = current_instant;
            }
        }
    }

    let mut used = tag_counter
        .into_iter()
        .map(|(tag, count)| {
            format!(
                "{}({:6}): {:?} ",
                if parser_regexes.alias_lookup.get(tag.as_str()).is_some() {
                    "YES"
                } else {
                    "---"
                },
                count,
                tag.as_str(),
            )
        })
        .collect::<Vec<_>>();

    used.sort();

    eprintln!("{:#?}", pages.len());
    eprintln!("{:#?}", used);

    let fst_path = std::path::Path::new(&std::env::var_os("OUT_DIR").unwrap_or(".".into()))
        .join("enwiktionary-word-tags.fst");

    build_fst_from_pages(
        pages.as_slice(),
        FSTOptions {
            exclude_pages_which_have_only_nouns: true,
            flatten_unicode: true,
        },
        fst_path,
    )?;

    Ok(())
}

pub struct FSTOptions {
    /// Result will always be flattened to lowercase
    pub flatten_unicode: bool,
    /// Anything that is only ProperNoun and/or Noun,
    /// Exclude it from the FST
    ///
    /// Anything that is a Noun & ...X, will include tags for Noun & ...X.
    /// For example, we will still include Noun tag on something that is Adjective and Noun.
    pub exclude_pages_which_have_only_nouns: bool,
}

fn build_fst_from_pages(
    pages: &[PageInfo],
    options: FSTOptions,
    fst_path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let w = std::io::BufWriter::new(std::fs::File::create(fst_path)?);
    let mut tb = TagsBuilder::new(w)?;

    let exclude_before_checking_empty = if options.exclude_pages_which_have_only_nouns {
        let mut excluded = TagSet::default();
        excluded.insert_tag(&Tag::Noun);
        excluded.insert_tag(&Tag::ProperNoun);
        excluded
    } else {
        TagSet::default()
    };

    let mut pages_sorted = pages
        .iter()
        .filter_map(|info| {
            if info
                .tags
                .remove_tag_set(&exclude_before_checking_empty)
                .is_empty()
            {
                // filter out words with no tags
                return None;
            }

            Some((
                // normalize title
                if options.flatten_unicode {
                    unidecode::unidecode(&info.title).to_ascii_lowercase()
                } else {
                    info.title.to_lowercase()
                },
                info.tags.clone(),
            ))
        })
        // flatten
        .fold(
            HashMap::<String, TagSet>::new(),
            |mut acc, (title, tag_set)| {
                acc.entry(title).or_default().extend(tag_set);
                acc
            },
        )
        .into_iter()
        .collect::<Vec<_>>();

    pages_sorted.sort_by(|a, b| a.0.cmp(&b.0));

    tb.extend_iter(pages_sorted)?;

    tb.finish()?;

    Ok(())
}
