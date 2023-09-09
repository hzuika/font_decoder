use std::ops::Range;

use crate::{
    data_types::{int16, uint16, uint8},
    decoder::{FromData, Stream},
};

pub struct GlyfTable<'a>(pub &'a [u8]);

impl<'a> GlyfTable<'a> {
    pub fn get_data(&self, range: Range<usize>) -> Option<&'a [u8]> {
        self.0.get(range)
    }
}

pub struct Glyph {
    pub header: GlyphHeader,
    pub subtable: GlyphTable,
}

impl Glyph {
    pub fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let header: GlyphHeader = s.read()?;
        match header.get_type() {
            GlyphType::Simple => {
                let subtable =
                    SimpleGlyphTable::parse(s.get_tail()?, header.numberOfContours as u16)?;
                Some(Glyph {
                    header,
                    subtable: GlyphTable::Simple(subtable),
                })
            }
            GlyphType::Composite => {
                todo!()
            }
        }
    }
}

pub enum GlyphTable {
    Simple(SimpleGlyphTable),
    Composite,
}

impl<'a> IntoIterator for &'a GlyphTable {
    type IntoIter = GlyphPointsIter<'a>;
    type Item = GlyphPoint;
    fn into_iter(self) -> Self::IntoIter {
        match self {
            GlyphTable::Simple(table) => GlyphPointsIter { table, index: 0 },
            GlyphTable::Composite => {
                todo!()
            }
        }
    }
}

#[allow(non_snake_case)]
pub struct GlyphHeader {
    pub numberOfContours: int16, // If the number of contours is greater than or equal to zero, this is a simple glyph. If negative, this is a composite glyph — the value -1 should be used for composite glyphs.
    pub xMin: int16,             // Minimum x for coordinate data.
    pub yMin: int16,             // Minimum y for coordinate data.
    pub xMax: int16,             // Maximum x for coordinate data.
    pub yMax: int16,             // Maximum y for coordinate data.
}

impl FromData for GlyphHeader {
    const SIZE: usize = 2 * 5;
    #[allow(non_snake_case)]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let numberOfContours = s.read()?;
        let xMin = s.read()?;
        let yMin = s.read()?;
        let xMax = s.read()?;
        let yMax = s.read()?;
        Some(Self {
            numberOfContours,
            xMin,
            yMin,
            xMax,
            yMax,
        })
    }
}

pub enum GlyphType {
    Simple,
    Composite,
}

impl GlyphHeader {
    pub fn get_type(&self) -> GlyphType {
        if self.numberOfContours >= 0 {
            GlyphType::Simple
        } else {
            GlyphType::Composite
        }
    }
}

#[allow(non_snake_case)]
pub struct SimpleGlyphTable {
    pub endPtsOfContours: Vec<uint16>, //[numberOfContours]Array of point indices for the last point of each contour, in increasing numeric order.
    pub instructionLength: uint16, //Total number of bytes for instructions. If instructionLength is zero, no instructions are present for this glyph, and this field is followed directly by the flags field.
    pub instructions: Vec<uint8>, //[instructionLength]Array of instruction byte code for the glyph.
    pub flags: Vec<SimpleGlyphFlags>, // flatten flags. uint8 flags[variable]Array of flag elements. See below for details regarding the number of flag array elements.
    pub xCoordinates: Vec<i16>, // uint8 or int16 xCoordinates[variable]Contour point x-coordinates. See below for details regarding the number of coordinate array elements. Coordinate for the first point is relative to (0,0); others are relative to previous point.
    pub yCoordinates: Vec<i16>, // uint8 or int16 yCoordinates[variable]Contour point y-coordinates. See below for details regarding the number of coordinate array elements. Coordinate for the first point is relative to (0,0); others are relative to previous point.
}

