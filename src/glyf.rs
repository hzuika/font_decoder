use std::ops::Range;

use crate::{
    data_types::{int16, uint16, uint8},
    decoder::{FromData, LazyArray, Stream},
};

pub struct GlyfTable<'a>(pub &'a [u8]);

impl<'a> GlyfTable<'a> {
    pub fn get_data(&self, range: Range<usize>) -> Option<&'a [u8]> {
        self.0.get(range)
    }
}

pub struct Glyph<'a> {
    pub header: GlyphHeader,
    pub subtable: GlyphSubtable<'a>,
}

impl<'a> Glyph<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let header: GlyphHeader = s.read()?;
        match header.get_type() {
            GlyphType::Simple => {
                let subtable = SimpleGlyphTable::parse(s.tail()?, header.numberOfContours as u16)?;
                Some(Glyph {
                    header,
                    subtable: GlyphSubtable::Simple(subtable),
                })
            }
            GlyphType::Composite => {
                todo!()
            }
        }
    }
}

pub enum GlyphSubtable<'a> {
    Simple(SimpleGlyphTable<'a>),
    Composite,
}

impl<'a> GlyphSubtable<'a> {
    pub fn get_glyph_points_iter(&self) -> &GlyphPointsIter<'a> {
        match self {
            Self::Simple(table) => &table.glyph_points_iter,
            Self::Composite => todo!(),
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
pub struct SimpleGlyphTable<'a> {
    pub endPtsOfContours: LazyArray<'a, uint16>, //[numberOfContours]Array of point indices for the last point of each contour, in increasing numeric order.
    pub instructionLength: uint16, //Total number of bytes for instructions. If instructionLength is zero, no instructions are present for this glyph, and this field is followed directly by the flags field.
    pub instructions: LazyArray<'a, uint8>, //[instructionLength]Array of instruction byte code for the glyph.
    pub glyph_points_iter: GlyphPointsIter<'a>,
    // flags, xCoordinates, yCoordinates をまとめて GlyphPointsIter で表す．
    // uint8 flags[variable]Array of flag elements. See below for details regarding the number of flag array elements.
    // uint8 or int16 xCoordinates[variable]Contour point x-coordinates. See below for details regarding the number of coordinate array elements. Coordinate for the first point is relative to (0,0); others are relative to previous point.
    // uint8 or int16 yCoordinates[variable]Contour point y-coordinates. See below for details regarding the number of coordinate array elements. Coordinate for the first point is relative to (0,0); others are relative to previous point.
}

impl<'a> SimpleGlyphTable<'a> {
    #[allow(non_snake_case)]
    pub fn parse(data: &'a [u8], number_of_contours: u16) -> Option<Self> {
        let mut s = Stream::new(data);
        let endPtsOfContours: LazyArray<'_, u16> = s.read_array(number_of_contours as usize)?;
        let number_of_points = endPtsOfContours.last()?.checked_add(1)?;
        let instructionLength = s.read()?;
        let instructions = s.read_array(instructionLength as usize)?;
        let flags_offset = s.get_offset();
        let (x_coords_len, y_coords_len) = get_coords_len(&mut s, number_of_points as usize)?;
        let x_coords_offset = s.get_offset();
        let y_coords_offset = x_coords_offset + x_coords_len;
        let y_coords_end = y_coords_offset + y_coords_len;

        let flags = FlagsIter::new(data.get(flags_offset..x_coords_offset)?);
        let x_coords = CoordsIter::new(data.get(x_coords_offset..y_coords_offset)?);
        let y_coords = CoordsIter::new(data.get(y_coords_offset..y_coords_end)?);
        let glyph_points_iter = GlyphPointsIter {
            endpoints: EndPointsIter::new(endPtsOfContours)?,
            flags,
            x_coords,
            y_coords,
            points_left: number_of_points,
        };

        Some(Self {
            endPtsOfContours,
            instructionLength,
            instructions,
            glyph_points_iter,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GlyphPoint {
    pub x: i16,
    pub y: i16,
    pub is_on_curve: bool,
    pub is_last: bool,
}

// TODO: GlyphContoursIter にしてもいいかもしれない．
#[derive(Clone)]
pub struct GlyphPointsIter<'a> {
    endpoints: EndPointsIter<'a>,
    flags: FlagsIter<'a>,
    x_coords: CoordsIter<'a>,
    y_coords: CoordsIter<'a>,
    pub points_left: u16, // グリフ内の残りの点の数．
}

impl<'a> Iterator for GlyphPointsIter<'a> {
    type Item = GlyphPoint;

    fn next(&mut self) -> Option<Self::Item> {
        self.points_left = self.points_left.checked_sub(1)?;

        let is_last = self.endpoints.next();
        let flags = self.flags.next()?;
        let x = self.x_coords.next(flags.get_x_type());
        let y = self.y_coords.next(flags.get_y_type());
        let is_on_curve = flags.is_on_curve_point();
        Some(GlyphPoint {
            x,
            y,
            is_on_curve,
            is_last,
        })
    }
}

// 輪郭の終点を指し示すイテレータ．
// 終点が           2   4     7 の場合
// 終点フラグを 0 0 1 0 1 0 0 1 のように立てる．
#[derive(Clone)]
struct EndPointsIter<'a> {
    endpoints: LazyArray<'a, u16>,
    index: u16,
    left: u16, // endpoint までの残り．0 になると，endpointになる．
}

impl<'a> EndPointsIter<'a> {
    fn new(end_points_of_contours: LazyArray<'a, u16>) -> Option<Self> {
        Some(EndPointsIter {
            endpoints: end_points_of_contours,
            index: 1, // 最初の終点の位置を取得するので，開始インデックスは 1 になる．
            left: end_points_of_contours.get(0)?, // 最初の終点の位置を取得しておく必要がある．
        })
    }

    fn next(&mut self) -> bool {
        if self.left == 0 {
            if let Some(endpoint) = self.endpoints.get(self.index as usize) {
                // 現在の endpoint と 前の endpoint の間にある数だけ false を返すように self.left を設定する．
                // 終点            2             4
                // フラグ 0 0 prev_endpoint 0 endpoint
                let prev_endpoint = self.endpoints.get(self.index as usize - 1).unwrap();
                self.left = endpoint.saturating_sub(prev_endpoint);
                self.left = self.left.saturating_sub(1);
            }
            if let Some(n) = self.index.checked_add(1) {
                self.index = n;
            }
            true
        } else {
            self.left -= 1;
            false
        }
    }
}

#[derive(Clone)]
struct FlagsIter<'a> {
    stream: Stream<'a>,
    repeats: u8, // Stream を消費する前に，repeasts の回数を消費する．
    flags: SimpleGlyphFlags,
}

impl<'a> FlagsIter<'a> {
    fn new(data: &'a [u8]) -> Self {
        FlagsIter {
            stream: Stream::new(data),
            repeats: 0,
            flags: SimpleGlyphFlags(0),
        }
    }
}

impl<'a> Iterator for FlagsIter<'a> {
    type Item = SimpleGlyphFlags;

