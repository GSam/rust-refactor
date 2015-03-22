#![feature(io)]
#![feature(box_syntax)]
#![feature(box_patterns)]
#![feature(collections)]
#![feature(rustc_private)]
#![feature(core)]
#![feature(unicode)]
#![feature(path)]

extern crate csv;
#[macro_use]
extern crate log;

pub mod rope;

use std::io::prelude::*;
use std::path::PathBuf;
use std::fs::File;
use rope::Rope;
use std::io::BufReader;

fn main() {
	let path = PathBuf::new("C:/Rust/helloworld.rs");

	let mut file = match File::open(&path) {
		Err(why) => panic!("couldn't open file {}", why.description()),
		Ok(file) => file,
	};

	let mut s = String::new();
	match file.read_to_string(&mut s) {
		Err(why) => panic!("couldn't read into string {}", why.description()),
		Ok(_) => println!("file reading ok {}", s),
	}

	let mut r: Rope = s.parse().unwrap();

	let mut analysis = BufReader::new(File::open(&"C:/Rust/dxr-temp/unknown_crate.csv").unwrap());

	for line in analysis.lines() {
		let mut rdr = csv::Reader::from_string(line.unwrap()).has_headers(false);
		for row in rdr.records() {
			let row = row.unwrap();
			println!("{:?}", row);
		}
		
	}

}
