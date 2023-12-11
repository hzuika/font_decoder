use std::{fs::File, io::Read, path::PathBuf};

use enum_fonts::os::enum_font_filepaths;
use font_decoder::{
    data_types::MORX,
    decoder::{FromData, Stream},
    morx::{ChainHeader, FeatureTable, MorxHeader, MorxSubtableHeader, MorxSubtableType},
    table::{is_ttc, Collection, Table},
};

fn callback(data: &[u8], filepath: &str) {
    let mut s = Stream::new(data);
    let header: MorxHeader = s.read().unwrap();
    for _chain_index in 0..header.nChains {
        let chain_header: ChainHeader = s.read().unwrap();
        let chain_last_offset =
            s.get_offset() + chain_header.chainLength as usize - ChainHeader::SIZE;

        for _feature_index in 0..chain_header.nFeatureEntries {
            let _feature: FeatureTable = s.read().unwrap();
        }

        for _subtable_index in 0..chain_header.nSubtables {
            let subtable_header: MorxSubtableHeader = s.read().unwrap();
            let subtable_last_offset =
                s.get_offset() + subtable_header.length as usize - MorxSubtableHeader::SIZE;

            let subtable_type = subtable_header.get_type();

            // println!("\t{:?}", &subtable_type);

            match subtable_type {
                MorxSubtableType::Rearrangement => {
                    // let stx_header: STXHeader = s.read().unwrap();
                    println!("{}", filepath);
                }
                _ => {}
            }

            s.set_offset(subtable_last_offset);
        }

        s.set_offset(chain_last_offset);
    }
}

fn main() {
    // let args: Vec<String> = std::env::args().collect();
    // let filepath = &args[1];

    let filepaths = enum_font_filepaths();
    let mut filepaths: Vec<PathBuf> = filepaths.into_iter().collect();
    filepaths.sort();
    for filepath in filepaths {
        // println!("{}", &filepath.to_str().unwrap());
        let mut file = File::open(&filepath).unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();

        if is_ttc(&buffer).unwrap() {
            let collection = Collection::new(&buffer).unwrap();
            let table = collection.get(0).unwrap();
            if let Some(data) = table.get_table_data(&MORX) {
                let filepath = filepath.to_str().unwrap();
                callback(data, filepath);
            }
        } else {
            let table = Table::new(&buffer).unwrap();
            if let Some(data) = table.get_table_data(&MORX) {
                let filepath = filepath.to_str().unwrap();
                callback(data, filepath);
            }
        }
    }
}
