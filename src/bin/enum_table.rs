use std::{fs::File, io::Read};

use font_decoder::table::{is_ttc, Collection, Table};

fn callback(table: &Table) {
    for record in &table.table_directory.table_records {
        let start = record.offset as usize;
        let end = start + record.length as usize;
        println!("{:?} = [{}..{}]", record.table_tag, start, end);
    }
}

// cargo run --bin enum_table
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Some(filepath) = args.get(1) {
        let mut file = File::open(filepath).unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();

        if is_ttc(&buffer).unwrap() {
            let collection = Collection::new(&buffer).unwrap();
            println!("collection = [0..{}]", buffer.len());
            for i in 0..collection.header.num_fonts as usize {
                println!(
                    "table directory[{}] = [{}..]",
                    i,
                    collection.header.table_directory_offsets.get(i).unwrap()
                );
                callback(&collection.get(i).unwrap())
            }
        } else {
            callback(&Table::new(&buffer).unwrap())
        }
    } else {
        println!("filepath is necessary")
    }
}
