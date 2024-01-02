use crate::{
    data_types::{uint16, Offset16, Offset32, Tag},
    decoder::{FromData, Stream},
};

#[allow(non_snake_case)]
#[derive(Debug)]
pub struct GsubHeader {
    pub majorVersion: uint16,        // Major version of the GSUB table, = 1
    pub minorVersion: uint16,        // Minor version of the GSUB table, = 0
    pub scriptListOffset: Offset16,  // Offset to ScriptList table, from beginning of GSUB table
    pub featureListOffset: Offset16, // Offset to FeatureList table, from beginning of GSUB table
    pub lookupListOffset: Offset16,  // Offset to LookupList table, from beginning of GSUB table
    pub featureVariationsOffset: Option<Offset32>, // Offset to FeatureVariations table, from beginning of the GSUB table (may be NULL)
}

impl GsubHeader {
    #[allow(non_snake_case)]
    pub fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let majorVersion = s.read()?;
        let minorVersion = s.read()?;
        let scriptListOffset = s.read()?;
        let featureListOffset = s.read()?;
        let lookupListOffset = s.read()?;
        let featureVariationsOffset = if majorVersion == 1 && minorVersion == 1 {
            Some(s.read()?)
        } else {
            None
        };
        Some(Self {
            majorVersion,
            minorVersion,
            scriptListOffset,
            featureListOffset,
            lookupListOffset,
            featureVariationsOffset,
        })
    }
}

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct ScriptList<'a> {
    pub data: &'a [u8],
    pub scriptCount: uint16,              // Number of ScriptRecords
    pub scriptRecords: Vec<ScriptRecord>, // Array of ScriptRecords, listed alphabetically by script tag
}

impl<'a> ScriptList<'a> {
    #[allow(non_snake_case)]
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let scriptCount: u16 = s.read()?;
        let scriptRecords = s.read_array(scriptCount as _)?;
        Some(Self {
            data,
            scriptCount,
            scriptRecords,
        })
    }

    pub fn get(&self, index: usize) -> Option<Script> {
        self.scriptRecords
            .get(index)
            .and_then(|x| self.data.get(x.scriptOffset as usize..))
            .and_then(Script::parse)
    }
}

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct ScriptRecord {
    pub scriptTag: Tag,         // 4-byte script tag identifier
    pub scriptOffset: Offset16, // Offset to Script table, from beginning of ScriptList
}

impl FromData for ScriptRecord {
    const SIZE: usize = Tag::SIZE + u16::SIZE;
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(Self {
            scriptTag: s.read()?,
            scriptOffset: s.read()?,
        })
    }
}

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct Script<'a> {
    pub data: &'a [u8],
    pub defaultLangSysOffset: Offset16, // Offset to default LangSys table, from beginning of Script table — may be NULL
    pub langSysCount: uint16, // Number of LangSysRecords for this script — excluding the default LangSys
    pub langSysRecords: Vec<LangSysRecord>, // Array of LangSysRecords, listed alphabetically by LangSys tag
}

impl<'a> Script<'a> {
    #[allow(non_snake_case)]
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let defaultLangSysOffset = s.read()?;
        let langSysCount: u16 = s.read()?;
        let langSysRecords = s.read_array(langSysCount as _)?;
        Some(Self {
            data,
            defaultLangSysOffset,
            langSysCount,
            langSysRecords,
        })
    }

    pub fn get_default_lang_sys_table(&self) -> Option<LangSys> {
        if self.defaultLangSysOffset == 0 {
            None
        } else {
            self.data
                .get(self.defaultLangSysOffset as usize..)
                .and_then(LangSys::parse)
        }
    }

    pub fn get(&self, index: usize) -> Option<LangSys> {
        self.langSysRecords
            .get(index)
            .and_then(|x| self.data.get(x.langSysOffset as usize..))
            .and_then(LangSys::parse)
    }
}

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct LangSysRecord {
    pub langSysTag: Tag,         // 4-byte LangSysTag identifier
    pub langSysOffset: Offset16, // Offset to LangSys table, from beginning of Script table
}

