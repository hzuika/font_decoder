use core::fmt;
// 1. loca table に glyph id を渡して， glyf table の範囲を取得する．
// 2. glyf table に範囲を渡して，バイト列を取得する．
// 3-1. Simple glyph の場合は，そのままパースする．
// 3-2. Composite glyph の場合は， components の数だけ 1. から繰り返す．
use std::ops::Range;

use crate::{
    data_types::{int16, uint16, uint8, F2DOT14},
    decoder::{FromData, Stream},
    loca::LocaTable,
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
        let header: GlyphHeader = s.read().unwrap();
        match header.get_type() {
            GlyphType::Simple => {
                let tail = s.get_tail().unwrap();
                let subtable = SimpleGlyphTable::parse(tail, header.numberOfContours as u16)?;
                Some(Glyph {
                    header,
                    subtable: GlyphTable::Simple(subtable),
                })
            }
            GlyphType::Composite => {
                let subtable = CompositeGlyphTable::parse(s.get_tail()?)?;
                Some(Glyph {
                    header,
                    subtable: GlyphTable::Composite(subtable),
                })
            }
        }
    }

    pub fn get_points(&self, loca: &LocaTable, glyf: &GlyfTable<'_>) -> Vec<GlyphPoint> {
        match &self.subtable {
            GlyphTable::Simple(table) => table.get_points(),
            GlyphTable::Composite(table) => table.get_points(loca, glyf),
        }
    }
}

pub enum GlyphTable {
    Simple(SimpleGlyphTable),
    Composite(CompositeGlyphTable),
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

