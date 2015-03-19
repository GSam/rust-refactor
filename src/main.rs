extern crate csv;

fn main() {
    let mut rdr = csv::Reader::from_file("C:/Rust/dxr-temp/unknown_crate.csv").unwrap();
    for record in rdr.records() {
        let r = record.unwrap();
        println!("{:?}", r);
    }
}
