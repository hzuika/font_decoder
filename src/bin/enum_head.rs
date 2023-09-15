use std::{fs::File, io::Read};

use font_decoder::table::{is_ttc, Collection, Table};

fn callback(table: &Table) {
    let head = table.get_head_table();
    dbg!(head);
}

// cargo run --bin enum_head
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Some(filepath) = args.get(1) {
        let mut file = File::open(filepath).unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();

        if is_ttc(&buffer).unwrap() {
            let collection = Collection::new(&buffer).unwrap();
            for i in 0..collection.header.numFonts as usize {
                callback(&collection.get(i).unwrap())
            }
        } else {
            callback(&Table::new(&buffer).unwrap())
        }
    } else {
        println!("filepath is necessary")
    }
}
