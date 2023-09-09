// glyf table は Glyph table が並べて格納される．
// glyf table から特定の Glyph table を取得するためには，loca table が必要である．
// loca table は glyf table の先頭から glyph id に対応する Glyph table のオフセットが格納される．
// 1. loca table に glyph id を渡して， Glyph table が存在する範囲を取得する．
// 2. glyf table から該当のバイト列を取得する．
// 3. バイト列をパースする．
// 4. Simple glyph table の場合は，そのままパースする．
// 4. Composite glyph table の場合は，指定された glyph id を使って，1 から順に実行して，パースされた Simple glyph table を変形させる．

use std::ops::Range;

use crate::{
    data_types::{int16, uint16, uint8, F2DOT14},
    decoder::{FromData, LazyArray, Stream},
    loca::LocaTable,
};

// glyf table 自体はただのバイト列を持つ．
// どの範囲に Glyph table が存在するか自身では分からない．
// glyph id に対応する Glyph table の範囲を取得するためには loca table の情報を使用する．
pub struct GlyfTable<'a>(pub &'a [u8]);

impl<'a> GlyfTable<'a> {
    // loca table を使って取得した範囲のバイト列を取得する．
    pub fn get_data(&self, range: Range<usize>) -> Option<&'a [u8]> {
        self.0.get(range)
    }
}

pub struct Glyph<'a> {
    pub header: GlyphHeader,
    pub subtable: GlyphTable<'a>,
}

impl<'a> Glyph<'a> {
    pub fn parse(data: &'a [u8], loca: LocaTable<'a>, glyf: GlyfTable<'a>) -> Option<Self> {
        let mut s = Stream::new(data);
        let header: GlyphHeader = s.read()?;
        match header.get_type() {
            GlyphType::Simple => {
                let table = SimpleGlyphTable::parse(s.get_tail()?, header.numberOfContours as u16)?;
                Some(Glyph {
                    header,
                    subtable: GlyphTable::Simple(table),
                })
            }
            GlyphType::Composite => {
                let table = CompositeGlyphTable::parse(s.get_tail()?, loca, glyf);
                Some(Glyph {
                    header,
                    subtable: GlyphTable::Composite(table),
                })
            }
        }
    }
}

// このヘッダーはバウンディングボックスの情報(xMin, yMin, xMax, yMax)を持っているが，正しいか分からないし，Variable fontの場合は結局バウンディングボックスの大きさが変わるので，この値は使うべきではない．
// バウンディングボックスの情報が欲しい場合は計算したほうが良い．
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

// Glyph には二種類のタイプが存在し，それらは numberOfContours の値によって分類できます．
impl GlyphHeader {
    pub fn get_type(&self) -> GlyphType {
        if self.numberOfContours >= 0 {
            GlyphType::Simple
        } else {
            GlyphType::Composite
        }
    }
}

pub enum GlyphType {
    Simple,
    Composite,
}

// GlyphTable は GlyfTable と異なり，一つの glyph id に対応するグリフの点の座標のデータを持っている．
pub enum GlyphTable<'a> {
    Simple(SimpleGlyphTable<'a>),
    Composite(CompositeGlyphTable<'a>),
}

impl<'a> GlyphTable<'a> {
    // グリフの点の座標を順番に返すようなイテレータを返す．
    pub fn get_glyph_points_iter(&self) -> &GlyphPointsIter<'a> {
        match self {
            Self::Simple(table) => &table.glyph_points_iter,
            Self::Composite(table) => todo!(),
        }
    }
}

#[allow(non_snake_case)]
pub struct SimpleGlyphTable<'a> {
    pub endPtsOfContours: LazyArray<'a, uint16>, //[numberOfContours]Array of point indices for the last point of each contour, in increasing numeric order.
    pub instructionLength: uint16, //Total number of bytes for instructions. If instructionLength is zero, no instructions are present for this glyph, and this field is followed directly by the flags field.
    pub instructions: LazyArray<'a, uint8>, //[instructionLength]Array of instruction byte code for the glyph.
    // 仕様では，flags, xCoordinates, yCoordinates などが存在しますが，これらは可変で型が分からない配列なので，これらをまとめて GlyphPointsIter で表す．
    pub glyph_points_iter: GlyphPointsIter<'a>,
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

        // 仕様では，flags, xCoordinates, yCoordinates などが存在しますが，これらは可変で型が分からない配列なので，これらをまとめて GlyphPointsIter で表す．
        let flags_offset = s.get_offset();
        // イテレータを構築するために，一度，各配列のバイト長を計算する．
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

