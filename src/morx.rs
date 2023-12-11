use core::fmt;

use num_enum::TryFromPrimitive;

use crate::{
    data_types::{uint16, uint32},
    decoder::{FromData, Stream},
};

#[derive(Debug)]
pub struct MorxTable<'a> {
    pub header: MorxHeader,
    pub chains: Vec<Chain<'a>>,
}

impl<'a> MorxTable<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let header: MorxHeader = s.read()?;
        let mut chains = Vec::new();
        for _ in 0..header.nChains {
            chains.push(Chain::parse(&mut s, header.version)?);
        }
        Some(Self { header, chains })
    }
}

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct MorxHeader {
    pub version: uint16, // Version number of the extended glyph metamorphosis table (either 2 or 3)
    pub unused: uint16,  // Set to 0
    pub nChains: uint32, // Number of metamorphosis chains contained in this table.
}

impl FromData for MorxHeader {
    const SIZE: usize = (uint16::SIZE * 2) + uint32::SIZE;
    #[allow(non_snake_case)]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let version = s.read()?;
        assert!(version == 2 || version == 3);
        let unused = s.read()?;
        assert_eq!(unused, 0);
        let nChains = s.read()?;
        assert!(nChains > 0);
        Some(Self {
            version,
            unused,
            nChains,
        })
    }
}

#[derive(Debug)]
pub struct Chain<'a> {
    pub header: ChainHeader,
    pub feature_tables: Vec<FeatureTable>,
    pub subtables: Vec<MorxSubtable<'a>>,
}

impl<'a> Chain<'a> {
    pub fn parse(stream: &mut Stream<'a>, version: u16) -> Option<Self> {
        let start = stream.get_offset();
        let header: ChainHeader = stream.read()?;

        stream.set_offset(start);
        let len = header.chainLength as usize;
        let data = stream.read_bytes(len)?;
        let mut s = Stream::new(data);
        s.set_offset(ChainHeader::SIZE);

        let mut feature_tables = Vec::new();
        for _ in 0..header.nFeatureEntries {
            feature_tables.push(s.read()?);
        }

        let mut subtables = Vec::new();
        for _ in 0..header.nSubtables {
            subtables.push(MorxSubtable::parse(&mut s)?);
        }

        if version == 3 {
            // Subtable Glyph Coverage Tables
        }

        Some(Self {
            header,
            feature_tables,
            subtables,
        })
    }
}

pub struct FeatureFlags(pub uint32);
impl FromData for FeatureFlags {
    const SIZE: usize = uint32::SIZE;
    fn parse(data: &[u8]) -> Option<Self> {
        uint32::parse(data).map(Self)
    }
}

impl fmt::Debug for FeatureFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{0} (= 0x{0:x})", self.0)
    }
}

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct ChainHeader {
    pub defaultFlags: FeatureFlags, // The default specification for subtables.
    pub chainLength: uint32, // Total byte count, including this header; must be a multiple of 4.
    pub nFeatureEntries: uint32, // Number of feature subtable entries.
    pub nSubtables: uint32,  // The number of subtables in the chain.
}

impl FromData for ChainHeader {
    const SIZE: usize = uint32::SIZE * 4;
    #[allow(non_snake_case)]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let defaultFlags = s.read()?;
        let chainLength = s.read()?;
        let nFeatureEntries = s.read()?;
        let nSubtables = s.read()?;

        assert_eq!(chainLength % 4, 0);
        assert!(nFeatureEntries > 0);
        assert!(nSubtables > 0);

        Some(Self {
            defaultFlags,
            chainLength,
            nFeatureEntries,
            nSubtables,
        })
    }
}

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct FeatureTable {
    pub featureType: uint16,        // The type of feature.
    pub featureSetting: uint16,     // The feature's setting (aka selector)
    pub enableFlags: FeatureFlags,  // Flags for the settings that this feature and setting enables.
    pub disableFlags: FeatureFlags, // Complement of flags for the settings that this feature and setting disable.
}

impl FromData for FeatureTable {
    const SIZE: usize = (uint16::SIZE * 2) + (uint32::SIZE * 2);
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(Self {
            featureType: s.read()?,
            featureSetting: s.read()?,
            enableFlags: s.read()?,
            disableFlags: s.read()?,
        })
    }
}

