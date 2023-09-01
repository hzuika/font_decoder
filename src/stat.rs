use crate::{
    data_types::{uint16, Fixed, Offset16, Offset32, Tag},
    decoder::{FromData, LazyArray, Stream, UnsizedLazyArray},
};

#[allow(non_snake_case)]
#[derive(Debug)]
pub struct StatHeader {
    pub majorVersion: u16, //Major version number of the style attributes table — set to 1.
    pub minorVersion: u16, //Minor version number of the style attributes table — set to 2.
    pub designAxisSize: u16, //The size in bytes of each axis record.
    pub designAxisCount: u16, //The number of axis records. In a font with an 'fvar' table, this value must be greater than or equal to the axisCount value in the 'fvar' table. In all fonts, must be greater than zero if axisValueCount is greater than zero.
    pub designAxesOffset: Offset32, //Offset in bytes from the beginning of the STAT table to the start of the design axes array. If designAxisCount is zero, set to zero; if designAxisCount is greater than zero, must be greater than zero.
    pub axisValueCount: u16,        //The number of axis value tables.
    pub offsetToAxisValueOffsets: Offset32, //Offset in bytes from the beginning of the STAT table to the start of the design axes value offsets array. If axisValueCount is zero, set to zero; if axisValueCount is greater than zero, must be greater than zero.
    pub elidedFallbackNameID: u16, //Name ID used as fallback when projection of names into a particular font model produces a subfamily name containing only elidable elements.
}

#[allow(non_snake_case)]
impl FromData for StatHeader {
    const SIZE: usize = 4 * 2 + 4 + 2 + 4 + 2;
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let majorVersion = s.read()?;
        assert_eq!(majorVersion, 1);

        let minorVersion = s.read()?;
        assert!([1_u16, 2].contains(&minorVersion)); // 0 is deprecated.

        let designAxisSize = s.read()?;
        let designAxisCount = s.read()?;

        let designAxesOffset = s.read()?;
        assert!(if designAxisCount > 0 {
            designAxesOffset > 0
        } else {
            designAxesOffset == 0
        });

        let axisValueCount = s.read()?;
        assert!(if axisValueCount > 0 {
            designAxisCount > 0
        } else {
            designAxisCount == 0
        });

        let offsetToAxisValueOffsets = s.read()?;
        assert!(if axisValueCount > 0 {
            offsetToAxisValueOffsets > 0
        } else {
            offsetToAxisValueOffsets == 0
        });
        let elidedFallbackNameID = s.read()?;
        Some(Self {
            majorVersion,
            minorVersion,
            designAxisSize,
            designAxisCount,
            designAxesOffset,
            axisValueCount,
            offsetToAxisValueOffsets,
            elidedFallbackNameID,
        })
    }
}

#[allow(non_snake_case)]
#[derive(Debug)]
pub struct AxisRecord {
    pub axisTag: Tag,      //A tag identifying the axis of design variation.
    pub axisNameID: u16, //The name ID for entries in the 'name' table that provide a display string for this axis.
    pub axisOrdering: u16, //A value that applications can use to determine primary sorting of face names, or for ordering of labels when composing family or face names.
}

impl FromData for AxisRecord {
    const SIZE: usize = 4 + 2 + 2;
    #[allow(non_snake_case)]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let axisTag = s.read()?;
        let axisNameID = s.read()?;
        let axisOrdering = s.read()?;
        Some(Self {
            axisTag,
            axisNameID,
            axisOrdering,
        })
    }
}

#[allow(non_snake_case)]
pub struct StatTable<'a> {
    pub header: StatHeader,
    pub designAxes: UnsizedLazyArray<'a, AxisRecord>, //	[designAxisCount]	The design-axes array.
    pub axisValueOffsets: LazyArray<'a, Offset16>, //	[axisValueCount]	Array of offsets to axis value tables, in bytes from the start of the axis value offsets array.
    axisValueTables: &'a [u8],
}

