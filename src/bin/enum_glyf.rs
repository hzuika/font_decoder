use std::{fs::File, io::Read};

use font_decoder::{
    glyf::Glyph,
    table::{is_ttc, Collection, Table},
};

fn callback(table: &Table) {
    let glyf = table.get_glyf_table().unwrap();
    let cmap = table.get_cmap_table().unwrap();
    let maxp = table.get_maxp_table().unwrap();
    let head = table.get_head_table().unwrap();
    let format = head.get_loca_offset_format();
    let num_glyphs = maxp.get_number_of_glyphs();
    let loca = table.get_loca_table(format, num_glyphs).unwrap();

    for item in &cmap.header.encodingRecords {
        match cmap.get_subtable(&item) {
            Some(subtable) => {
                let glyph_id = subtable.get_glyph_id('L');
                if let Some(glyph_id) = glyph_id {
                    if let Some(range) = loca.get_glyf_range(glyph_id) {
                        let data = glyf.get_data(range).unwrap();
                        let glyph = Glyph::parse(data, loca, glyf).unwrap();
                        let it = glyph.subtable.get_glyph_points_iter();
                        for point in it.clone() {
                            dbg!(point);
                        }
                        break;
                    }
                }
            }
            None => {}
        }
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
