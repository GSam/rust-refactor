#![feature(io)]
#![feature(box_syntax)]
#![feature(box_patterns)]
#![feature(collections)]
#![feature(rustc_private)]
#![feature(core)]
#![feature(path)]

#[macro_use]
extern crate log;

extern crate refactor;

use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

fn main() {
	let args = env::args();

	if args.len() < 6 {
		println!("Not enough args: <var|type|fn> <analysis> <src> <var> <outvar>");
		println!("var: <nodeid> | <name>:<row or -1>:<col or -1>");
		return;
	}

	let args: Vec<_> = args.collect();
	let path = PathBuf::new(&args[3]);
	let mut s;
	let mut rename_var = &args[4];

	let mut file = match File::open(&path) {
		Err(why) => panic!("couldn't open file {}", why.description()),
		Ok(file) => file,
	};
	let mut file_str = String::new();
	file.read_to_string(&mut file_str);

	let mut analysis = match File::open(&args[2]) {
		Err(why) => panic!("couldn't open file {}", why.description()),
		Ok(file) => file,
	};
	let mut analysis_str = String::new();
	analysis.read_to_string(&mut analysis_str);


	let v: Vec<_> = args[4].split(":").collect();
	if v.len() == 3 {
		s = refactor::refactor::identify_id(path.file_name().unwrap().to_str().unwrap(), &analysis_str,
											v[0], v[1].parse().unwrap(), 
											v[2].parse().unwrap());
		println!("NODE ID: {}", s);
		rename_var = &s;
	}

	match &*args[1] {
		"var" => {
			let result = refactor::refactor::rename_variable(&file_str, &analysis_str, &args[5], rename_var);
			match result {
				Ok(x) => println!("{}", x),
				Err(x) => println!("{:?}", x)
			}
		},
		"type" => {
			let result = refactor::refactor::rename_type(&file_str, &analysis_str, &args[5], rename_var);
			match result {
				Ok(x) => println!("{}", x),
				Err(x) => println!("{:?}", x)
			}
		},
		"fn" => {
			let result = refactor::refactor::rename_function(&file_str, &analysis_str, &args[5], rename_var);
			match result {
				Ok(x) => println!("{}", x),
				Err(x) => println!("{:?}", x)
			}
		},
		_ => {
			println!("Unknown rename function.");
		}
	}
}