impl<'a> StatTable<'a> {
    #[allow(non_snake_case)]
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let header: StatHeader = s.read()?;
        s.set_offset(header.designAxesOffset as usize);
        let designAxes = s.read_unsized_array(
            header.designAxisCount as usize,
            header.designAxisSize as usize,
            Box::new(|x| AxisRecord::parse(x)),
        )?;
        s.set_offset(header.offsetToAxisValueOffsets as usize);
        let axisValueOffsets = s.read_array(header.axisValueCount as usize)?;
        let axisValueTables = data.get(header.offsetToAxisValueOffsets as usize..)?;
        Some(Self {
            header,
            designAxes,
            axisValueOffsets,
            axisValueTables,
        })
    }

    pub fn get_axis_value_table(&self, index: usize) -> Option<AxisValueTable<'a>> {
        let offset = self.axisValueOffsets.get(index)?;
        AxisValueTable::parse(self.axisValueTables.get(offset as usize..)?)
    }

    pub fn get_axis_value_table_iter<'b>(&'b self) -> AxisValueTableIter<'b, 'a> {
        AxisValueTableIter::new(self)
    }
}

#[derive(Debug)]
pub enum AxisValueTable<'a> {
    Format1(AxisValueFormat1),
    Format2(AxisValueFormat2),
    Format3(AxisValueFormat3),
    Format4(AxisValueFormat4<'a>),
}

impl<'a> AxisValueTable<'a> {
    pub fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let format: u16 = s.read()?;
        match format {
            1 => Some(Self::Format1(AxisValueFormat1::parse(data)?)),
            2 => Some(Self::Format2(AxisValueFormat2::parse(data)?)),
            3 => Some(Self::Format3(AxisValueFormat3::parse(data)?)),
            _ => {
                panic!("invalid format {}", format)
            }
        }
    }

    pub fn get_value_name_id(&self) -> uint16 {
        match self {
            AxisValueTable::Format1(x) => x.valueNameID,
            AxisValueTable::Format2(x) => x.valueNameID,
            AxisValueTable::Format3(x) => x.valueNameID,
            AxisValueTable::Format4(x) => x.valueNameID,
        }
    }

    pub fn get_axis_indices(&self) -> Vec<uint16> {
        match self {
            AxisValueTable::Format1(x) => vec![x.axisIndex],
            AxisValueTable::Format2(x) => vec![x.axisIndex],
            AxisValueTable::Format3(x) => vec![x.axisIndex],
            AxisValueTable::Format4(x) => x
                .axisValues
                .into_iter()
                .map(|item| item.axisIndex)
                .collect(),
        }
    }
}

#[allow(non_snake_case)]
#[derive(Debug)]
pub struct AxisValueFormat1 {
    pub format: uint16,      //Format identifier — set to 1.
    pub axisIndex: uint16, //Zero-base index into the axis record array identifying the axis of design variation to which the axis value table applies. Must be less than designAxisCount.
    pub flags: uint16,     //Flags — see below for details.
    pub valueNameID: uint16, //The name ID for entries in the 'name' table that provide a display string for this attribute value.
    pub value: Fixed,        //A numeric value for this attribute value.
}

impl FromData for AxisValueFormat1 {
    const SIZE: usize = 2 * 4 + 4;
    #[allow(non_snake_case)]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let format = s.read()?;
        assert_eq!(format, 1);
        let axisIndex = s.read()?;
        let flags = s.read()?;
        let valueNameID = s.read()?;
        let value = s.read()?;
        Some(Self {
            format,
            axisIndex,
            flags,
            valueNameID,
            value,
        })
    }
}

