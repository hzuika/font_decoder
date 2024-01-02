use std::io::Read;

use font_decoder::table::{is_ttc, Table};

fn callback(table: &Table) {
    if let Some(gsub) = table.get_gsub_table() {
        for feature_record in gsub.feature_list.featureRecords {
            println!("{}", feature_record.featureTag);
        }
    }
}

fn main() {
    // let data = include_bytes!("../../NunitoSans_10pt-Black.ttf");
    let args: Vec<String> = std::env::args().collect();
    if let Some(filepath) = args.get(1) {
        let mut file = std::fs::File::open(filepath).unwrap();
        let mut buf = vec![];
        file.read_to_end(&mut buf).unwrap();
        if is_ttc(&buf) {
        } else {
            callback(&Table::new(&buf).unwrap());
        }
    } else {
    }
}
