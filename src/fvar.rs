use crate::{
    data_types::{Fixed, Offset16, Tag},
    decoder::{FromData, LazyArray, Stream, UnsizedLazyArray},
};

pub struct FvarTableHeader {
    pub major_version: u16, // Major version number of the font variations table — set to 1.
    pub minor_version: u16, //Minor version number of the font variations table — set to 0.
    pub axes_array_offset: Offset16, //Offset in bytes from the beginning of the table to the start of the VariationAxisRecord array.
    pub reserved: u16,               //This field is permanently reserved. Set to 2.
    pub axis_count: u16, //The number of variation axes in the font (the number of records in the axes array).
    pub axis_size: u16, //The size in bytes of each VariationAxisRecord — set to 20 (0x0014) for this version.
    pub instance_count: u16, //The number of named instances defined in the font (the number of records in the instances array).
    pub instance_size: u16, //The size in bytes of each InstanceRecord — set to either axisCount * sizeof(Fixed) + 4, or to axisCount * sizeof(Fixed) + 6.
}

impl FromData for FvarTableHeader {
    const SIZE: usize = 2 * 8;
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(Self {
            major_version: s.read()?,
            minor_version: s.read()?,
            axes_array_offset: s.read()?,
            reserved: s.read()?,
            axis_count: s.read()?,
            axis_size: s.read()?,
            instance_count: s.read()?,
            instance_size: s.read()?,
        })
    }
}

#[derive(Debug)]
pub struct VariationAxisRecord {
    pub axis_tag: Tag,        // Tag identifying the design variation for the axis.
    pub min_value: Fixed,     // The minimum coordinate value for the axis.
    pub default_value: Fixed, // The default coordinate value for the axis.
    pub max_value: Fixed,     // The maximum coordinate value for the axis.
    pub flags: u16,           // Axis qualifiers — see details below.
    pub axis_name_id: u16, // The name ID for entries in the 'name' table that provide a display name for this axis.
}

impl FromData for VariationAxisRecord {
    const SIZE: usize = 20;
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(Self {
            axis_tag: s.read()?,
            min_value: s.read()?,
            default_value: s.read()?,
            max_value: s.read()?,
            flags: s.read()?,
            axis_name_id: s.read()?,
        })
    }
}

pub struct UserTuple<'a> {
    pub coordinates: LazyArray<'a, Fixed>, // axisCount
}

pub struct InstanceRecord<'a> {
    pub subfamily_name_id: u16, // The name ID for entries in the 'name' table that provide subfamily names for this instance.
    pub flags: u16,             // Reserved for future use — set to 0.
    pub coordinates: UserTuple<'a>, // The coordinates array for this instance.
    pub post_script_name_id: u16, // Optional. The name ID for entries in the 'name' table that provide PostScript names for this instance.
}

impl<'a> InstanceRecord<'a> {
    pub fn parse(data: &'a [u8], instance_count: usize) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(Self {
            subfamily_name_id: s.read()?,
            flags: s.read()?,
            coordinates: UserTuple {
                coordinates: s.read_array(instance_count)?,
            },
            post_script_name_id: s.read()?,
        })
    }
}

pub struct FvarTable<'a, F> {
    pub data: &'a [u8],
    pub header: FvarTableHeader,
    pub axes: LazyArray<'a, VariationAxisRecord>,
    pub instances: UnsizedLazyArray<'a, InstanceRecord<'a>, F>,
}

impl<'a, F> FvarTable<'a, F> {
    pub fn parse(
        data: &'a [u8],
    ) -> Option<FvarTable<'a, impl Fn(&'a [u8]) -> Option<InstanceRecord<'a>>>> {
        let mut s = Stream::new(data);
        let header: FvarTableHeader = s.read()?;
        let offset = header.axes_array_offset as usize;
        s.set_offset(offset);
        let axes = s.read_array(header.axis_count as usize)?;
        let instance_size = header.instance_size as usize;
        let instance_count = header.instance_count as usize;
        let instances = s.read_unsized_array(instance_count, instance_size, move |data| {
            InstanceRecord::parse(data, instance_count)
        })?;
        Some(FvarTable {
            data,
            header,
            axes,
            instances,
        })
    }
}
