// named instance の座標のうち，デフォルト値かどうか判定する．
// `cargo run --bin enum_named_instances_default_value`
use std::io::Read;

use font_decoder::{
    data_types::{Fixed, Tag},
    name::NameTableIterItem,
    table::{is_ttc, Table},
};

fn callback(table: &Table) -> Option<()> {
    let names: Vec<NameTableIterItem> = table.get_name_table()?.into_iter().collect();
    let fvar = table.get_fvar_table()?;
    let into_iter = fvar.axes.into_iter();
    let default_values: Vec<(Fixed, Tag)> =
        into_iter.map(|x| (x.defaultValue, x.axisTag)).collect();
    for instance in &fvar.instances {
        let mut it = names.iter();
        loop {
            let name = it.find(|x| x.name_id.0 == instance.subfamilyNameId);
            if let Some(name) = name {
                dbg!(&name.name);
                break;
            } else {
                break;
            }
        }
        for (i, coord) in instance.coordinates.coordinates.into_iter().enumerate() {
            if coord == default_values[i].0 {
                println!("[{} ({:?})] {:?} (default)", i, default_values[i].1, coord);
            } else {
                println!("[{} ({:?})] {:?}", i, default_values[i].1, coord);
            }
        }
    }

    Some(())
}

// fvar の named instance の座標がデフォルト値なのかを含めて列挙する。
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