impl FromData for LangSysRecord {
    const SIZE: usize = Tag::SIZE + u16::SIZE;
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(Self {
            langSysTag: s.read()?,
            langSysOffset: s.read()?,
        })
    }
}

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct LangSys {
    pub lookupOrderOffset: Offset16, // = NULL (reserved for an offset to a reordering table)
    pub requiredFeatureIndex: uint16, // Index of a feature required for this language system; if no required features = 0xFFFF
    pub featureIndexCount: uint16, // Number of feature index values for this language system — excludes the required feature
    pub featureIndices: Vec<uint16>, // Array of indices into the FeatureList, in arbitrary order
}

impl LangSys {
    #[allow(non_snake_case)]
    pub fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let lookupOrderOffset = s.read()?;
        let requiredFeatureIndex = s.read()?;
        let featureIndexCount = s.read()?;
        let featureIndices = s.read_array(featureIndexCount as _)?;
        Some(Self {
            lookupOrderOffset,
            requiredFeatureIndex,
            featureIndexCount,
            featureIndices,
        })
    }
}

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct FeatureList<'a> {
    pub data: &'a [u8],
    pub featureCount: uint16, // Number of FeatureRecords in this table
    pub featureRecords: Vec<FeatureRecord>, // Array of FeatureRecords — zero-based (first feature has FeatureIndex = 0), listed alphabetically by feature tag
}

impl<'a> FeatureList<'a> {
    #[allow(non_snake_case)]
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let featureCount = s.read()?;
        let featureRecords = s.read_array(featureCount as _)?;
        Some(Self {
            data,
            featureCount,
            featureRecords,
        })
    }

    pub fn get(&self, index: usize) -> Option<Feature> {
        self.featureRecords
            .get(index)
            .and_then(|x| self.data.get(x.featureOffset as usize..))
            .and_then(Feature::parse)
    }
}

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct FeatureRecord {
    pub featureTag: Tag,         // 4-byte feature identification tag
    pub featureOffset: Offset16, // Offset to Feature table, from beginning of FeatureList
}

impl FromData for FeatureRecord {
    const SIZE: usize = Tag::SIZE + u16::SIZE;
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(Self {
            featureTag: s.read()?,
            featureOffset: s.read()?,
        })
    }
}

#[allow(non_snake_case)]
#[derive(Debug)]
pub struct Feature {
    pub featureParamsOffset: Offset16, // Offset from start of Feature table to FeatureParams table, if defined for the feature and present, else NULL
    pub lookupIndexCount: uint16,      // Number of LookupList indices for this feature
    pub lookupListIndices: Vec<uint16>, // Array of indices into the LookupList — zero-based (first lookup is LookupListIndex = 0)
}

impl Feature {
    #[allow(non_snake_case)]
    pub fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let featureParamsOffset = s.read()?;
        let lookupIndexCount = s.read()?;
        let lookupListIndices = s.read_array(lookupIndexCount as _)?;
        Some(Self {
            featureParamsOffset,
            lookupIndexCount,
            lookupListIndices,
        })
    }
}

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct LookupList<'a> {
    pub data: &'a [u8],
    pub lookupCount: uint16,          // Number of lookups in this table
    pub lookupOffsets: Vec<Offset16>, // Array of offsets to Lookup tables, from beginning of LookupList — zero based (first lookup is Lookup index = 0)
}

impl<'a> LookupList<'a> {
    #[allow(non_snake_case)]
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let lookupCount = s.read()?;
        let lookupOffsets = s.read_array(lookupCount as _)?;
        Some(Self {
            data,
            lookupCount,
            lookupOffsets,
        })
    }

    pub fn get(&self, index: usize) -> Option<Lookup> {
        self.lookupOffsets
            .get(index)
            .and_then(|x| self.data.get(*x as usize..))
            .and_then(Lookup::parse)
    }
}

#[derive(Debug)]
pub enum GsubLookupType {
    Single = 1,                // (format 1.1 1.2) Replace one glyph with one glyph
    Multiple = 2,              // (format 2.1) Replace one glyph with more than one glyph
    Alternate = 3,             // (format 3.1) Replace one glyph with one of many glyphs
    Ligature = 4,              // (format 4.1) Replace multiple glyphs with one glyph
    Context = 5,               // (format 5.1 5.2 5.3) Replace one or more glyphs in context
    ChainingContext = 6,       // (format 6.1 6.2 6.3) Replace one or more glyphs in chained context
    ExtensionSubstitution = 7, // (format 7.1) Extension mechanism for other substitutions (i.e. this excludes the Extension type substitution itself)
    ReverseChainingContextSingle = 8, // (format 8.1)
                               // Reserved, For future use (set to zero)
}