#[derive(Debug)]
pub enum MorxSubtable<'a> {
    Rearrangement(MorxSubtableRearrangement),
    Contextual(MorxSubtableContextual<'a>),
    Ligature(MorxSubtableLigature<'a>),
    Noncontextual(MorxSubtableNoncontextual<'a>),
    Insertion(MorxSubtableInsertion<'a>),
}

impl<'a> MorxSubtable<'a> {
    pub fn parse(stream: &mut Stream<'a>) -> Option<Self> {
        let header: MorxSubtableHeader = stream.read()?;

        let len = header.length as usize - MorxSubtableHeader::SIZE;
        let data = stream.read_bytes(len)?;

        match header.get_type() {
            MorxSubtableType::Rearrangement => Some(Self::Rearrangement(
                MorxSubtableRearrangement::parse(header, data)?,
            )),
            MorxSubtableType::Contextual => Some(Self::Contextual(MorxSubtableContextual::parse(
                header, data,
            )?)),
            MorxSubtableType::Ligature => {
                Some(Self::Ligature(MorxSubtableLigature::parse(header, data)?))
            }
            MorxSubtableType::Noncontextual => Some(Self::Noncontextual(
                MorxSubtableNoncontextual::parse(header, data)?,
            )),
            MorxSubtableType::Insertion => {
                Some(Self::Insertion(MorxSubtableInsertion::parse(header, data)?))
            }
        }
    }
}

// グリフごとのテーブル (per-glyph table) は使用されません。
// つまり、 The Entry Subtable の glyphOffsets が存在しないことを表している。
pub struct MorxSubtableRearrangement {
    pub header: MorxSubtableHeader,
    pub stx_header: STXHeader,
    pub class_table: LookupTable, // グリフインデックスをクラスにマップするルックアップテーブル。
    pub state_array: Vec<Vec<uint16>>, // クラス数の Vec<uint16> が状態の数だけある。State 0 は start of text state で State 1 は start of line state で事前定義されている。
    pub entry_table: Vec<RearrangementEntry>,
}

impl fmt::Debug for MorxSubtableRearrangement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let number_of_classes = self.stx_header.nClasses as usize;
        writeln!(f, "Number of classes: {}", number_of_classes)?;
        for (state_index, state) in self.state_array.iter().enumerate() {
            writeln!(f, "State {}:", state_index)?;
            for (class_index, entry_index) in state.iter().enumerate() {
                let entry = &self.entry_table[*entry_index as usize];
                let new_state_index = entry.get_new_state_index(number_of_classes);
                writeln!(f, "  Class {}: New state {}", class_index, new_state_index)?;
            }
        }
        Ok(())
    }
}

impl MorxSubtableRearrangement {
    pub fn parse(header: MorxSubtableHeader, data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let stx_header: STXHeader = s.read()?;

        // クラスルックアップテーブル、状態配列、エントリサブテーブルの順序は決まっていない。
        // そのためデータ範囲の上限を計算して、その範囲でテーブルを作成する。

        let class_start = stx_header.classTableOffset as usize;
        let state_start = stx_header.stateArrayOffset as usize;
        let entry_start = stx_header.entryTableOffset as usize;
        let end = data.len();

        let offsets = [class_start, state_start, entry_start, end];
        let ranks = make_rank(&offsets);
        let class_end = offsets[ranks[0] + 1];
        let state_end = offsets[ranks[1] + 1];
        let entry_end = offsets[ranks[2] + 1];

        let class_table = LookupTable::parse(&mut Stream::new(data.get(class_start..class_end)?))?;

        // 状態の数はエントリーを見てみないとわからないが、データ範囲が決まったので、一応、数がわかる。
        let mut state_stream = Stream::new(data.get(state_start..state_end)?);
        let mut state_array: Vec<Vec<u16>> = Vec::new();
        loop {
            let Some(state) = state_stream.read_array(stx_header.nClasses as usize) else {
                break
            };
            state_array.push(state);
        }
        // 状態0と状態1は確定している。
        assert!(state_array.len() >= 2);

        let mut entry_stream = Stream::new(data.get(entry_start..entry_end)?);
        let entry_table = entry_stream.read_all_array()?;

        Some(Self {
            header,
            stx_header,
            class_table,
            state_array,
            entry_table,
        })
    }
}

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct RearrangementEntry {
    pub newState: uint16, // Byte offset from beginning of state table to the new state
    pub flags: uint16,    // Table specific
}

impl FromData for RearrangementEntry {
    const SIZE: usize = uint16::SIZE * 2;
    #[allow(non_snake_case)]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let newState = s.read()?;
        let flags = s.read()?;
        assert_eq!(flags & Self::RESERVED, 0);
        Some(Self { newState, flags })
    }
}

