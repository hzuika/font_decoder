use crate::{
    cmap::CmapTable,
    data_types::{Offset32, TableTag, Tag, CMAP, FVAR, NAME, OS_2, STAT},
    decoder::{FromData, LazyArray, Stream},
    fvar::FvarTable,
    name::NameTable,
    os_2::OS2Table,
    stat::StatTable,
};

#[allow(non_snake_case)]
pub struct TTCHeader<'a> {
    pub ttcTag: Tag, // Font Collection ID string: 'ttcf' (used for fonts with CFF or CFF2 outlines as well as TrueType outlines)
    pub majorVersion: u16, // Major version of the TTC Header, = 1.
    pub minorVersion: u16, // Minor version of the TTC Header, = 0.
    pub numFonts: u32, // Number of fonts in TTC
    pub tableDirectoryOffsets: LazyArray<'a, Offset32>, // Array of offsets to the TableDirectory for each font from the beginning of the file
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
            ttcTag: ttc_tag,
            majorVersion: major_version,
            minorVersion: minor_version,
            numFonts: num_fonts,
            tableDirectoryOffsets: table_directory_offsets,
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
        let offset = self.header.tableDirectoryOffsets.get(index)? as usize;
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

#[allow(non_snake_case)]
#[derive(Debug)]
pub struct TableRecord {
    pub tableTag: TableTag,
    pub checksum: u32,
    pub offset: Offset32,
    pub length: u32,
}

impl FromData for TableRecord {
    const SIZE: usize = 4 * 4;
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(Self {
            tableTag: Tag(s.read()?),
            checksum: s.read()?,
            offset: s.read()?,
            length: s.read()?,
        })
    }
}

#[allow(non_snake_case)]
pub struct TableDirectory<'a> {
    pub sfntVersion: Tag,
    pub numTables: u16,
    pub searchRange: u16,
    pub entrySelector: u16,
    pub rangeShift: u16,
    pub tableRecords: LazyArray<'a, TableRecord>,
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
            sfntVersion: sfnt_version,
            numTables: num_tables,
            searchRange: search_range,
            entrySelector: entry_selector,
            rangeShift: range_shift,
            tableRecords: table_records,
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
            .tableRecords
            .binary_search_by(|record| record.tableTag.cmp(tag))?;
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

    pub fn get_stat_table(&self) -> Option<StatTable<'a>> {
        let data = self.get_table_data(&STAT)?;
        let stat = StatTable::parse(data);
        stat
    }

    pub fn get_cmap_table(&self) -> Option<CmapTable<'a>> {
        let data = self.get_table_data(&CMAP)?;
        let cmap = CmapTable::parse(data);
        cmap
    }

    pub fn get_os2_table(&self) -> Option<OS2Table> {
        let data = self.get_table_data(&OS_2)?;
        let os2 = OS2Table::parse(data);
        os2
    }
}