#[allow(non_snake_case)]
#[derive(Debug)]
pub struct AxisValueFormat2 {
    pub format: uint16,       //Format identifier — set to 2.
    pub axisIndex: uint16, //Zero-base index into the axis record array identifying the axis of design variation to which the axis value table applies. Must be less than designAxisCount.
    pub flags: uint16,     //Flags — see below for details.
    pub valueNameID: uint16, //The name ID for entries in the 'name' table that provide a display string for this attribute value.
    pub nominalValue: Fixed, //A nominal numeric value for this attribute value.
    pub rangeMinValue: Fixed, //The minimum value for a range associated with the specified name ID.
    pub rangeMaxValue: Fixed, //The maximum value for a range associated with the specified name ID.
}

impl FromData for AxisValueFormat2 {
    const SIZE: usize = 4 * 2 + 4 * 3;
    #[allow(non_snake_case)]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let format = s.read()?;
        assert_eq!(format, 2);
        let axisIndex = s.read()?;
        let flags = s.read()?;
        let valueNameID = s.read()?;
        let nominalValue = s.read()?;
        let rangeMinValue = s.read()?;
        let rangeMaxValue = s.read()?;
        Some(Self {
            format,
            axisIndex,
            flags,
            valueNameID,
            nominalValue,
            rangeMinValue,
            rangeMaxValue,
        })
    }
}

#[allow(non_snake_case)]
#[derive(Debug)]
pub struct AxisValueFormat3 {
    pub format: uint16,      // Format identifier — set to 3.
    pub axisIndex: uint16, // Zero-base index into the axis record array identifying the axis of design variation to which the axis value table applies. Must be less than designAxisCount.
    pub flags: uint16,     // Flags — see below for details.
    pub valueNameID: uint16, // The name ID for entries in the 'name' table that provide a display string for this attribute value.
    pub value: Fixed,        // A numeric value for this attribute value.
    pub linkedValue: Fixed,  // The numeric value for a style-linked mapping from this value.
}

impl FromData for AxisValueFormat3 {
    const SIZE: usize = 2 * 4 + 4 * 2;
    #[allow(non_snake_case)]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let format = s.read()?;
        assert_eq!(format, 3);
        let axisIndex = s.read()?;
        let flags = s.read()?;
        let valueNameID = s.read()?;
        let value = s.read()?;
        let linkedValue = s.read()?;
        Some(Self {
            format,
            axisIndex,
            flags,
            valueNameID,
            value,
            linkedValue,
        })
    }
}

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct AxisValueFormat4<'a> {
    pub format: uint16,                       //Format identifier — set to 4.
    pub axisCount: uint16, //The total number of axes contributing to this axis-values combination.
    pub flags: uint16,     //Flags — see below for details.
    pub valueNameID: uint16, //The name ID for entries in the 'name' table that provide a display string for this combination of axis values.
    pub axisValues: LazyArray<'a, AxisValue>, // [axisCount]	Array of AxisValue records that provide the combination of axis values, one for each contributing axis.
}

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct AxisValue {
    pub axisIndex: uint16, //Zero-base index into the axis record array identifying the axis to which this value applies. Must be less than designAxisCount.
    pub value: Fixed,      //A numeric value for this attribute value.
}

impl FromData for AxisValue {
    const SIZE: usize = 2 + 4;
    #[allow(non_snake_case)]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let axisIndex = s.read()?;
        let value = s.read()?;
        Some(Self { axisIndex, value })
    }
}

pub struct AxisValueTableIter<'a, 'b> {
    stat_table: &'a StatTable<'b>,
    index: usize,
}

impl<'a, 'b> AxisValueTableIter<'a, 'b> {
    pub fn new(stat_table: &'a StatTable<'b>) -> Self {
        Self {
            stat_table,
            index: 0,
        }
    }
}

impl<'a, 'b> Iterator for AxisValueTableIter<'a, 'b> {
    type Item = AxisValueTable<'b>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.len() {
            self.index += 1;
            self.stat_table.get_axis_value_table(self.index - 1)
        } else {
            None
        }
    }
}

impl<'a, 'b> ExactSizeIterator for AxisValueTableIter<'a, 'b> {
    fn len(&self) -> usize {
        self.stat_table.header.axisValueCount as usize
    }
}