impl RearrangementEntry {
    pub const MARK_FIRST: u16 = 0x8000; // If set, make the current glyph the first glyph to be rearranged.
    pub const DONT_ADVANCE: u16 = 0x4000; // If set, don't advance to the next glyph before going to the new state. This means that the glyph index doesn't change, even if the glyph at that index has changed.
    pub const MARK_LAST: u16 = 0x2000; // If set, make the current glyph the last glyph to be rearranged.
    pub const RESERVED: u16 = 0x1FF0; // These bits are reserved and should be set to 0.
    pub const VERB: u16 = 0x000F; // The type of rearrangement specified.
    pub fn is_mark_first(&self) -> bool {
        self.flags & Self::MARK_FIRST != 0
    }
    pub fn is_dont_advance(&self) -> bool {
        self.flags & Self::DONT_ADVANCE != 0
    }
    pub fn is_mark_last(&self) -> bool {
        self.flags & Self::MARK_LAST != 0
    }
    pub fn get_verb(&self) -> RearrangementVerb {
        let verb = (self.flags & Self::VERB) as u8;
        RearrangementVerb::try_from(verb).unwrap()
    }
    pub fn get_new_state_index(&self, number_of_classes: usize) -> usize {
        if number_of_classes == 0 {
            0
        } else {
            let size_of_state = u16::SIZE * number_of_classes;
            let new_state_byte_offset = self.newState as usize;
            // State の先頭のバイトオフセットなので、割り切れるはず。
            assert_eq!(new_state_byte_offset % size_of_state, 0);
            new_state_byte_offset / size_of_state
        }
    }
}

#[derive(Debug, TryFromPrimitive)]
#[repr(u8)]
#[allow(non_camel_case_types)]
pub enum RearrangementVerb {
    NoChange = 0,
    Ax_xA = 1,
    xD_Dx = 2,
    AxD_DxA = 3,
    ABx_xAB = 4,
    ABx_xBA = 5,
    xCD_CDx = 6,
    xCD_DCx = 7,
    AxCD_CDxA = 8,
    AxCD_DCxA = 9,
    ABxD_DxAB = 10,
    ABxD_DxBA = 11,
    ABxCD_CDxAB = 12,
    ABxCD_CDxBA = 13,
    ABxCD_DCxAB = 14,
    ABxCD_DCxBA = 15,
}

#[derive(Debug)]
pub struct MorxSubtableContextual<'a> {
    pub data: &'a [u8],
    pub header: MorxSubtableHeader,
    pub stx_header: STXHeader,
    // beginning of the state subtable というのは STXHeader の先頭のことである。
    pub substitution_table_offset: u32, // Byte offset from the beginning of the state subtable to the beginning of the substitution tables.
    pub class_table: LookupTable,
    pub state_array: Vec<Vec<u16>>,
}

impl<'a> MorxSubtableContextual<'a> {
    pub fn parse(header: MorxSubtableHeader, data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let stx_header: STXHeader = s.read()?;
        let substitution_table_offset = s.read()?;

        let class_start = stx_header.classTableOffset as usize;
        let state_start = stx_header.stateArrayOffset as usize;
        let entry_start = stx_header.entryTableOffset as usize;
        let substitution_start = substitution_table_offset as usize;
        let end = data.len();

        let offsets = [
            class_start,
            state_start,
            entry_start,
            substitution_start,
            end,
        ];
        let ranks = make_rank(&offsets);
        let class_end = offsets[ranks[0] + 1];
        let state_end = offsets[ranks[1] + 1];
        let entry_end = offsets[ranks[2] + 1];
        let substitution_end = offsets[ranks[3] + 1];

        let class_table = LookupTable::parse(&mut Stream::new(data.get(class_start..class_end)?))?;

        let mut state_stream = Stream::new(data.get(state_start..state_end)?);
        let mut state_array = Vec::new();
        loop {
            let Some(state) = state_stream.read_array(stx_header.nClasses as usize) else {
                break
            };
            state_array.push(state)
        }
        // 状態0と状態1は確定している。
        assert!(state_array.len() >= 2);

        Some(Self {
            data,
            header,
            stx_header,
            substitution_table_offset,
            class_table,
            state_array,
        })
    }
}

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct ContextualEntry {
    pub newState: uint16,     // Zero-based index to the new state
    pub flags: uint16,        // Table-specific flags.
    pub markIndex: uint16, // Index of the substitution table for the marked glyph (use 0xFFFF for none)
    pub currentIndex: uint16, // Index of the substitution table for the current glyph (use 0xFFFF for none)
}

