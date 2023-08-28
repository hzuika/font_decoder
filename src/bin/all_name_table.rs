use std::{fs::File, io::Read};

use font_decoder::{data_types::NAME, name::NameTable, table::Table};

// cargo run --bin all_name_table
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Some(filepath) = args.get(1) {
        let mut file = File::open(filepath).unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();

        let table = Table::new(&buffer).unwrap();
        let name = table.get_table_data(&NAME).unwrap();
        let name = NameTable::parse(name).unwrap();
        for record in &name.name_records {
            dbg!(&record);
            let string = name.get_string(&record).unwrap();
            dbg!(&string);
        }
    } else {
        println!("filepath is necessary")
    }
}
