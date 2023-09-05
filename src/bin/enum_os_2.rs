use std::io::Read;

use font_decoder::table::Table;

// `cargo run --bin enum_os_2`
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = &args[1];
    let mut file = std::fs::File::open(path).unwrap();
    let mut buf = vec![];
    file.read_to_end(&mut buf).unwrap();
    let table = Table::new(&buf).unwrap();
    let os2 = table.get_os2_table().unwrap();
    dbg!(os2);
}