impl FromData for ContextualEntry {
    const SIZE: usize = uint16::SIZE * 4;
    #[allow(non_snake_case)]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let newState = s.read()?;
        let flags = s.read()?;
        let markIndex = s.read()?;
        let currentIndex = s.read()?;
        assert_eq!(flags & Self::RESERVED, 0);
        Some(Self {
            newState,
            flags,
            markIndex,
            currentIndex,
        })
    }
}

impl ContextualEntry {
    pub const SET_MARK: u16 = 0x8000; // If set, make the current glyph the marked glyph.
    pub const DONT_ADVANCE: u16 = 0x4000; // If set, don't advance to the next glyph before going to the new state.
    pub const RESERVED: u16 = 0x3FFF; // These bits are reserved and should be set to 0.
    pub fn is_set_mark(&self) -> bool {
        self.flags & Self::SET_MARK != 0
    }
    pub fn is_dont_advance(&self) -> bool {
        self.flags & Self::DONT_ADVANCE != 0
    }
}

#[derive(Debug)]
pub struct MorxSubtableLigature<'a> {
    pub data: &'a [u8],
    pub header: MorxSubtableHeader,
    pub stx_header: STXHeader,
    pub ligature_action_table_offset: u32,
    pub component_table_offset: u32,
    pub ligature_list_offset: u32,
}

impl<'a> MorxSubtableLigature<'a> {
    pub fn parse(header: MorxSubtableHeader, data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        s.set_offset(MorxSubtableHeader::SIZE);

        let stx_header: STXHeader = s.read()?;
        let ligature_action_table_offset = s.read()?;
        let component_table_offset = s.read()?;
        let ligature_list_offset = s.read()?;

        Some(Self {
            data,
            header,
            stx_header,
            ligature_action_table_offset,
            component_table_offset,
            ligature_list_offset,
        })
    }
}

#[derive(Debug)]
pub struct MorxSubtableNoncontextual<'a> {
    pub data: &'a [u8],
    pub header: MorxSubtableHeader,
    pub lookup_table: LookupTable,
}

impl<'a> MorxSubtableNoncontextual<'a> {
    pub fn parse(header: MorxSubtableHeader, data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        s.set_offset(MorxSubtableHeader::SIZE);

        let lookup_table = LookupTable::parse(&mut s)?;

        Some(Self {
            data,
            header,
            lookup_table,
        })
    }
}

#[derive(Debug)]
pub enum LookupTable {
    Format0(SimpleArray),
    Format2(SegmentSingle),
    Format4(SegmentArray),
    Format6(SingleTable),
    Format8(TrimmedArray),
    Format10(ExtendedTrimmedArray),
}

impl LookupTable {
    pub fn parse(stream: &mut Stream<'_>) -> Option<Self> {
        let format: u16 = stream.read()?;
        match format {
            0 => Some(Self::Format0(SimpleArray::parse(stream)?)),
            2 => Some(Self::Format2(SegmentSingle::parse(stream)?)),
            4 => Some(Self::Format4(SegmentArray::parse(stream)?)),
            6 => Some(Self::Format6(SingleTable::parse(stream)?)),
            8 => Some(Self::Format8(TrimmedArray::parse(stream)?)),
            10 => Some(Self::Format10(ExtendedTrimmedArray::parse(stream)?)),
            _ => panic!("invalid lookup table format {}", format),
        }
    }
}

