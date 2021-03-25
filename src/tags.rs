#[derive(Default, Clone)]
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

    pub(crate) fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub(crate) fn insert_tag(&mut self, tag: &Tag) {
        self.0 |= tag.to_mask();
    }

    pub(crate) fn remove_tag_set(&self, tag_set: &Self) -> Self {
        TagSet(self.0 & !tag_set.0)
    }

    pub(crate) fn extend(&mut self, other: Self) {
        self.0 |= other.0;
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
                    Some(Tag::from_u32((i + trailing_zeros) as u32))
                } else {
                    None
                }
            })
    }
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
    pub fn new(writer: W) -> Result<Self, fst::Error> {
        Ok(TagsBuilder(fst::MapBuilder::new(writer)?))
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

    pub fn insert_tag_set(&mut self, key: &str, tag_set: &TagSet) -> Result<(), String> {
        self.0.insert(key, tag_set.to_mask() as u64).map_err(|err| {
            format!(
                "Expected to insert key ({:?}) with tags ({:?}), but got error:\n{:#?}",
                key, tag_set, err
            )
        })
    }

    pub fn extend_iter<I: IntoIterator<Item = (String, TagSet)>>(
        &mut self,
        iter: I,
    ) -> Result<(), String> {
        self.0
            .extend_iter(
                iter.into_iter()
                    .map(|(key, tag_set)| (key, tag_set.to_mask() as u64)),
            )
            .map_err(|err| format!("Expected to insert key but got error:\n{:#?}", err))
    }

    pub fn finish(self) -> Result<(), fst::Error> {
        self.0.finish()
    }
}

pub struct TagsLookup<D>(fst::Map<D>);

impl<D: AsRef<[u8]>> TagsLookup<D> {
    pub fn new(data: D) -> Result<Self, String> {
        fst::Map::new(data)
            .map(TagsLookup)
            .map_err(|fst_err| format!("Invalid TagsLookup: {:?}", fst_err))
    }

    pub fn get(&self, key: &str) -> Option<TagSet> {
        self.0.get(key).map(|mask| TagSet::from_mask(mask as u32))
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
