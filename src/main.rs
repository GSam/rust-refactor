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
use std::collections::HashMap;

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

	let mut var_map = HashMap::new();
	let mut var_ref_map = HashMap::new();
	for line in analysis.lines() {
		let mut rdr = csv::Reader::from_string(line.unwrap()).has_headers(false);
		for row in rdr.records() {
			let row = row.unwrap();
			let mut map_record = HashMap::new();
			println!("{:?}", row);

			let mut it = row.iter();
			it.next(); // discard first value
			while let Some(key) = it.next() {
				if let Some(val) = it.next() {
					// has pair of values as expected
					map_record.insert(key.clone(), val.clone());
				} else {
					break;
				}
			}

			match row[0].as_slice() {
				"crate" => {},
				"external_crate" => {},
				"end_external_crates" => {},
				"function" => {},
				"function_ref" => {},
				"variable" => {
					let key = map_record.get("id").unwrap().clone();
					var_map.insert(key, map_record);
				},
				"var_ref" => {
					let key = map_record.get("refid").unwrap().clone();

					if !var_ref_map.contains_key("refid") {
						let v = vec![map_record];
						var_ref_map.insert(key, v);
					} else {
						let vec = var_ref_map.get_mut(&key);
						vec.unwrap().push(map_record);
					
					}
				},
				"type" => {},
				"type_ref" => {},
				"module" => {},
				"module_ref" => {},
				"module_alias" => {},
				"unknown_ref" => {},
				_ => {}
			}
		}
		
	}


	for (key, value) in var_map.iter() {
		println!("{}: \"{}\"", *key, value.get("id").unwrap());
	}

	for (key, value) in var_ref_map.iter() {
		println!("{}: \"{:?}\"", *key, value);
	}
}
