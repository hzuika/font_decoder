use std::ops::Range;

use crate::{
    data_types::{Offset16, Offset32},
    decoder::Stream,
    head::LocaOffsetFormat,
};

pub enum LocaTable {
    Short(Vec<Offset16>), // [n] The actual local offset divided by 2 is stored. The value of n is numGlyphs + 1. The value for numGlyphs is found in the 'maxp' table.
    Long(Vec<Offset32>), // [n] The actual local offset is stored. The value of n is numGlyphs + 1. The value for numGlyphs is found in the 'maxp' table.
}

impl LocaTable {
    pub fn parse(data: &[u8], format: LocaOffsetFormat, num_glyphs: u16) -> Option<Self> {
        let mut s = Stream::new(data);
        match format {
            LocaOffsetFormat::Offset16 => {
                let offsets = s.read_array(num_glyphs as usize + 1)?;
                Some(Self::Short(offsets))
            }
            LocaOffsetFormat::Offset32 => {
                let offsets = s.read_array(num_glyphs as usize + 1)?;
                Some(Self::Long(offsets))
            }
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Short(offsets) => offsets.len(),
            Self::Long(offsets) => offsets.len(),
        }
    }

    pub fn get_glyf_range(&self, glyph_id: u16) -> Option<Range<usize>> {
        let glyph_id = glyph_id as usize;
        let next_glyph_id = glyph_id + 1;
        if next_glyph_id >= self.len() {
            return None;
        }

        let range = match self {
            Self::Short(offsets) => {
                let start = *offsets.get(glyph_id)? as usize * 2;
                let end = *offsets.get(next_glyph_id)? as usize * 2;
                start..end
            }
            Self::Long(offsets) => {
                let start = *offsets.get(glyph_id)? as usize * 2;
                let end = *offsets.get(next_glyph_id)? as usize * 2;
                start..end
            }
        };

        if range.start >= range.end {
            None
        } else {
            Some(range)
        }
    }
}