impl GsubLookupType {
    pub fn new(lookup_type: u16) -> Self {
        match lookup_type {
            1 => Self::Single,
            2 => Self::Multiple,
            3 => Self::Alternate,
            4 => Self::Ligature,
            5 => Self::Context,
            6 => Self::ChainingContext,
            7 => Self::ExtensionSubstitution,
            8 => Self::ReverseChainingContextSingle,
            _ => panic!("invalid lookup type"),
        }
    }
}

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct Lookup<'a> {
    pub data: &'a [u8],
    pub lookupType: GsubLookupType, // Different enumerations for GSUB and GPOS
    pub lookupFlag: uint16,         // Lookup qualifiers
    pub subTableCount: uint16,      // Number of subtables for this lookup
    pub subTableOffsets: Vec<Offset16>, // Array of offsets to lookup subtables, from beginning of Lookup table
    pub markFilteringSet: uint16, // Index (base 0) into GDEF mark glyph sets structure. This field is only present if the USE_MARK_FILTERING_SET lookup flag is set.
}

impl<'a> Lookup<'a> {
    #[allow(non_snake_case)]
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let lookupType = GsubLookupType::new(s.read()?);
        let lookupFlag = s.read()?;
        let subTableCount: u16 = s.read()?;
        let subTableOffsets = s.read_array(subTableCount as _)?;
        let markFilteringSet = s.read()?;
        Some(Self {
            data,
            lookupType,
            lookupFlag,
            subTableCount,
            subTableOffsets,
            markFilteringSet,
        })
    }
}

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct ChainedSequenceContextFormat1 {
    pub format: uint16,                          // Format identifier: format = 1
    pub coverageOffset: Offset16, // Offset to Coverage table, from beginning of ChainSequenceContextFormat1 table
    pub chainedSeqRuleSetCount: uint16, // Number of ChainedSequenceRuleSet tables
    pub chainedSeqRuleSetOffsets: Vec<Offset16>, // [chainedSeqRuleSetCount] Array of offsets to ChainedSeqRuleSet tables, from beginning of ChainedSequenceContextFormat1 table (may be NULL)
}

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct ChainedSequenceContextFormat2 {
    pub format: uint16,                               // Format identifier: format = 2
    pub coverageOffset: Offset16, // Offset to Coverage table, from beginning of ChainedSequenceContextFormat2 table
    pub backtrackClassDefOffset: Offset16, // Offset to ClassDef table containing backtrack sequence context, from beginning of ChainedSequenceContextFormat2 table
    pub inputClassDefOffset: Offset16, // Offset to ClassDef table containing input sequence context, from beginning of ChainedSequenceContextFormat2 table
    pub lookaheadClassDefOffset: Offset16, // Offset to ClassDef table containing lookahead sequence context, from beginning of ChainedSequenceContextFormat2 table
    pub chainedClassSeqRuleSetCount: uint16, // Number of ChainedClassSequenceRuleSet tables
    pub chainedClassSeqRuleSetOffsets: Vec<Offset16>, // [chainedClassSeqRuleSetCount] Array of offsets to ChainedClassSequenceRuleSet tables, from beginning of ChainedSequenceContextFormat2 table (may be NULL)
}

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct SequenceLookupRecord {
    pub sequenceIndex: uint16, // Index (zero-based) into the input glyph sequence
    pub lookupListIndex: uint16, // Index (zero-based) into the LookupList
}

impl FromData for SequenceLookupRecord {
    const SIZE: usize = uint16::SIZE * 2;
    #[allow(non_snake_case)]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let sequenceIndex = s.read()?;
        let lookupListIndex = s.read()?;
        Some(Self {
            sequenceIndex,
            lookupListIndex,
        })
    }
}

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct ChainedSequenceContextFormat3<'a> {
    pub data: &'a [u8],
    pub format: uint16,                          // Format identifier: format = 3
    pub backtrackGlyphCount: uint16,             // Number of glyphs in the backtrack sequence
    pub backtrackCoverageOffsets: Vec<Offset16>, // [backtrackGlyphCount] Array of offsets to coverage tables for the backtrack sequence
    pub inputGlyphCount: uint16,                 // Number of glyphs in the input sequence
    pub inputCoverageOffsets: Vec<Offset16>, // [inputGlyphCount] Array of offsets to coverage tables for the input sequence
    pub lookaheadGlyphCount: uint16,         // Number of glyphs in the lookahead sequence
    pub lookaheadCoverageOffsets: Vec<Offset16>, // [lookaheadGlyphCount] Array of offsets to coverage tables for the lookahead sequence
    pub seqLookupCount: uint16,                  // Number of SequenceLookupRecords
    pub seqLookupRecords: Vec<SequenceLookupRecord>, // [seqLookupCount] Array of SequenceLookupRecords
}

