use std::io::Read;

use font_decoder::table::{is_ttc, Table};

fn callback(table: &Table) -> Option<()> {
    let names: Vec<font_decoder::name::NameTableIterItem> =
        table.get_name_table().into_iter().collect();
    let fvar = table.get_fvar_table()?;
    for instance in &fvar.instances {
        dbg!(&instance);
        let mut it = names.iter();
        loop {
            let name = it.find(|x| x.nameId.0 == instance.subfamilyNameId);
            if let Some(name) = name {
                dbg!(name);
            } else {
                break;
            }
        }
    }

    Some(())
}

// fvar の named instance を列挙する。
// `cargo run --bin enum_named_instances`
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Some(filepath) = args.get(1) {
        let mut file = std::fs::File::open(filepath).unwrap();
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();

        if is_ttc(&buf).unwrap() {
        } else {
            callback(&Table::new(&buf).unwrap()).unwrap();
        }
    } else {
        println!("filepath is necessary");
    }
}