// TODO: GlyphContoursIter にしてもいいかもしれない．だが，輪郭ごとのイテレータだとしても，自分が輪郭を構成する最後の点であるという情報は必要になりそう．そうしないと，閉じるパスを作ることができない．
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

// Glyph の点の座標と属性．
#[derive(Debug, Clone, Copy)]
pub struct GlyphPoint {
    pub x: i16,
    pub y: i16,
    pub is_on_curve: bool,
    pub is_last: bool,
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

// x座標，y座標が格納された配列のバイト長を取得する．
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

// Composite glyph table には，点の座標のデータは含まれていない．
// 代わりに，自身とは別の glyph id と，その glyph id に対応する glyph をどのように変形させ，配置させるかの情報を持つ．
// したがって， Composite Glyph Table 自身の glyph の点を取得するためには， loca table と glyf table が必要になる．
pub struct CompositeGlyphTable<'a> {
    pub iter: CompositeGlyphIter<'a>,
    pub loca: LocaTable<'a>,
    pub glyf: GlyfTable<'a>,
}

impl<'a> CompositeGlyphTable<'a> {
    pub fn parse(data: &'a [u8], loca: LocaTable<'a>, glyf: GlyfTable<'a>) -> Self {
        let iter = CompositeGlyphIter::new(data);
        Self { iter, loca, glyf }
    }
}

// Composite glyph が構成するためのコンポーネントを返すイテレータ
pub struct CompositeGlyphIter<'a> {
    stream: Stream<'a>,
}

impl<'a> CompositeGlyphIter<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            stream: Stream::new(data),
        }
    }
}

// Composite Glyph を構成するコンポーネント．
pub struct CompositeGlyphComponent {
    pub glyph_id: u16,
    pub transform: Transform,
    pub flags: CompositeGlyphFlags,
    pub parent_glyph_point_number: Option<u16>, // Component glyph を構築した後のポイントから，番号を指定して取得する必要がある．Vecが必要?
    pub child_glyph_point_number: Option<u16>,
}

impl<'a> Iterator for CompositeGlyphIter<'a> {
    type Item = CompositeGlyphComponent;
    fn next(&mut self) -> Option<Self::Item> {
        let flags = CompositeGlyphFlags(self.stream.read()?);
        let glyph_id = self.stream.read::<u16>()?;

        let mut transform = Transform::default();

        let mut parent_glyph_point_number = None;
        let mut child_glyph_point_number = None;

        if flags.args_are_xy_values() {
            // コンポーネントグリフの各制御点の座標に追加されるオフセットベクトル．
            // Variable font の場合は， gvar table のデルタによってオフセットベクトルを変更できる．
            // オフセットベクトルに変換行列を適用するかどうかは，SCALED_COMPONENT_OFFSET と UNSCALED_COMPONENT_OFFSET フラグによって決定する．
            if flags.arg_1_and_2_are_16bit() {
                transform.e = f32::from(self.stream.read::<i16>()?);
                transform.f = f32::from(self.stream.read::<i16>()?);
            } else {
                transform.e = f32::from(self.stream.read::<i8>()?);
                transform.f = f32::from(self.stream.read::<i8>()?);
            }
        } else {
            // unsigned point numbers.
            if flags.arg_1_and_2_are_16bit() {
                // 以前のコンポーネントグリフから組み込まれ，再番号付けされた輪郭からのポイント番号．
                parent_glyph_point_number = Some(self.stream.read::<u16>()?);
                // 子コンポーネントグリフの再番号付け前のポイント番号．
                // このポイント番号にある制御点を，親グリフのポイント番号にある制御点に一致するように子コンポーネントグリフを配置する．
                // 変換行列が指定されている場合は，位置合わせの前に，子のグリフに変換が適用される．
                child_glyph_point_number = Some(self.stream.read::<u16>()?);
            } else {
                parent_glyph_point_number = Some(self.stream.read::<u8>()? as u16);
                child_glyph_point_number = Some(self.stream.read::<u8>()? as u16);
            }
        }

        if flags.we_have_a_two_by_two() {
            transform.a = self.stream.read::<F2DOT14>()?.to_f32();
            transform.b = self.stream.read::<F2DOT14>()?.to_f32();
            transform.c = self.stream.read::<F2DOT14>()?.to_f32();
            transform.d = self.stream.read::<F2DOT14>()?.to_f32();
        } else if flags.we_have_an_x_and_y_scale() {
            transform.a = self.stream.read::<F2DOT14>()?.to_f32();
            transform.d = self.stream.read::<F2DOT14>()?.to_f32();
        } else if flags.we_have_a_scale() {
            transform.a = self.stream.read::<F2DOT14>()?.to_f32();
            transform.d = transform.a;
        }

        if !flags.more_components() {
            // ここで offset を最後まで移動させておくことで，次の next() 呼び出し時に，最初の get() で None を返す．
            self.stream.set_end();
        }

        Some(CompositeGlyphComponent {
            glyph_id,
            transform,
            flags,
            parent_glyph_point_number,
            child_glyph_point_number,
        })
    }
}