    fn get_points(&self) -> Vec<GlyphPoint> {
        let mut points = vec![];
        for i in 0..self.flags.len() as usize {
            points.push(GlyphPoint {
                x: f64::from(self.xCoordinates[i]),
                y: f64::from(self.yCoordinates[i]),
                flags: self.flags[i],
                is_last: self
                    .endPtsOfContours
                    .iter()
                    .find(|&&x| x as usize == i)
                    .is_some(),
            })
        }
        points
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GlyphPoint {
    pub x: f64, // 変換行列の適用により，整数ではなくなるため Rust の基準型である f64 を使う．
    pub y: f64,
    pub flags: SimpleGlyphFlags,
    pub is_last: bool,
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

// 位置合わせに使うための値．
#[derive(Debug)]
pub enum CompositeGlyphArgs {
    Offset { x: i16, y: i16 },         // offset x, offset y
    Point { parent: u16, child: u16 }, // parent point number, child point number
}
// Composite Glyph を構成するコンポーネント．
#[derive(Debug)]
pub struct CompositeGlyphComponent {
    pub flags: CompositeGlyphFlags,
    pub glyph_id: u16,
    pub args: CompositeGlyphArgs,
    pub transform: Transform,
}

impl CompositeGlyphComponent {
    fn parse(data: &[u8]) -> Vec<Self> {
        let mut s = Stream::new(data);
        let mut v = vec![];
        while !s.is_end() {
            let flags = CompositeGlyphFlags(s.read().unwrap());
            let glyph_id = s.read::<u16>().unwrap();

            let args = if flags.args_are_xy_values() {
                // コンポーネントグリフの各制御点の座標に追加されるオフセットベクトル．
                // Variable font の場合は， gvar table のデルタによってオフセットベクトルを変更できる．
                // オフセットベクトルに変換行列を適用するかどうかは，SCALED_COMPONENT_OFFSET と UNSCALED_COMPONENT_OFFSET フラグによって決定する．
                let (x, y) = if flags.arg_1_and_2_are_16bit() {
                    let x = s.read::<i16>().unwrap();
                    let y = s.read::<i16>().unwrap();
                    (x, y)
                } else {
                    let x = s.read::<i8>().unwrap() as i16;
                    let y = s.read::<i8>().unwrap() as i16;
                    (x, y)
                };
                CompositeGlyphArgs::Offset { x, y }
            } else {
                // unsigned point numbers.
                let (parent, child) = if flags.arg_1_and_2_are_16bit() {
                    // parent は以前のコンポーネントグリフから組み込まれ，再番号付けされた輪郭からのポイント番号．
                    // child は子コンポーネントグリフの再番号付け前のポイント番号．
                    // このポイント番号にある制御点を，親グリフのポイント番号にある制御点に一致するように子コンポーネントグリフを配置する．
                    // 変換行列が指定されている場合は，位置合わせの前に，子のグリフに変換が適用される．
                    let parent = s.read::<u16>().unwrap();
                    let child = s.read::<u16>().unwrap();
                    (parent, child)
                } else {
                    let parent = s.read::<u8>().unwrap() as u16;
                    let child = s.read::<u8>().unwrap() as u16;
                    (parent, child)
                };
                CompositeGlyphArgs::Point { parent, child }
            };

            let mut transform = Transform::default();
            if flags.we_have_a_two_by_two() {
                transform.a = s.read::<F2DOT14>().unwrap().to_f32().into();
                transform.b = s.read::<F2DOT14>().unwrap().to_f32().into();
                transform.c = s.read::<F2DOT14>().unwrap().to_f32().into();
                transform.d = s.read::<F2DOT14>().unwrap().to_f32().into();
            } else if flags.we_have_an_x_and_y_scale() {
                transform.a = s.read::<F2DOT14>().unwrap().to_f32().into();
                transform.d = s.read::<F2DOT14>().unwrap().to_f32().into();
            } else if flags.we_have_a_scale() {
                transform.a = s.read::<F2DOT14>().unwrap().to_f32().into();
                transform.d = transform.a;
            }

            if !flags.more_components() {
                // ここで offset を最後まで移動させておくことで，次の next() 呼び出し時に，最初の get() で None を返す．
                s.set_end();
            }

            v.push(CompositeGlyphComponent {
                flags,
                glyph_id,
                args,
                transform,
            });
        }
        v
    }
}

#[derive(Debug)]
pub struct CompositeGlyphTable {
    pub components: Vec<CompositeGlyphComponent>,
}

impl CompositeGlyphTable {
    pub fn parse<'a>(data: &'a [u8]) -> Option<Self> {
        let components = CompositeGlyphComponent::parse(data);
        Some(Self { components })
    }

    pub fn get_points(&self, loca: &LocaTable, glyf: &GlyfTable<'_>) -> Vec<GlyphPoint> {
        let mut v: Vec<GlyphPoint> = vec![];
        for component in &self.components {
            let glyph_id = component.glyph_id;
            // Composite glyph を構成する Glyph は必ず存在するので， unwrap() を使う．
            let range = loca.get_glyf_range(glyph_id).unwrap();
            let data = glyf.get_data(range).unwrap();
            let glyph = Glyph::parse(data).unwrap();
            let mut points = glyph.get_points(loca, glyf);
            for point in &mut points {
                (point.x, point.y) = component.transform.multiply(point.x, point.y);
            }
            match component.args {
                CompositeGlyphArgs::Offset { x, y } => {
                    let (x, y): (f64, f64) = if component.flags.unscaled_component_offset() {
                        (f64::from(x), f64::from(y))
                    } else {
                        // scaled.
                        component.transform.multiply(f64::from(x), f64::from(y))
                    };

                    // オフセットを行う
                    for point in &mut points {
                        (point.x, point.y) = (point.x + x, point.y + y);
                    }
                }
                CompositeGlyphArgs::Point { parent, child } => {
                    // 親の parent 番目の point と子の child 番目の point が重なるように 子のグリフ点を移動させる．
                    // 例 child (1, 1), parent (0, 0) -> offset (-1, -1)
                    let parent = v[parent as usize];
                    let child = points[child as usize];
                    let (x, y) = (parent.x - child.x, parent.y - child.y);
                    for point in &mut points {
                        (point.x, point.y) = (point.x + x, point.y + y);
                    }
                }
            }
            v.extend(points);
        }
        v
    }
}

#[derive(Clone, Copy)]
pub struct CompositeGlyphFlags(u16);

impl fmt::Debug for CompositeGlyphFlags {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut v = vec![];
        if self.0 & Self::ARG_1_AND_2_ARE_WORDS != 0 { v.push("ARG_1_AND_2_ARE_WORDS")}
        if self.0 & Self::ARGS_ARE_XY_VALUES != 0 { v.push("ARGS_ARE_XY_VALUES")}
        if self.0 & Self::ROUND_XY_TO_GRID != 0 { v.push("ROUND_XY_TO_GRID")}
        if self.0 & Self::WE_HAVE_A_SCALE != 0 { v.push("WE_HAVE_A_SCALE")}
        if self.0 & Self::MORE_COMPONENTS != 0 { v.push("MORE_COMPONENTS")}
        if self.0 & Self::WE_HAVE_AN_X_AND_Y_SCALE != 0 { v.push("WE_HAVE_AN_X_AND_Y_SCALE")}
        if self.0 & Self::WE_HAVE_A_TWO_BY_TWO != 0 { v.push("WE_HAVE_A_TWO_BY_TWO")}
        if self.0 & Self::WE_HAVE_INSTRUCTIONS != 0 { v.push("WE_HAVE_INSTRUCTIONS")}
        if self.0 & Self::USE_MY_METRICS != 0 { v.push("USE_MY_METRICS")}
        if self.0 & Self::OVERLAP_COMPOUND != 0 { v.push("OVERLAP_COMPOUND")}
        if self.0 & Self::SCALED_COMPONENT_OFFSET != 0 { v.push("SCALED_COMPONENT_OFFSET")}
        if self.0 & Self::UNSCALED_COMPONENT_OFFSET != 0 { v.push("UNSCALED_COMPONENT_OFFSET")}
        let v = v.join(",");
        write!(f, "{}", v)
    }
}

impl CompositeGlyphFlags {
    const ARG_1_AND_2_ARE_WORDS: u16 = 0x0001; //Bit 0: If this is set, the arguments are 16-bit (uint16 or int16); otherwise, they are bytes (uint8 or int8).
    const ARGS_ARE_XY_VALUES: u16 = 0x0002; //Bit 1: If this is set, the arguments are signed xy values; otherwise, they are unsigned point numbers.
    const ROUND_XY_TO_GRID: u16 = 0x0004; //Bit 2: If set and ARGS_ARE_XY_VALUES is also set, the xy values are rounded to the nearest grid line. Ignored if ARGS_ARE_XY_VALUES is not set. 変換行列と，Variable font の delta が適用された後の オフセットベクトルを最も近いピクセルグリッドラインにフィットさせる．
    const WE_HAVE_A_SCALE: u16 = 0x0008; //Bit 3: This indicates that there is a simple scale for the component. Otherwise, scale = 1.0.
    const MORE_COMPONENTS: u16 = 0x0020; //Bit 5: Indicates at least one more glyph after this one.
    const WE_HAVE_AN_X_AND_Y_SCALE: u16 = 0x0040; //Bit 6: The x direction will use a different scale from the y direction.
    const WE_HAVE_A_TWO_BY_TWO: u16 = 0x0080; //Bit 7: There is a 2 by 2 transformation that will be used to scale the component.
    const WE_HAVE_INSTRUCTIONS: u16 = 0x0100; //Bit 8: Following the last component are instructions for the composite character.
    const USE_MY_METRICS: u16 = 0x0200; //Bit 9: If set, this forces the aw and lsb (and rsb) for the composite to be equal to those from this component glyph. This works for hinted and unhinted glyphs.
    const OVERLAP_COMPOUND: u16 = 0x0400; //Bit 10: If set, the components of the compound glyph overlap. Use of this flag is not required in OpenType — that is, it is valid to have components overlap without having this flag set. It may affect behaviors in some platforms, however. (See Apple’s specification for details regarding behavior in Apple platforms.) When used, it must be set on the flag word for the first component. See additional remarks, above, for the similar OVERLAP_SIMPLE flag used in simple-glyph descriptions.
    const SCALED_COMPONENT_OFFSET: u16 = 0x0800; //Bit 11: The composite is designed to have the component offset scaled. Ignored if ARGS_ARE_XY_VALUES is not set.
    const UNSCALED_COMPONENT_OFFSET: u16 = 0x1000; //Bit 12: The composite is designed not to have the component offset scaled. Ignored if ARGS_ARE_XY_VALUES is not set.
    fn args_are_xy_values(&self) -> bool {
        self.0 & Self::ARGS_ARE_XY_VALUES != 0
    }
    fn arg_1_and_2_are_16bit(&self) -> bool {
        self.0 & Self::ARG_1_AND_2_ARE_WORDS != 0
    }

