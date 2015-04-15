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

	if args.len() < 5 {
		println!("Not enough args: <analysis> <src> <var> <outvar>");
		println!("var: <nodeid> | <name>:<row or -1>:<col or -1>");
		return;
	}

	let args: Vec<_> = args.collect();
	let path = PathBuf::new(&args[2]);
	let mut s;
	let mut rename_var = &args[3];

	let mut file = match File::open(&path) {
		Err(why) => panic!("couldn't open file {}", why.description()),
		Ok(file) => file,
	};
	let mut file_str = String::new();
	file.read_to_string(&mut file_str);

	let mut analysis = match File::open(&args[1]) {
		Err(why) => panic!("couldn't open file {}", why.description()),
		Ok(file) => file,
	};
	let mut analysis_str = String::new();
	analysis.read_to_string(&mut analysis_str);


	let v: Vec<_> = args[3].split(":").collect();
	if v.len() == 3 {
		s = refactor::refactor::identify_id(path.file_name().unwrap().to_str().unwrap(), &analysis_str,
											v[0], v[1].parse().unwrap(), 
											v[2].parse().unwrap());
		println!("NODE ID: {}", s);
		rename_var = &s;
	}

	if args.len() == 6 {
		if args[5] == "type" {
			let result = refactor::refactor::rename_type(&file_str, &analysis_str, &args[4], rename_var);
			match result {
				Ok(x) => println!("{}", x),
				Err(x) => println!("{:?}", x)
			}
		} else {
			let result = refactor::refactor::rename_function(&file_str, &analysis_str, &args[4], rename_var);
			match result {
				Ok(x) => println!("{}", x),
				Err(x) => println!("{:?}", x)
			}
		}
	} else {
		let result = refactor::refactor::rename_variable(&file_str, &analysis_str, &args[4], rename_var);
		match result {
			Ok(x) => println!("{}", x),
			Err(x) => println!("{:?}", x)
		}
	}
}

