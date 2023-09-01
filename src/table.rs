use crate::{
    data_types::{Offset32, TableTag, Tag, FVAR, NAME},
    decoder::{FromData, LazyArray, Stream},
    fvar::FvarTable,
    name::NameTable,
};

pub struct TTCHeader<'a> {
    pub ttc_tag: Tag, // Font Collection ID string: 'ttcf' (used for fonts with CFF or CFF2 outlines as well as TrueType outlines)
    pub major_version: u16, // Major version of the TTC Header, = 1.
    pub minor_version: u16, // Minor version of the TTC Header, = 0.
    pub num_fonts: u32, // Number of fonts in TTC
    pub table_directory_offsets: LazyArray<'a, Offset32>, // Array of offsets to the TableDirectory for each font from the beginning of the file
}

impl<'a> TTCHeader<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let ttc_tag = s.read()?;
        let major_version = s.read()?;
        let minor_version = s.read()?;
        let num_fonts = s.read()?;
        let table_directory_offsets = s.read_array(num_fonts as usize)?;
        Some(Self {
            ttc_tag,
            major_version,
            minor_version,
            num_fonts,
            table_directory_offsets,
        })
    }
}

pub struct Collection<'a> {
    data: &'a [u8],
    pub header: TTCHeader<'a>,
}

impl<'a> Collection<'a> {
    pub fn new(data: &'a [u8]) -> Option<Self> {
        let header = TTCHeader::parse(data)?;
        Some(Self { data, header })
    }

    pub fn get(&self, index: usize) -> Option<Table<'a>> {
        let offset = self.header.table_directory_offsets.get(index)? as usize;
        let table_record_data = self.data.get(offset..self.data.len())?;
        let table_directory = TableDirectory::parse(table_record_data)?;
        Some(Table {
            data: self.data,
            table_directory,
        })
    }
}

pub fn is_ttc(data: &[u8]) -> Option<bool> {
    let mut s = Stream::new(data);
    let tag: Tag = s.read()?;
    Some(tag == Tag::from_be_bytes(*b"ttcf"))
}

fn check_sfnt_version(sfnt_version: &Tag) {
    const TRUETYPE: Tag = Tag(0x00010000);
    const CFF: Tag = Tag::from_be_bytes(*b"OTTO");
    assert!(
        sfnt_version == &TRUETYPE || sfnt_version == &CFF,
        "invalid sfnt version 0x{:x}",
        sfnt_version.0
    );
}

#[derive(Debug)]
pub struct TableRecord {
    pub table_tag: TableTag,
    pub checksum: u32,
    pub offset: Offset32,
    pub length: u32,
}

impl FromData for TableRecord {
    const SIZE: usize = 4 * 4;
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(Self {
            table_tag: Tag(s.read()?),
            checksum: s.read()?,
            offset: s.read()?,
            length: s.read()?,
        })
    }
}

pub struct TableDirectory<'a> {
    pub sfnt_version: Tag,
    pub num_tables: u16,
    pub search_range: u16,
    pub entry_selector: u16,
    pub range_shift: u16,
    pub table_records: LazyArray<'a, TableRecord>,
}

impl<'a> TableDirectory<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let sfnt_version: Tag = s.read()?;
        check_sfnt_version(&sfnt_version);
        let num_tables = s.read()?;
        let search_range = s.read()?;
        let entry_selector = s.read()?;
        let range_shift = s.read()?;
        let table_records = s.read_array(num_tables as usize)?;
        Some(Self {
            sfnt_version,
            num_tables,
            search_range,
            entry_selector,
            range_shift,
            table_records,
        })
    }
}

pub struct Table<'a> {
    data: &'a [u8], // all data.
    pub table_directory: TableDirectory<'a>,
}

impl<'a> Table<'a> {
    pub fn new(data: &'a [u8]) -> Option<Self> {
        let table_directory = TableDirectory::parse(data)?;
        Some(Self {
            data,
            table_directory,
        })
    }

    pub fn get_table_record(&self, tag: &Tag) -> Option<TableRecord> {
        let (_, table_record) = self
            .table_directory
            .table_records
            .binary_search_by(|record| record.table_tag.cmp(tag))?;
        Some(table_record)
    }

    pub fn get_table_data(&self, tag: &Tag) -> Option<&'a [u8]> {
        let table_record = self.get_table_record(tag)?;
        let offset = table_record.offset as usize;
        let end = {
            let length = table_record.length as usize;
            offset.checked_add(length)?
        };
        self.data.get(offset..end)
    }

    pub fn get_name_table(&self) -> Option<NameTable<'a>> {
        let data = self.get_table_data(&NAME)?;
        let name = NameTable::parse(data);
        name
    }

    pub fn get_fvar_table(&self) -> Option<FvarTable<'a>> {
        let data = self.get_table_data(&FVAR)?;
        let fvar = FvarTable::parse(data);
        fvar
    }
}