    fn we_have_a_two_by_two(&self) -> bool {
        self.0 & Self::WE_HAVE_A_TWO_BY_TWO != 0
    }

    fn we_have_an_x_and_y_scale(&self) -> bool {
        self.0 & Self::WE_HAVE_AN_X_AND_Y_SCALE != 0
    }

    fn we_have_a_scale(&self) -> bool {
        self.0 & Self::WE_HAVE_A_SCALE != 0
    }

    fn more_components(&self) -> bool {
        self.0 & Self::MORE_COMPONENTS != 0
    }

    fn unscaled_component_offset(&self) -> bool {
        // 両方のフラグが立っているような不正な状態はデフォルトの値が使われる．
        // デフォルトは UNSCALED_COMPONENT_OFFSET である．
        //                                | SCALED_COMPONENT_OFFSET ON | UNSCALED_COMPONENT_OFFSET ON
        //  SCALED_COMPONENT_OFFSET   OFF | true                       | true
        //  UNSCALED_COMPONENT_OFFSET OFF | false                      | true
        if self.0 & (Self::SCALED_COMPONENT_OFFSET | Self::UNSCALED_COMPONENT_OFFSET)
            == (Self::SCALED_COMPONENT_OFFSET)
        {
            false
        } else {
            true
        }
    }
}

// [a b]
// [c d]
/// コンポーネントグリフの点を変形させるための2x2行列．
#[derive(Debug, Clone, PartialEq)]
pub struct Transform {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
}

impl Transform {
    #[inline]
    /// (new x, new y) = (ax + by, cx + dy)
    fn multiply(&self, x: f64, y: f64) -> (f64, f64) {
        (self.a * x + self.b * y, self.c * x + self.d * y)
    }
}

impl Default for Transform {
    #[inline]
    fn default() -> Self {
        Transform {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
        }
    }
}
