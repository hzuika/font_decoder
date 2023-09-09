use crate::{
    data_types::{Fixed, Offset16, Tag},
    decoder::{FromData, Stream},
};

#[allow(non_snake_case)]
pub struct FvarHeader {
    pub majorVersion: u16, // Major version number of the font variations table — set to 1.
    pub minorVersion: u16, //Minor version number of the font variations table — set to 0.
    pub axesArrayOffset: Offset16, //Offset in bytes from the beginning of the table to the start of the VariationAxisRecord array.
    pub reserved: u16,             //This field is permanently reserved. Set to 2.
    pub axisCount: u16, //The number of variation axes in the font (the number of records in the axes array).
    pub axisSize: u16, //The size in bytes of each VariationAxisRecord — set to 20 (0x0014) for this version.
    pub instanceCount: u16, //The number of named instances defined in the font (the number of records in the instances array).
    pub instanceSize: u16, //The size in bytes of each InstanceRecord — set to either axisCount * sizeof(Fixed) + 4, or to axisCount * sizeof(Fixed) + 6.
}

impl FromData for FvarHeader {
    const SIZE: usize = 2 * 8;
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(Self {
            majorVersion: s.read()?,
            minorVersion: s.read()?,
            axesArrayOffset: s.read()?,
            reserved: s.read()?,
            axisCount: s.read()?,
            axisSize: s.read()?,
            instanceCount: s.read()?,
            instanceSize: s.read()?,
        })
    }
}

#[allow(non_snake_case)]
#[derive(Debug)]
pub struct VariationAxisRecord {
    pub axisTag: Tag,        // Tag identifying the design variation for the axis.
    pub minValue: Fixed,     // The minimum coordinate value for the axis.
    pub defaultValue: Fixed, // The default coordinate value for the axis.
    pub maxValue: Fixed,     // The maximum coordinate value for the axis.
    pub flags: u16,          // Axis qualifiers — see details below.
    pub axisNameId: u16, // The name ID for entries in the 'name' table that provide a display name for this axis.
}

impl FromData for VariationAxisRecord {
    const SIZE: usize = 20;
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(Self {
            axisTag: s.read()?,
            minValue: s.read()?,
            defaultValue: s.read()?,
            maxValue: s.read()?,
            flags: s.read()?,
            axisNameId: s.read()?,
        })
    }
}

#[derive(Debug)]
pub struct UserTuple {
    pub coordinates: Vec<Fixed>, // axisCount
}

#[allow(non_snake_case)]
#[derive(Debug)]
pub struct InstanceRecord {
    pub subfamilyNameId: u16, // The name ID for entries in the 'name' table that provide subfamily names for this instance.
    pub flags: u16,           // Reserved for future use — set to 0.
    pub coordinates: UserTuple, // The coordinates array for this instance.
    pub postScriptNameId: Option<u16>, // Optional. The name ID for entries in the 'name' table that provide PostScript names for this instance.
}

impl InstanceRecord {
    pub fn parse(data: &[u8], axis_count: usize) -> Option<Self> {
        let mut s = Stream::new(data);
        let subfamily_name_id = s.read()?;
        let flags = s.read()?;
        let coordinates = UserTuple {
            coordinates: s.read_array(axis_count)?,
        };
        let post_script_name_id = s.read();

        Some(Self {
            subfamilyNameId: subfamily_name_id,
            flags,
            coordinates,
            postScriptNameId: post_script_name_id,
        })
    }
}

pub struct FvarTable<'a> {
    pub data: &'a [u8],
    pub header: FvarHeader,
    pub axes: Vec<VariationAxisRecord>,
    pub instances: Vec<InstanceRecord>,
}

impl<'a> FvarTable<'a> {
    pub fn parse(data: &'a [u8]) -> Option<FvarTable<'a>> {
        let mut s = Stream::new(data);
        let header: FvarHeader = s.read()?;
        let offset = header.axesArrayOffset as usize;
        s.set_offset(offset);
        let axes = s.read_array(header.axisCount as usize)?;
        let instance_size = header.instanceSize as usize;
        let instance_count = header.instanceCount as usize;
        let axis_count = header.axisCount as usize;
        let instances = s.read_unsized_array(
            instance_count,
            instance_size,
            Box::new(move |data| InstanceRecord::parse(data, axis_count)),
        )?;
        Some(FvarTable {
            data,
            header,
            axes,
            instances,
        })
    }
}