#[derive(Debug)]
pub struct SimpleArray {
    pub lookup_values: Vec<uint16>, // XXX: morx のルックアップ値は 16 bitであるが、他はそうとは限らない。
}

impl SimpleArray {
    /// stream の最後まで読み込む。
    fn parse(stream: &mut Stream<'_>) -> Option<Self> {
        let lookup_values = stream.read_all_array()?;
        Some(Self { lookup_values })
    }
}

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct SegmentSingle {
    pub binSrchHeader: BinSrchHeader, // The units for this binary search are of type LookupSegment, and always have a minimum length of 6.
    pub segments: Vec<Format2LookupSegment>, // The actual segments. These must already be sorted, according to the first word in each one (the last glyph in each segment).
}
impl SegmentSingle {
    #[allow(non_snake_case)]
    fn parse(stream: &mut Stream<'_>) -> Option<Self> {
        let binSrchHeader: BinSrchHeader = stream.read()?;
        // XXX: morx のルックアップ値は 16 bitなので unitSize が決まるが、他はそうとは限らない。
        assert_eq!(binSrchHeader.unitSize, 6);
        let mut segments = Vec::new();
        for _ in 0..binSrchHeader.nUnits {
            segments.push(stream.read()?);
        }
        Some(Self {
            binSrchHeader,
            segments,
        })
    }
}

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct Format2LookupSegment {
    pub lastGlyph: uint16,  // Last glyph index in this segment
    pub firstGlyph: uint16, // First glyph index in this segment
    // XXX: morx のルックアップ値は 16 bitであるが、他はそうとは限らない。
    pub value: uint16, // The lookup value (only one)
}

impl FromData for Format2LookupSegment {
    const SIZE: usize = uint16::SIZE * 3;
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(Self {
            lastGlyph: s.read()?,
            firstGlyph: s.read()?,
            value: s.read()?,
        })
    }
}

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct SegmentArray {
    pub binSrchHeader: BinSrchHeader, // The units for this binary search are of type LookupSegment and always have a minimum length of 6.
    pub segments: Vec<Format4LookupSegment>, // The actual segments. These must already be sorted, according to the first word in each one (the last glyph in each segment).
    pub lookup_values: Vec<uint16>, // XXX: morx のルックアップ値は 16 bitであるが、他はそうとは限らない。
}
impl SegmentArray {
    #[allow(non_snake_case)]
    fn parse(stream: &mut Stream<'_>) -> Option<Self> {
        let start = stream.get_offset();
        let binSrchHeader: BinSrchHeader = stream.read()?;
        assert_eq!(binSrchHeader.unitSize as usize, Format4LookupSegment::SIZE);
        let mut segments: Vec<Format4LookupSegment> = Vec::new();
        for _ in 0..binSrchHeader.nUnits {
            segments.push(stream.read()?);
        }

        let mut lookup_values = Vec::new();
        for segment in &segments {
            stream.set_offset(start + segment.value as usize);
            lookup_values.push(stream.read()?);
        }

        Some(Self {
            binSrchHeader,
            segments,
            lookup_values,
        })
    }
}

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct Format4LookupSegment {
    pub lastGlyph: uint16,  // Last glyph index in this segment
    pub firstGlyph: uint16, // First glyph index in this segment
    pub value: uint16,      // A 16-bit offset from the start of the table to the data
}

impl FromData for Format4LookupSegment {
    const SIZE: usize = uint16::SIZE * 3;
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(Self {
            lastGlyph: s.read()?,
            firstGlyph: s.read()?,
            value: s.read()?,
        })
    }
}

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct SingleTable {
    pub binSrchHeader: BinSrchHeader, // The units for this binary search are of type LookupSingle and always have a minimum length of 4.
    pub entries: Vec<Format6LookupSingle>, // The actual entries, sorted by glyph index.
}
impl SingleTable {
    #[allow(non_snake_case)]
    fn parse(stream: &mut Stream<'_>) -> Option<Self> {
        let binSrchHeader: BinSrchHeader = stream.read()?;
        // XXX: morx のルックアップ値は 16 bitなので unitSize が決まるが、他はそうとは限らない。
        assert_eq!(binSrchHeader.unitSize, 4);
        let mut entries = Vec::new();
        for _ in 0..binSrchHeader.nUnits {
            entries.push(stream.read()?);
        }
        Some(Self {
            binSrchHeader,
            entries,
        })
    }
}
#[derive(Debug)]
pub struct Format6LookupSingle {
    pub glyph: uint16, // The glyph index
    // XXX: morx のルックアップ値は 16 bitであるが、他はそうとは限らない。
    pub value: uint16, // The lookup value
}

