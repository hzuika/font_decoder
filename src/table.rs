use crate::{
    data_types::{Offset32, TableTag, Tag},
    decoder::{FromData, LazyArray, Stream},
};

#[derive(Debug)]
pub struct TableDirectoryHeader {
    pub sfnt_version: Tag,
    pub num_tables: u16,
    pub search_range: u16,
    pub entry_selector: u16,
    pub range_shift: u16,
}

impl FromData for TableDirectoryHeader {
    const SIZE: usize = 4 + 2 * 4;
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(Self {
            sfnt_version: Tag(s.read()?),
            num_tables: s.read()?,
            search_range: s.read()?,
            entry_selector: s.read()?,
            range_shift: s.read()?,
        })
    }
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
    pub header: TableDirectoryHeader,
    pub table_records: LazyArray<'a, TableRecord>,
}

impl<'a> TableDirectory<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let header: TableDirectoryHeader = s.read()?;
        let table_records = s.read_array(header.num_tables as usize)?;
        Some(Self {
            header,
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
}
