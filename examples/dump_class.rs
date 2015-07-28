extern crate classreader;

use classreader::ClassReader;
use std::env;

pub fn main() {
    let file_name = env::args().nth(1).expect("usage: class_reader <class file>");

    let class = ClassReader::new_from_path(&file_name).unwrap();
    println!("{:?}", class);
}