impl<'a> ChainedSequenceContextFormat3<'a> {
    #[allow(non_snake_case)]
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let format = s.read()?;
        let backtrackGlyphCount: u16 = s.read()?;
        let backtrackCoverageOffsets = s.read_array(backtrackGlyphCount as usize)?;
        let inputGlyphCount: u16 = s.read()?;
        let inputCoverageOffsets = s.read_array(inputGlyphCount as usize)?;
        let lookaheadGlyphCount: u16 = s.read()?;
        let lookaheadCoverageOffsets = s.read_array(lookaheadGlyphCount as usize)?;
        let seqLookupCount: u16 = s.read()?;
        let seqLookupRecords = s.read_array(seqLookupCount as usize)?;
        Some(Self {
            data,
            format,
            backtrackGlyphCount,
            backtrackCoverageOffsets,
            inputGlyphCount,
            inputCoverageOffsets,
            lookaheadGlyphCount,
            lookaheadCoverageOffsets,
            seqLookupCount,
            seqLookupRecords,
        })
    }
}

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct CoverageFormat1 {
    pub coverageFormat: uint16,  // Format identifier — format = 1
    pub glyphCount: uint16,      // Number of glyphs in the glyph array
    pub glyphArray: Vec<uint16>, // [glyphCount] Array of glyph IDs — in numerical order
}

impl CoverageFormat1 {
    #[allow(non_snake_case)]
    pub fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let coverageFormat = s.read()?;
        let glyphCount: u16 = s.read()?;
        let glyphArray = s.read_array(glyphCount as usize)?;
        Some(Self {
            coverageFormat,
            glyphCount,
            glyphArray,
        })
    }
}

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct CoverageFormat2 {
    pub coverageFormat: uint16,         // Format identifier — format = 2
    pub rangeCount: uint16,             // Number of RangeRecords
    pub rangeRecords: Vec<RangeRecord>, // [rangeCount] Array of glyph ranges — ordered by startGlyphID.
}

impl CoverageFormat2 {
    #[allow(non_snake_case)]
    pub fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let coverageFormat = s.read()?;
        let rangeCount: u16 = s.read()?;
        let rangeRecords = s.read_array(rangeCount as usize)?;
        Some(Self {
            coverageFormat,
            rangeCount,
            rangeRecords,
        })
    }
}

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct RangeRecord {
    pub startGlyphID: uint16,       // First glyph ID in the range
    pub endGlyphID: uint16,         // Last glyph ID in the range
    pub startCoverageIndex: uint16, // Coverage Index of first glyph ID in range
}

impl FromData for RangeRecord {
    const SIZE: usize = u16::SIZE * 3;
    #[allow(non_snake_case)]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let startGlyphID = s.read()?;
        let endGlyphID = s.read()?;
        let startCoverageIndex = s.read()?;
        Some(Self {
            startGlyphID,
            endGlyphID,
            startCoverageIndex,
        })
    }
}

#[derive(Debug)]
pub struct GsubTable<'a> {
    pub header: GsubHeader,
    pub script_list: ScriptList<'a>,
    pub feature_list: FeatureList<'a>,
    pub lookup_list: LookupList<'a>,
}

impl<'a> GsubTable<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let header = GsubHeader::parse(data)?;
        let script_list = ScriptList::parse(data.get(header.scriptListOffset as _..)?)?;
        let feature_list = FeatureList::parse(data.get(header.featureListOffset as _..)?)?;
        let lookup_list = LookupList::parse(data.get(header.lookupListOffset as _..)?)?;
        Some(Self {
            header,
            script_list,
            feature_list,
            lookup_list,
        })
    }
}
