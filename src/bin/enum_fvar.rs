use std::{fs::File, io::Read};

use font_decoder::{data_types::FVAR, fvar::FvarTable, table::Table};

fn callback(table: &Table) {
    let fvar = table.get_table_data(&FVAR);
    match fvar {
        Some(fvar) => {
            let fvar = FvarTable::parse(fvar).unwrap();
            dbg!(fvar.axes);
            dbg!(fvar.instances);
        }
        None => {}
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Some(filepath) = args.get(1) {
        let mut file = File::open(filepath).unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();

        callback(&Table::new(&buffer).unwrap());
    } else {
        println!("filepath is required");
    }
}
