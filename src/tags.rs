#[derive(Default)]
pub struct TagSet(u32);

impl std::fmt::Debug for TagSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.tags().collect::<Vec<_>>())
    }
}

impl TagSet {
    fn to_mask(&self) -> u32 {
        self.0
    }

    pub fn from_mask(mask: u32) -> Self {
        Self(mask)
    }

    pub(crate) fn insert_tag(&mut self, tag: &Tag) {
        self.0 |= tag.to_mask();
    }

    pub(crate) fn insert_tag_mask(&mut self, tag: u32) {
        self.0 |= tag;
    }

    pub fn tags(&self) -> impl Iterator<Item = Tag> {
        // We iterate mask's bits from the first non zero to the last non zero
        // Each iteration will check one bit
        // If the bit is non zero we yield the `Tag` matching this bit index

        const BIT_IN_MASK: usize = 32;
        let trailing_zeros = self.0.trailing_zeros() as usize;

        std::iter::repeat(self.0)
            .take(BIT_IN_MASK - self.0.leading_zeros() as usize)
            .skip(trailing_zeros)
            .enumerate()
            .flat_map(move |(i, mask)| {
                if mask & 1 << i + trailing_zeros != 0 {
                    Some(Tag::from_u32(i as u32))
                } else {
                    None
                }
            })
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Tag {
    /// adj
    Adjective,
    /// adv
    Adverb,
    /// con
    Conjunction,
    /// det
    Determiner,
    /// interj
    Interjection,
    /// noun
    Noun,
    /// num
    Numeral,
    /// part
    Particle,
    /// postp
    Postposition,
    /// prep
    Preposition,
    /// pron
    Pronoun,
    /// proper noun
    ProperNoun,
    /// verb
    Verb,
}

pub struct TagsBuilder<W: std::io::Write>(fst::MapBuilder<W>);

// in memory construction
impl TagsBuilder<Vec<u8>> {
    pub fn in_memory() -> Self {
        TagsBuilder(fst::MapBuilder::memory())
    }
    /// # panics
    pub fn into_inner(self) -> Vec<u8> {
        self.0.into_inner().unwrap()
    }
}

impl<W: std::io::Write> TagsBuilder<W> {
    pub fn new(writer: W) -> Self {
        TagsBuilder(fst::MapBuilder::new(writer).unwrap())
    }

    pub fn insert_tag(&mut self, key: &str, tag: &Tag) {
        self.0
            .insert(key, tag.to_mask() as u64)
            .map_err(|err| {
                format!(
                    "Expected to insert key ({:?}), but got error:\n{:#?}",
                    key, err
                )
            })
            .unwrap();
    }

    pub fn insert_tag_set(&mut self, key: &str, tag_set: &TagSet) {
        self.0
            .insert(key, tag_set.to_mask() as u64)
            .map_err(|err| {
                format!(
                    "Expected to insert key ({:?}) with tags ({:?}), but got error:\n{:#?}",
                    key, tag_set, err
                )
            })
            .unwrap();
    }
}

impl Tag {
    fn to_mask(self) -> u32 {
        1 << match self {
            Tag::Adjective => 1,
            Tag::Adverb => 2,
            Tag::Conjunction => 3,
            Tag::Determiner => 4,
            Tag::Interjection => 5,
            Tag::Noun => 6,
            Tag::Numeral => 7,
            Tag::Particle => 8,
            Tag::Postposition => 9,
            Tag::Preposition => 10,
            Tag::Pronoun => 11,
            Tag::ProperNoun => 12,
            Tag::Verb => 13,
        }
    }

    fn from_u32(i: u32) -> Self {
        match i {
            1 => Tag::Adjective,
            2 => Tag::Adverb,
            3 => Tag::Conjunction,
            4 => Tag::Determiner,
            5 => Tag::Interjection,
            6 => Tag::Noun,
            7 => Tag::Numeral,
            8 => Tag::Particle,
            9 => Tag::Postposition,
            10 => Tag::Preposition,
            11 => Tag::Pronoun,
            12 => Tag::ProperNoun,
            13 => Tag::Verb,
            other => panic!("Invalid Tag variant from_u32({})", other),
        }
    }
}