// コンポーネントのフラグ．
pub struct CompositeGlyphFlags(u16);

impl CompositeGlyphFlags {
    pub const ARG_1_AND_2_ARE_WORDS: u16 = 0x0001; //Bit 0: If this is set, the arguments are 16-bit (uint16 or int16); otherwise, they are bytes (uint8 or int8).
    pub const ARGS_ARE_XY_VALUES: u16 = 0x0002; //Bit 1: If this is set, the arguments are signed xy values; otherwise, they are unsigned point numbers.
    pub const ROUND_XY_TO_GRID: u16 = 0x0004; //Bit 2: If set and ARGS_ARE_XY_VALUES is also set, the xy values are rounded to the nearest grid line. Ignored if ARGS_ARE_XY_VALUES is not set. 変換行列と，Variable font の delta が適用された後の オフセットベクトルを最も近いピクセルグリッドラインにフィットさせる．
    pub const WE_HAVE_A_SCALE: u16 = 0x0008; //Bit 3: This indicates that there is a simple scale for the component. Otherwise, scale = 1.0.
    pub const MORE_COMPONENTS: u16 = 0x0020; //Bit 5: Indicates at least one more glyph after this one.
    pub const WE_HAVE_AN_X_AND_Y_SCALE: u16 = 0x0040; //Bit 6: The x direction will use a different scale from the y direction.
    pub const WE_HAVE_A_TWO_BY_TWO: u16 = 0x0080; //Bit 7: There is a 2 by 2 transformation that will be used to scale the component.
    pub const WE_HAVE_INSTRUCTIONS: u16 = 0x0100; //Bit 8: Following the last component are instructions for the composite character.
    pub const USE_MY_METRICS: u16 = 0x0200; //Bit 9: If set, this forces the aw and lsb (and rsb) for the composite to be equal to those from this component glyph. This works for hinted and unhinted glyphs.
    pub const OVERLAP_COMPOUND: u16 = 0x0400; //Bit 10: If set, the components of the compound glyph overlap. Use of this flag is not required in OpenType — that is, it is valid to have components overlap without having this flag set. It may affect behaviors in some platforms, however. (See Apple’s specification for details regarding behavior in Apple platforms.) When used, it must be set on the flag word for the first component. See additional remarks, above, for the similar OVERLAP_SIMPLE flag used in simple-glyph descriptions.
    pub const SCALED_COMPONENT_OFFSET: u16 = 0x0800; //Bit 11: The composite is designed to have the component offset scaled. Ignored if ARGS_ARE_XY_VALUES is not set.
    pub const UNSCALED_COMPONENT_OFFSET: u16 = 0x1000; //Bit 12: The composite is designed not to have the component offset scaled. Ignored if ARGS_ARE_XY_VALUES is not set.
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
}

// コンポーネントを変形させるための行列．
// [a b e]
// [d d f]
pub struct Transform {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub e: f32,
    pub f: f32,
}

impl Default for Transform {
    #[inline]
    fn default() -> Self {
        Transform {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: 0.0,
            f: 0.0,
        }
    }
}
