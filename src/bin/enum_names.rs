use std::{fs::File, io::Read};

use font_decoder::{
    data_types::NAME,
    name::NameTable,
    table::{is_ttc, Collection, Table},
};

fn callback(table: &Table) {
    let name = table.get_table_data(&NAME).unwrap();
    let name = NameTable::parse(name).unwrap();
    for record in &name.name_records {
        dbg!(&record);
        let string = name.get_string(&record);
        dbg!(&string);
    }
}

// cargo run --bin enum_names
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Some(filepath) = args.get(1) {
        let mut file = File::open(filepath).unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();

        if is_ttc(&buffer).unwrap() {
            let collection = Collection::new(&buffer).unwrap();
            for i in 0..collection.header.num_fonts as usize {
                callback(&collection.get(i).unwrap())
            }
        } else {
            callback(&Table::new(&buffer).unwrap())
        }
    } else {
        println!("filepath is necessary")
    }
}
