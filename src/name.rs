use core::fmt;
use encoding_rs;

use crate::{
    data_types::Offset16,
    decoder::{FromData, LazyArray, Stream},
    id::{EncodingID, LanguageID, NameID, PlatformID},
};

pub struct NameTableHeader {
    pub version: u16,
    pub count: u16,
    pub storage_offset: Offset16,
}

impl FromData for NameTableHeader {
    const SIZE: usize = 6;
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(Self {
            version: s.read()?,
            count: s.read()?,
            storage_offset: s.read()?,
        })
    }
}

#[derive(Debug)]
pub struct NameRecord {
    pub platform_id: PlatformID,
    pub encoding_id: EncodingID,
    pub language_id: LanguageID,
    pub name_id: NameID,
    pub length: u16,
    pub string_offset: Offset16,
}

impl FromData for NameRecord {
    const SIZE: usize = 2 * 6;
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let platform_id = s.read()?;
        Some(Self {
            platform_id: PlatformID::new(platform_id),
            encoding_id: EncodingID::new(s.read()?, platform_id),
            language_id: LanguageID::new(s.read()?, platform_id),
            name_id: NameID(s.read()?),
            length: s.read()?,
            string_offset: s.read()?,
        })
    }
}

impl fmt::Display for NameRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Platform: {}, Encoding: {}, Language: {}, Name: {}",
            self.platform_id, self.encoding_id, self.language_id, self.name_id
        )
    }
}

pub struct LangTagRecord {
    pub length: u16,
    pub lang_tag_offset: Offset16,
}

impl FromData for LangTagRecord {
    const SIZE: usize = 4;
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(Self {
            length: s.read()?,
            lang_tag_offset: s.read()?,
        })
    }
}

pub struct NameTable<'a> {
    pub header: NameTableHeader,
    pub name_records: LazyArray<'a, NameRecord>,
    pub lang_tag_count: u16,
    pub lang_tag_records: LazyArray<'a, LangTagRecord>,
    pub storage: &'a [u8],
}

impl<'a> NameTable<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let header: NameTableHeader = s.read()?;
        match header.version {
            0 => {
                let name_records = s.read_array(header.count as usize)?;
                let lang_tag_count = 0;
                let lang_tag_records = LazyArray::new(&[]);
                let storage = data.get(header.storage_offset as usize..data.len())?;
                assert_ne!(storage.len(), 0);
                Some(Self {
                    header,
                    lang_tag_count,
                    lang_tag_records,
                    name_records,
                    storage,
                })
            }
            1 => {
                let name_records = s.read_array(header.count as usize)?;
                let lang_tag_count = s.read()?;
                let lang_tag_records = s.read_array(lang_tag_count as usize)?;
                let storage = data.get(header.storage_offset as usize..data.len())?;
                assert_ne!(storage.len(), 0);
                Some(Self {
                    header,
                    lang_tag_count,
                    lang_tag_records,
                    name_records,
                    storage,
                })
            }
            _ => {
                panic!("invalid name table version {}", header.version);
            }
        }
    }

    pub fn get_string(&self, record: &NameRecord) -> Option<String> {
        let offset = record.string_offset as usize;
        let length = record.length as usize;
        let bytes = self.storage.get(offset..offset + length)?;
        match record.platform_id {
            PlatformID::Unicode(_) => {
                // UTF16 BE
                let bytes: Vec<u16> = LazyArray::new(bytes).into_iter().collect();
                String::from_utf16(&bytes).ok()
            }
            PlatformID::Mac(_) => {
                //
                match &record.encoding_id {
                    EncodingID::Mac(id) => {
                        match id.0 {
                            0 => {
                                // Roman is UTF8?
                                let (cow, _encoding_used, _had_errors) =
                                    encoding_rs::MACINTOSH.decode(bytes.into());
                                Some(cow.into())
                            }
                            1 => {
                                // Japanese is Shift JIS?
                                let (cow, _encoding_used, _had_errors) =
                                    encoding_rs::SHIFT_JIS.decode(bytes.into());
                                Some(cow.into())
                            }
                            _ => {
                                todo!()
                            }
                        }
                    }
                    _ => {
                        panic!("unreachable")
                    }
                }
            }
            PlatformID::Win(_) => {
                // UTF16 BE
                let bytes: Vec<u16> = LazyArray::new(bytes).into_iter().collect();
                String::from_utf16(&bytes).ok()
            }
        }
    }
}

pub struct NameTableIter<'a> {
    table: &'a NameTable<'a>,
    index: usize,
}

#[derive(Debug)]
pub struct NameTableIterItem {
    pub platform_id: PlatformID,
    pub encoding_id: EncodingID,
    pub language_id: LanguageID,
    pub name_id: NameID,
    pub name: String,
}

impl<'a> Iterator for NameTableIter<'a> {
    type Item = NameTableIterItem;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.table.header.count as usize {
            self.index += 1;
            let record = self.table.name_records.get(self.index - 1)?;
            let name = self.table.get_string(&record)?;
            Some(Self::Item {
                platform_id: record.platform_id,
                encoding_id: record.encoding_id,
                language_id: record.language_id,
                name_id: record.name_id,
                name,
            })
        } else {
            None
        }
    }
}

impl<'a> IntoIterator for &'a NameTable<'a> {
    type IntoIter = NameTableIter<'a>;
    type Item = NameTableIterItem;
    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            table: self,
            index: 0,
        }
    }
}