impl SimpleGlyphTable {
    #[allow(non_snake_case)]
    pub fn parse(data: &[u8], number_of_contours: u16) -> Option<Self> {
        let mut s = Stream::new(data);
        let endPtsOfContours: Vec<u16> = s.read_array(number_of_contours as usize)?;
        let number_of_points = endPtsOfContours.last()?.checked_add(1)?;
        let instructionLength = s.read()?;
        let instructions = s.read_array(instructionLength as usize)?;

        // flatten flags -> group by contour.
        let mut flags_left = number_of_points;
        let mut flags = vec![];
        while flags_left > 0 {
            let flag = s.read::<SimpleGlyphFlags>()?;

            let repeat = 1 + if flag.repeat_flag() {
                s.read::<u8>()? as u16
            } else {
                0
            };

            for _ in 0..repeat {
                flags.push(flag);
            }

            flags_left -= repeat;
        }

        assert_eq!(flags.len(), number_of_points as usize);

        let mut prev: i16 = 0;
        let mut xCoordinates = vec![];
        for flag in &flags {
            let delta = match flag.get_x_type() {
                CoordType::Positive8 => i16::from(s.read::<u8>()?),
                CoordType::Negative8 => -i16::from(s.read::<u8>()?),
                CoordType::I16 => s.read::<i16>()?,
                CoordType::SamePrevious => 0,
            };
            prev = prev.wrapping_add(delta);
            xCoordinates.push(prev);
        }

        let mut prev: i16 = 0;
        let mut yCoordinates = vec![];
        for flag in &flags {
            let delta = match flag.get_y_type() {
                CoordType::Positive8 => i16::from(s.read::<u8>()?),
                CoordType::Negative8 => -i16::from(s.read::<u8>()?),
                CoordType::I16 => s.read::<i16>()?,
                CoordType::SamePrevious => 0,
            };
            prev = prev.wrapping_add(delta);
            yCoordinates.push(prev);
        }

        Some(Self {
            endPtsOfContours,
            instructionLength,
            instructions,
            flags,
            xCoordinates,
            yCoordinates,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GlyphPoint {
    pub x: i16,
    pub y: i16,
    pub flags: SimpleGlyphFlags,
    pub is_last: bool,
}

// TODO: GlyphContoursIter にしてもいいかもしれない．
#[derive(Clone)]
pub struct GlyphPointsIter<'a> {
    table: &'a SimpleGlyphTable,
    index: usize,
}

impl<'a> Iterator for GlyphPointsIter<'a> {
    type Item = GlyphPoint;

    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;
        let index = self.index - 1;

        let x = *self.table.xCoordinates.get(index)?;
        let y = *self.table.yCoordinates.get(index)?;
        let flags = *self.table.flags.get(index)?;
        let is_last = self
            .table
            .endPtsOfContours
            .iter()
            .find(|&&x| x as usize == index)
            .is_some();

        Some(GlyphPoint {
            x,
            y,
            flags,
            is_last,
        })
    }
}

pub enum CoordType {
    Positive8,
    Negative8,
    SamePrevious,
    I16,
}

#[derive(Clone, Copy, Debug)]
pub struct SimpleGlyphFlags(pub u8);
impl FromData for SimpleGlyphFlags {
    const SIZE: usize = 1;
    fn parse(data: &[u8]) -> Option<Self> {
        u8::parse(data).map(Self)
    }
}

impl SimpleGlyphFlags {
    pub const ON_CURVE_POINT: u8 = 0x01;
    pub const X_SHORT_VECTOR: u8 = 0x02;
    pub const Y_SHORT_VECTOR: u8 = 0x04;
    pub const REPEAT_FLAG: u8 = 0x08;
    pub const X_IS_SAME_OR_POSITIVE_X_SHORT_VECTOR: u8 = 0x10;
    pub const Y_IS_SAME_OR_POSITIVE_Y_SHORT_VECTOR: u8 = 0x20;

    #[inline]
    pub fn is_1byte_x(&self) -> bool {
        self.0 & Self::X_SHORT_VECTOR != 0
    }

    #[inline]
    pub fn is_1byte_y(&self) -> bool {
        self.0 & Self::Y_SHORT_VECTOR != 0
    }

    pub fn is_on_curve_point(&self) -> bool {
        self.0 & Self::ON_CURVE_POINT != 0
    }

    pub fn get_x_type(&self) -> CoordType {
        if self.is_1byte_x() {
            if self.0 & Self::X_IS_SAME_OR_POSITIVE_X_SHORT_VECTOR != 0 {
                CoordType::Positive8
            } else {
                CoordType::Negative8
            }
        } else {
            if self.0 & Self::X_IS_SAME_OR_POSITIVE_X_SHORT_VECTOR != 0 {
                CoordType::SamePrevious
            } else {
                CoordType::I16
            }
        }
    }

    pub fn get_y_type(&self) -> CoordType {
        if self.is_1byte_y() {
            if self.0 & Self::Y_IS_SAME_OR_POSITIVE_Y_SHORT_VECTOR != 0 {
                CoordType::Positive8
            } else {
                CoordType::Negative8
            }
        } else {
            if self.0 & Self::Y_IS_SAME_OR_POSITIVE_Y_SHORT_VECTOR != 0 {
                CoordType::SamePrevious
            } else {
                CoordType::I16
            }
        }
    }

    // true のとき，このバイトの次のバイト(u8)の数だけ，このバイトを足す．
    // 例: [f,3] → [f,f,f,f] のように展開される．
    pub fn repeat_flag(&self) -> bool {
        self.0 & Self::REPEAT_FLAG != 0
    }
}
