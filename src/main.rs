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
		return;
	}

	let args:Vec<_> = args.collect();
	let path = PathBuf::new(&args[2]);
	let rename_var = &args[3];

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


	if args.len() == 6 {
		println!("{}", refactor::refactor::rename_type(&file_str, &analysis_str, &args[4], rename_var));
	} else {
		println!("{}", refactor::refactor::rename_variable(&file_str, &analysis_str, &args[4], rename_var));
	}
}

