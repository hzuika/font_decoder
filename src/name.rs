use core::fmt;
use encoding_rs;

use crate::{
    data_types::Offset16,
    decoder::{FromData, LazyArray, Stream},
    id::{EncodingID, LanguageID, NameID, PlatformID},
};

#[allow(non_snake_case)]
#[derive(Debug)]
pub struct NameRecord {
    pub platformId: PlatformID,
    pub encodingId: EncodingID,
    pub languageId: LanguageID,
    pub nameId: NameID,
    pub length: u16,
    pub stringOffset: Offset16,
}

impl FromData for NameRecord {
    const SIZE: usize = 2 * 6;
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let platform_id = s.read()?;
        Some(Self {
            platformId: PlatformID::new(platform_id),
            encodingId: EncodingID::new(s.read()?, platform_id),
            languageId: LanguageID::new(s.read()?, platform_id),
            nameId: NameID(s.read()?),
            length: s.read()?,
            stringOffset: s.read()?,
        })
    }
}

impl fmt::Display for NameRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Platform: {}, Encoding: {}, Language: {}, Name: {}",
            self.platformId, self.encodingId, self.languageId, self.nameId
        )
    }
}

#[allow(non_snake_case)]
pub struct LangTagRecord {
    pub length: u16,
    pub langTagOffset: Offset16,
}

impl FromData for LangTagRecord {
    const SIZE: usize = 4;
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(Self {
            length: s.read()?,
            langTagOffset: s.read()?,
        })
    }
}

#[allow(non_snake_case)]
pub struct NameTable<'a> {
    pub version: u16,
    pub count: u16,
    pub storageOffset: Offset16,
    pub nameRecords: LazyArray<'a, NameRecord>,
    pub langTagCount: u16,
    pub langTagRecords: LazyArray<'a, LangTagRecord>,
    pub storage: &'a [u8],
}

impl<'a> NameTable<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let version = s.read()?;
        let count = s.read()?;
        let storage_offset = s.read()?;
        let name_records = s.read_array(count as usize)?;
        let (lang_tag_count, lang_tag_records) = match version {
            0 => {
                let lang_tag_count = 0;
                let lang_tag_records = LazyArray::new(&[]);
                (lang_tag_count, lang_tag_records)
            }
            1 => {
                let lang_tag_count = s.read()?;
                let lang_tag_records = s.read_array(lang_tag_count as usize)?;
                (lang_tag_count, lang_tag_records)
            }
            _ => {
                panic!("invalid name table version {}", version);
            }
        };

        let storage = data.get(storage_offset as usize..data.len())?;
        assert_ne!(storage.len(), 0);
        Some(Self {
            version,
            count,
            storageOffset: storage_offset,
            nameRecords: name_records,
            langTagCount: lang_tag_count,
            langTagRecords: lang_tag_records,
            storage,
        })
    }

    pub fn get_string(&self, record: &NameRecord) -> Option<String> {
        let offset = record.stringOffset as usize;
        let length = record.length as usize;
        let bytes = self.storage.get(offset..offset + length)?;
        match record.platformId {
            PlatformID::Unicode(_) => {
                // UTF16 BE
                let bytes: Vec<u16> = LazyArray::new(bytes).into_iter().collect();
                String::from_utf16(&bytes).ok()
            }
            PlatformID::Mac(_) => {
                //
                match &record.encodingId {
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
                                // TODO
                                Some("not implemented".to_owned())
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

    pub fn get_strings_by_name_id(&self, name_id: NameID) -> Vec<LocalizedString> {
        let mut v = vec![];
        for name_record in self.nameRecords.into_iter().filter(|x| x.nameId == name_id) {
            let string = self.get_string(&name_record).unwrap();
            v.push(LocalizedString {
                string,
                locale: name_record.languageId.to_string(),
            })
        }
        v
    }
}

#[derive(Debug)]
pub struct LocalizedString {
    pub string: String,
    pub locale: String,
}

impl fmt::Display for LocalizedString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.string, self.locale)
    }
}

pub struct NameTableIter<'a, 'b> {
    table: &'a NameTable<'b>,
    index: usize,
}

#[allow(non_snake_case)]
#[derive(Debug)]
pub struct NameTableIterItem {
    pub platformId: PlatformID,
    pub encodingId: EncodingID,
    pub languageId: LanguageID,
    pub nameId: NameID,
    pub name: String,
}

impl<'a, 'b> Iterator for NameTableIter<'a, 'b> {
    type Item = NameTableIterItem;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.table.count as usize {
            self.index += 1;
            let record = self.table.nameRecords.get(self.index - 1)?;
            let name = self.table.get_string(&record)?;
            Some(Self::Item {
                platformId: record.platformId,
                encodingId: record.encodingId,
                languageId: record.languageId,
                nameId: record.nameId,
                name,
            })
        } else {
            None
        }
    }
}

impl<'a, 'b> IntoIterator for &'a NameTable<'b> {
    type IntoIter = NameTableIter<'a, 'b>;
    type Item = NameTableIterItem;
    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            table: self,
            index: 0,
        }
    }
}
