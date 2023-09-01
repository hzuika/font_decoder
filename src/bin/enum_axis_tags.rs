use std::io::Read;

use font_decoder::{
    name::NameTableIterItem,
    table::{is_ttc, Table},
};

fn callback(table: &Table) -> Option<()> {
    let fvar = table.get_fvar_table()?;
    let names: Vec<NameTableIterItem> = table.get_name_table()?.into_iter().collect();
    for (i, axis) in (&fvar.axes).into_iter().enumerate() {
        dbg!(i);
        dbg!(axis.axisTag);
        let mut it = names.iter();
        loop {
            let name = it.find(|x| x.name_id.0 == axis.axisNameId);
            if let Some(name) = name {
                dbg!(&name.name);
            } else {
                break;
            }
        }
    }
    Some(())
}

// fvar の axisTag を順番通りに列挙する．
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Some(filepath) = args.get(1) {
        let mut file = std::fs::File::open(filepath).unwrap();
        let mut buf = vec![];
        file.read_to_end(&mut buf).unwrap();
        if is_ttc(&buf).unwrap() {
        } else {
            callback(&Table::new(&buf).unwrap());
        }
    } else {
    }
}