impl FromData for Format6LookupSingle {
    const SIZE: usize = uint16::SIZE * 2;
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(Self {
            glyph: s.read()?,
            value: s.read()?,
        })
    }
}

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct TrimmedArray {
    pub firstGlyph: uint16, // First glyph index included in the trimmed array.
    pub glyphCount: uint16, // Total number of glyphs (equivalent to the last glyph minus the value of firstGlyph plus 1).
    // XXX: TrimmedArray のルックアップ値は 2 bytes なので u16 や i16 などであり、ここでは u16 とする。
    pub valueArray: Vec<uint16>, // The lookup values (indexed by the glyph index minus the value of firstGlyph). Entries in the value array must be two bytes.
}
impl TrimmedArray {
    #[allow(non_snake_case)]
    fn parse(stream: &mut Stream<'_>) -> Option<Self> {
        let firstGlyph = stream.read()?;
        let glyphCount = stream.read()?;
        let mut valueArray = Vec::new();
        for _ in 0..glyphCount {
            valueArray.push(stream.read()?);
        }
        Some(Self {
            firstGlyph,
            glyphCount,
            valueArray,
        })
    }
}

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct ExtendedTrimmedArray {
    pub unitSize: uint16, // Size of a lookup unit for this lookup table in bytes. Allowed values are 1, 2, 4, and 8.
    pub firstGlyph: uint16, // First glyph index included in the trimmed array.
    pub glyphCount: uint16, // Total number of glyphs (equivalent to the last glyph minus the value of firstGlyph plus 1).
    // XXX: ルックアップ値のサイズは unitSize で確認しないといけないのだが、morx だと 16 bit なので u16 とする。
    pub valueArray: Vec<uint16>, // The lookup values (indexed by the glyph index minus the value of firstGlyph).
}
impl ExtendedTrimmedArray {
    #[allow(non_snake_case)]
    fn parse(stream: &mut Stream<'_>) -> Option<Self> {
        let unitSize = stream.read()?;
        assert_eq!(unitSize, 2);
        let firstGlyph = stream.read()?;
        let glyphCount = stream.read()?;
        let mut valueArray = Vec::new();
        for _ in 0..glyphCount {
            valueArray.push(stream.read()?);
        }

        Some(Self {
            unitSize,
            firstGlyph,
            glyphCount,
            valueArray,
        })
    }
}

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct BinSrchHeader {
    pub unitSize: uint16,      // Size of a lookup unit for this search in bytes.
    pub nUnits: uint16,        // Number of units of the preceding size to be searched.
    pub searchRange: uint16, // The value of unitSize times the largest power of 2 that is less than or equal to the value of nUnits.
    pub entrySelector: uint16, // The log base 2 of the largest power of 2 less than or equal to the value of nUnits.
    pub rangeShift: uint16, // The value of unitSize times the difference of the value of nUnits minus the largest power of 2 less than or equal to the value of nUnits.
}

impl FromData for BinSrchHeader {
    const SIZE: usize = uint16::SIZE * 5;
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(Self {
            unitSize: s.read()?,
            nUnits: s.read()?,
            searchRange: s.read()?,
            entrySelector: s.read()?,
            rangeShift: s.read()?,
        })
    }
}

#[derive(Debug)]
pub struct MorxSubtableInsertion<'a> {
    pub data: &'a [u8],
    pub header: MorxSubtableHeader,
    pub stx_header: STXHeader,
    pub insertion_action_table_offset: u32,
}

impl<'a> MorxSubtableInsertion<'a> {
    pub fn parse(header: MorxSubtableHeader, data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        s.set_offset(MorxSubtableHeader::SIZE);

        let stx_header: STXHeader = s.read()?;
        let insertion_action_table_offset = s.read()?;
        Some(Self {
            data,
            header,
            stx_header,
            insertion_action_table_offset,
        })
    }
}
#[derive(Debug)]
#[allow(non_snake_case)]
pub struct MorxSubtableHeader {
    pub length: uint32,                // Total subtable length, including this header.
    pub coverage: uint32,              // Coverage flags and subtable type.
    pub subFeatureFlags: FeatureFlags, // The 32-bit mask identifying which subtable this is (the subtable being executed if the AND of this value and the processed defaultFlags is nonzero)
}

