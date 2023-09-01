use std::io::Read;

use font_decoder::{id::NameID, table::Table};

// `cargo run --bin enum_stat`
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = &args[1];
    let mut file = std::fs::File::open(path).unwrap();
    let mut buf = vec![];
    file.read_to_end(&mut buf).unwrap();
    let table = Table::new(&buf).unwrap();
    let stat = table.get_stat_table();
    let name = table.get_name_table().unwrap();
    match stat {
        Some(stat) => {
            dbg!(&stat.header);
            let mut design_axes = vec![];
            for (i, design_axis) in stat.designAxes.into_iter().enumerate() {
                dbg!(i, &design_axis);
                let name_id = design_axis.axisNameID;
                let localized_strings = name.get_strings_by_name_id(NameID(name_id));
                for localized_string in &localized_strings {
                    println!("{}", localized_string);
                }
                design_axes.push(design_axis);
            }
            for (i, axis_value_table) in stat.get_axis_value_table_iter().enumerate() {
                dbg!(i, &axis_value_table);
                let name_id = axis_value_table.get_value_name_id();
                let localized_strings = name.get_strings_by_name_id(NameID(name_id));
                for localized_string in &localized_strings {
                    println!("{}", localized_string);
                }
                for axis_index in axis_value_table.get_axis_indices() {
                    println!("{}", design_axes[axis_index as usize].axisTag);
                }
            }
        }
        None => {}
    }
}
