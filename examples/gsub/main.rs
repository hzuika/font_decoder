use font_decoder::table::Table;

fn main() {
    let data = include_bytes!("../../NunitoSans_10pt-Black.ttf");
    let table = Table::new(data).unwrap();
    if let Some(gsub) = table.get_gsub_table() {
        for feature_record in gsub.feature_list.featureRecords {
            println!("{}", feature_record.featureTag);
        }
    }
}
