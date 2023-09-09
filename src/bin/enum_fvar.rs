use std::{fs::File, io::Read};

use font_decoder::{data_types::FVAR, fvar::FvarTable, id::NameID, table::Table};

fn callback(table: &Table) {
    let fvar = table.get_table_data(&FVAR);
    let name = table.get_name_table();
    match fvar {
        Some(fvar) => {
            let fvar = FvarTable::parse(fvar).unwrap();
            for (i, axis) in fvar.axes.into_iter().enumerate() {
                dbg!(i, &axis);
                for string in name.get_strings_by_name_id(NameID(axis.axisNameId)) {
                    println!("{}", string);
                }
            }
            for (i, instance) in fvar.instances.into_iter().enumerate() {
                dbg!(i, &instance);
                for string in name.get_strings_by_name_id(NameID(instance.subfamilyNameId)) {
                    println!("{}", string);
                }
            }
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