    fn next(&mut self) -> Option<Self::Item> {
        if self.repeats == 0 {
            // Stream を消費する．
            self.flags = SimpleGlyphFlags(self.stream.read().unwrap());
            if self.flags.repeat_flag() {
                self.repeats = self.stream.read().unwrap();
            }
        } else {
            // repeats を消費する
            self.repeats -= 1;
        }

        Some(self.flags)
    }
}

#[derive(Clone)]
struct CoordsIter<'a> {
    stream: Stream<'a>,
    prev: i16, // 差分データから絶対座標を計算するために，一つ前の絶対座標を保存する．
}

impl<'a> CoordsIter<'a> {
    fn new(data: &'a [u8]) -> Self {
        CoordsIter {
            stream: Stream::new(data),
            prev: 0,
        }
    }
    fn next(&mut self, coord_type: CoordType) -> i16 {
        let delta = match coord_type {
            CoordType::Positive8 => i16::from(self.stream.read::<u8>().unwrap()),
            CoordType::Negative8 => -i16::from(self.stream.read::<u8>().unwrap()),
            CoordType::I16 => self.stream.read::<i16>().unwrap_or(0),
            CoordType::SamePrevious => 0,
        };

        self.prev = self.prev.wrapping_add(delta);
        self.prev
    }
}

enum CoordType {
    Positive8,
    Negative8,
    SamePrevious,
    I16,
}

#[derive(Clone, Copy)]
struct SimpleGlyphFlags(u8);
impl FromData for SimpleGlyphFlags {
    const SIZE: usize = 1;
    fn parse(data: &[u8]) -> Option<Self> {
        u8::parse(data).map(Self)
    }
}

impl SimpleGlyphFlags {
    const ON_CURVE_POINT: u8 = 0x01;
    const X_SHORT_VECTOR: u8 = 0x02;
    const Y_SHORT_VECTOR: u8 = 0x04;
    const REPEAT_FLAG: u8 = 0x08;
    const X_IS_SAME_OR_POSITIVE_X_SHORT_VECTOR: u8 = 0x10;
    const Y_IS_SAME_OR_POSITIVE_Y_SHORT_VECTOR: u8 = 0x20;

    #[inline]
    fn is_1byte_x(&self) -> bool {
        self.0 & Self::X_SHORT_VECTOR != 0
    }
    #[inline]
    fn is_2byte_x(&self) -> bool {
        self.0 & (Self::X_SHORT_VECTOR | Self::X_IS_SAME_OR_POSITIVE_X_SHORT_VECTOR) == 0
    }
    #[inline]
    fn get_x_byte_size(&self) -> usize {
        // SimpleGlyphFlags::X_SHORT_VECTOR で排他的なので，どちらかは0になる．
        self.is_1byte_x() as usize + (self.is_2byte_x() as usize * 2)
    }

    #[inline]
    fn is_1byte_y(&self) -> bool {
        self.0 & Self::Y_SHORT_VECTOR != 0
    }
    #[inline]
    fn is_2byte_y(&self) -> bool {
        self.0 & (Self::Y_SHORT_VECTOR | Self::Y_IS_SAME_OR_POSITIVE_Y_SHORT_VECTOR) == 0
    }
    #[inline]
    fn get_y_byte_size(&self) -> usize {
        self.is_1byte_y() as usize + (self.is_2byte_y() as usize * 2)
    }

    fn is_on_curve_point(&self) -> bool {
        self.0 & Self::ON_CURVE_POINT != 0
    }

    fn get_x_type(&self) -> CoordType {
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

    fn get_y_type(&self) -> CoordType {
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
    fn repeat_flag(&self) -> bool {
        self.0 & Self::REPEAT_FLAG != 0
    }
}

fn get_coords_len(s: &mut Stream, number_of_points: usize) -> Option<(usize, usize)> {
    let mut flags_left = number_of_points;
    let mut x_coords_len = 0;
    let mut y_coords_len = 0;
    while flags_left > 0 {
        let flags = SimpleGlyphFlags(s.read()?);
        let repeats = if flags.repeat_flag() {
            s.read::<u8>()? as usize + 1
        } else {
            1
        };
        if repeats > flags_left {
            return None;
        }

        x_coords_len += flags.get_x_byte_size() * repeats;
        y_coords_len += flags.get_y_byte_size() * repeats;

        flags_left -= repeats;
    }

    Some((x_coords_len, y_coords_len))
}
