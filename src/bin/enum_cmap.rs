use std::io::Read;

use font_decoder::table::Table;

// `cargo run --bin enum_cmap`
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = &args[1];
    let mut file = std::fs::File::open(path).unwrap();
    let mut buf = vec![];
    file.read_to_end(&mut buf).unwrap();
    let table = Table::new(&buf).unwrap();
    let cmap = table.get_cmap_table().unwrap();
    for item in &cmap.header.encodingRecords {
        dbg!(&item);
        match cmap.get_subtable(&item) {
            Some(subtable) => {
                let map = subtable.get_code_point_glyph_id_map();
                let mut map: Vec<(u16, u16)> = map.into_iter().collect();
                dbg!(map.len());
                map.sort_by(|a, b| a.0.cmp(&b.0));
                for (code_point, glyph_id) in map {
                    let char = std::char::from_u32(code_point as u32);
                    match char {
                        Some(char) => {
                            println!("U+{:06X} ('{:?}') => {}", code_point, char, glyph_id);
                        }
                        None => {
                            panic!("invalid unicode character or surrogate {:x}", code_point);
                        }
                    }
                }
            }
            None => {}
        }
    }
}