impl FromData for MorxSubtableHeader {
    const SIZE: usize = uint32::SIZE * 3;
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(Self {
            length: s.read()?,
            coverage: s.read()?,
            subFeatureFlags: s.read()?,
        })
    }
}

pub enum MorxSubtableType {
    Rearrangement = 0,
    Contextual = 1,
    Ligature = 2,
    Noncontextual = 4,
    Insertion = 5,
}

impl fmt::Debug for MorxSubtableType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rearrangement => write!(f, "0 (Rearrangement)"),
            Self::Contextual => write!(f, "1 (Contextual)"),
            Self::Ligature => write!(f, "2 (Ligature)"),
            Self::Noncontextual => write!(f, "4 (Noncontextual)"),
            Self::Insertion => write!(f, "5 (Insertion)"),
        }
    }
}

impl MorxSubtableHeader {
    pub fn get_type(&self) -> MorxSubtableType {
        match self.coverage & 0xFF {
            0 => MorxSubtableType::Rearrangement,
            1 => MorxSubtableType::Contextual,
            2 => MorxSubtableType::Ligature,
            4 => MorxSubtableType::Noncontextual,
            5 => MorxSubtableType::Insertion,
            _ => {
                panic!("invalid morx subtable type");
            }
        }
    }
}

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct STXHeader {
    pub nClasses: uint32, // Number of classes, which is the number of 16-bit entry indices in a single line in the state array.
    pub classTableOffset: uint32, // Offset from the start of this state table header to the start of the class table.
    pub stateArrayOffset: uint32, // Offset from the start of this state table header to the start of the state array.
    pub entryTableOffset: uint32, // Offset from the start of this state table header to the start of the entry table.
}

impl FromData for STXHeader {
    const SIZE: usize = uint32::SIZE * 4;
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(Self {
            nClasses: s.read()?,
            classTableOffset: s.read()?,
            stateArrayOffset: s.read()?,
            entryTableOffset: s.read()?,
        })
    }
}

// Glyph id -> Class lookup table -> Class
//
//          Class 0    , Class 1    , ...
// State 0 [Entry index, Entry index, ...]
// State 1 [Entry index, Entry index, ...]
// State 2 [Entry index, Entry index, ...]
// ...
//
// Entry [0]: Next state (byte offset), Action flags, Option info
// Entry [1]: Next state (byte offset), Action flags, Option info
// ...

#[derive(Debug)]
pub struct ExtendedStateTable {}

impl ExtendedStateTable {
    pub fn get_class(_glyph_id: &u16) -> u16 {
        todo!()
    }
}

// 有限状態機械。
// 現在の状態を持つ。
pub struct FiniteStateMachine {
    pub current_state: uint16,
}

// 拡張状態テーブルのクラス テーブルは単純な LookupTable になり、ルックアップ値は 16 ビットのクラス値

/// 順位付け。
fn make_rank(values: &[usize]) -> Vec<usize> {
    let mut ranks = Vec::new();
    for value in values {
        let rank: usize = values.iter().fold(0, |accumulator, part| {
            accumulator + (if value > part { 1 } else { 0 })
        });
        ranks.push(rank);
    }
    ranks
}

#[cfg(test)]
mod tests {
    use crate::morx::make_rank;

    #[test]
    fn test_rank() {
        assert_eq!(make_rank(&[0, 2, 6]), vec![0, 1, 2]);
        assert_eq!(make_rank(&[0, 6, 2]), vec![0, 2, 1]);
        assert_eq!(make_rank(&[2, 0, 6]), vec![1, 0, 2]);
        assert_eq!(make_rank(&[2, 6, 0]), vec![1, 2, 0]);
        assert_eq!(make_rank(&[6, 0, 2]), vec![2, 0, 1]);
        assert_eq!(make_rank(&[6, 2, 0]), vec![2, 1, 0]);
    }
}
