use std::collections::HashMap;

use crate::{
    data_types::{int16, uint16, Offset32},
    decoder::{FromData, LazyArray, Stream},
};

#[allow(non_snake_case)]
pub struct CmapHeader<'a> {
    pub version: uint16,                                // Table version number (0).
    pub numTables: uint16,                              // Number of encoding tables that follow.
    pub encodingRecords: LazyArray<'a, EncodingRecord>, // [numTables]
}

impl<'a> CmapHeader<'a> {
    #[allow(non_snake_case)]
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let version = s.read()?;
        let numTables = s.read()?;
        let encodingRecords = s.read_array(numTables as usize)?;
        Some(Self {
            version,
            numTables,
            encodingRecords,
        })
    }
}

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct EncodingRecord {
    pub platformID: uint16,       // Platform ID.
    pub encodingID: uint16,       // Platform-specific encoding ID.
    pub subtableOffset: Offset32, // Byte offset from beginning of table to the subtable for this encoding.
}

impl FromData for EncodingRecord {
    const SIZE: usize = 4 + 4;
    #[allow(non_snake_case)]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let platformID = s.read()?;
        let encodingID = s.read()?;
        let subtableOffset = s.read()?;
        Some(Self {
            platformID,
            encodingID,
            subtableOffset,
        })
    }
}

pub enum CmapSubtable<'a> {
    Format0,
    Format2,
    Format4(CmapSubtableFormat4<'a>),
    Format6,
    Format8,
    Format10,
    Format12,
    Format13,
    Format14,
}

impl<'a> CmapSubtable<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let format: u16 = s.read()?;
        match format {
            4 => Some(Self::Format4(CmapSubtableFormat4::parse(data)?)),
            _ => None,
        }
    }

    pub fn get_glyph_id(&self, code_point: u16) -> Option<u16> {
        match self {
            Self::Format4(x) => x.get_glyph_id(code_point),
            _ => todo!(),
        }
    }

    // TODO: Iterator
    pub fn get_code_point_glyph_id_map(&self) -> HashMap<u16, u16> {
        match self {
            Self::Format4(x) => x.get_code_point_glyph_id_map(),
            _ => todo!(),
        }
    }
}

#[allow(non_snake_case)]
pub struct CmapSubtableFormat4<'a> {
    pub format: uint16,                        // Format number is set to 4.
    pub length: uint16,                        // This is the length in bytes of the subtable.
    pub language: uint16, // For requirements on use of the language field, see “Use of the language field in 'cmap' subtables” in this document.
    pub segCountX2: uint16, // 2 × segCount.
    pub searchRange: uint16, // Maximum power of 2 less than or equal to segCount, times 2 ((2**floor(log2(segCount))) * 2, where “**” is an exponentiation operator)
    pub entrySelector: uint16, // Log2 of the maximum power of 2 less than or equal to segCount (log2(searchRange/2), which is equal to floor(log2(segCount)))
    pub rangeShift: uint16,    // segCount times 2, minus searchRange ((segCount * 2) - searchRange)
    pub endCode: LazyArray<'a, uint16>, // [segCount] End characterCode for each segment, last=0xFFFF.
    pub reservedPad: uint16,            // Set to 0.
    pub startCode: LazyArray<'a, uint16>, // [segCount] Start character code for each segment.
    pub idDelta: LazyArray<'a, int16>,  // [segCount] Delta for all character codes in segment.
    pub idRangeOffsets: LazyArray<'a, uint16>, // [segCount] Offsets into glyphIdArray or 0
    pub glyphIdArray: LazyArray<'a, uint16>, // [ ] Glyph index array (arbitrary length)
}

