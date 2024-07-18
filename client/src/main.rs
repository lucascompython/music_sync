use std::{fs, io::Read};
use utils::cbf;

fn main() {
    println!("Unpacking files...");

    let mut file = fs::File::open("../server/glob.cbf").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();

    let entries = cbf::read(&mut buffer.as_slice()).unwrap();

    for entry in entries {
        fs::write(entry.name, entry.data).unwrap();
    }
}
