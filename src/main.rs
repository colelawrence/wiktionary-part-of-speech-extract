use std::{env, io::Read, time::Instant};

use regex::internal::Inst;
use ustr::{UstrMap, UstrSet};

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

/// Adjective: adj
/// Adverb: adv
/// Conjunction: con
/// Determiner: det
/// Interjection: interj
/// Noun: noun
/// Numeral: num
/// Particle: part
/// Postposition: postp
/// Preposition: prep
/// Pronoun: pron
/// Proper noun: proper noun
/// Verb: verb
fn main() -> Result<(), MyError> {
    use parse::ParserRegexes;
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let parser_regexes = ParserRegexes::default();
    let mut interner = UstrMap::default();
    let mut pages = Vec::new();
    // Prints each argument on a separate line
    for file_to_parse in env::args().skip(1) {
        println!("{}", file_to_parse);

        let file = File::open(file_to_parse)?;
        let total_bytes = file.metadata().unwrap().len();
        let buffer = BufReader::new(file);
        let mut page = String::new();
        let mut is_inside_page = false;

        let mut time_since_last_report = Instant::now();
        let mut total_bytes_seen = 0;
        let mut report_percentage_after = 0f64;

        for line in buffer.lines() {
            let line = line?;
            total_bytes_seen += line.len();

            if !is_inside_page && line.contains(OPENING_PAGE) {
                is_inside_page = true;
            } else {
                if line.contains(CLOSING_PAGE) {
                    parse::parse_page(&parser_regexes, &mut interner, &mut pages, &page)?;

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
                println!(
                    "{}% complete in {:?}",
                    (report_percentage_after * 100f64).round(),
                    current_instant.duration_since(time_since_last_report.clone())
                );
                report_percentage_after += 0.05;
                time_since_last_report = current_instant;
            }
        }
    }

    println!("{:#?}", pages.len());
    println!("{:#?}", interner);

    Ok(())
}

mod parse {
    use regex::Regex;
    use ustr::{ustr, UstrMap, UstrSet};

    pub struct ParserRegexes {
        tag_regex: Regex,
        title_regex: Regex,
    }

    impl std::default::Default for ParserRegexes {
        fn default() -> Self {
            ParserRegexes {
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
            }
        }
    }

    #[derive(Debug)]
    pub struct PageInfo {
        title: String,
        tags: UstrSet,
    }

    pub fn parse_page(
        regexes: &ParserRegexes,
        tag_interner: &mut UstrMap<usize>,
        add_to: &mut Vec<PageInfo>,
        page_contents: &str,
    ) -> Result<(), String> {
        regexes
            .title_regex
            .captures(&page_contents)
            .ok_or_else(|| format!("Failed to find title for page"))
            .map(|title| {
                add_to.push(PageInfo {
                    title: String::from(&title[1]),
                    tags: regexes
                        .tag_regex
                        .captures_iter(&page_contents)
                        .map(|cap| {
                            let handle = ustr(&cap[1].trim());
                            *tag_interner.entry(handle).or_default() += 1;
                            handle
                        })
                        .collect(),
                });
            })
    }
}
