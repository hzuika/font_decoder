use std::{fs::File, io::Read};

use font_decoder::{
    loca::LocaTable,
    table::{is_ttc, Collection, Table},
};

fn callback(table: &Table) {
    let glyf = table.get_glyf_table().unwrap();
    println!("glyf tableの長さ {}", glyf.0.len());
    let maxp = table.get_maxp_table();
    let head = table.get_head_table();
    let format = head.get_loca_offset_format();
    let num_glyphs = maxp.get_number_of_glyphs();
    let loca = table.get_loca_table(format, num_glyphs).unwrap();
    match &loca {
        LocaTable::Long(_) => println!("long loca"),
        LocaTable::Short(_) => println!("short loca"),
    }
    for i in 0..loca.len() {
        println!(
            "glyph id [{}], offset = [{}], range = [{:?}]",
            i,
            loca.at(i),
            loca.get_glyf_range(i as u16)
        );
    }
}

// cargo run --bin enum_names
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Some(filepath) = args.get(1) {
        let mut file = File::open(filepath).unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();

        if is_ttc(&buffer) {
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