impl<'a> CmapSubtableFormat4<'a> {
    #[allow(non_snake_case)]
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let format = s.read()?;
        let length = s.read()?;
        s.set_len(length as usize);
        let language = s.read()?;
        let segCountX2 = s.read()?;
        let segCount = (segCountX2 / 2) as usize;
        let searchRange = s.read()?;
        let entrySelector = s.read()?;
        let rangeShift = s.read()?;
        let endCode = s.read_array(segCount)?;
        let reservedPad = s.read()?;
        let startCode = s.read_array(segCount)?;
        let idDelta = s.read_array(segCount)?;
        let idRangeOffsets = s.read_array(segCount)?;
        let glyphIdArray = LazyArray::new(s.tail()?);
        Some(Self {
            format,
            length,
            language,
            segCountX2,
            searchRange,
            entrySelector,
            rangeShift,
            endCode,
            reservedPad,
            startCode,
            idDelta,
            idRangeOffsets,
            glyphIdArray,
        })
    }

    pub fn get_glyph_id(&self, code_point: u16) -> Option<u16> {
        let mut start_index = 0;
        let mut end_index = self.startCode.len(); // segCount?
        while end_index > start_index {
            let mid_index = (start_index + end_index) / 2;
            let end_code_point = self.endCode.get(mid_index)?;
            if end_code_point < code_point {
                // [... , mid_index, start_index, ..., end_index]
                start_index = mid_index + 1;
                continue;
            }
            let start_code_point = self.startCode.get(mid_index)?;
            if code_point < start_code_point {
                // [start_index, ... , end_index = mid_index, ...]
                end_index = mid_index;
                continue;
            }

            // start_code_point <= code_point <= end_code_point

            let id_range_offset = self.idRangeOffsets.get(mid_index)?;
            let id_delta = self.idDelta.get(mid_index)?;
            if id_range_offset == 0 {
                // 2の補数表現を使っているから，negative i16 を u16 と解釈してオーバフロー分を無視して加算すれば減算と同じ．
                // 例: FFFF (= -1) + 0001 (= 1) = 0
                return Some(code_point.wrapping_add(id_delta as u16));
            }

            // id_range_offset が 0 で無い場合は，mid_indexの場所から id_range_offset の分だけオフセットした位置の glyph_id_array を取得する．

            // indices of idRangeOffsets [..., mid_index, ..., len() - 1]
            // → Example (mid_index == 2, id_range_offset = idRangeOffsets[2] == 12, idRangeOffsets.len() == 6)
            // indices of idRangeOffsets [0, 1,        2, 3, 4,        5]
            // idRangeOffsets[u16]       [ ,  ,       12,  ,  ,         ]
            // current pointer                         ↑
            // indices of glyphIdArray                                   [ 0,  1,  2,  3, ...]
            // glyphIdArray[u16]                                         [15, 16, 20, 21, ...]
            // current pointer + (idRangeOffsets.len() - mid_index)        ↑
            // current pointer + (id_range_offset / 2)                             ↑
            let glyph_id_array_index =
                (id_range_offset / 2) - (self.idRangeOffsets.len() - mid_index) as u16;
            return Some(self.glyphIdArray.get(glyph_id_array_index as usize)?);
        }
        return Some(0); // notdef.
    }

    pub fn get_code_point_glyph_id_map(&self) -> HashMap<u16, u16> {
        let mut map = HashMap::new();
        for (i, start_code_point) in self.startCode.into_iter().enumerate() {
            let end_code_point = self.endCode.get(i).unwrap();
            let id_delta = self.idDelta.get(i).unwrap();
            let id_range_offset = self.idRangeOffsets.get(i).unwrap();
            for code_point in start_code_point..=end_code_point {
                let glyph_id = if id_range_offset == 0 {
                    code_point.wrapping_add(id_delta as u16)
                } else {
                    let glyph_id_array_index =
                        (id_range_offset as usize / 2) - (self.idRangeOffsets.len() - i);
                    self.glyphIdArray.get(glyph_id_array_index).unwrap()
                };
                map.insert(code_point, glyph_id);
            }
        }
        map
    }
}

pub struct CmapTable<'a> {
    data: &'a [u8],
    pub header: CmapHeader<'a>,
}

impl<'a> CmapTable<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let header = CmapHeader::parse(data)?;
        Some(Self { data, header })
    }

    pub fn get_subtable(&self, encoding_record: &EncodingRecord) -> Option<CmapSubtable> {
        let offset = encoding_record.subtableOffset as usize;
        let data = self.data.get(offset..)?;
        CmapSubtable::parse(data)
    }
}
