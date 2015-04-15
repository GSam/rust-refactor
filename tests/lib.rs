#![feature(io)]

extern crate refactor;

use std::fs::File;
use std::io::prelude::*;
use refactor::refactor::Response;

fn read_to_string(filename: &str) -> String {
	let mut file = match File::open(filename) {
		Err(why) => panic!("couldn't open file {}", why.description()),
		Ok(file) => file,
	};

	let mut output = String::new();
	file.read_to_string(&mut output);

	return output;
}

#[test]
fn working_rename_1() {
	let input = read_to_string("tests/rename/basic_rename.rs");
	let output = read_to_string("tests/rename/working_rename_1_out.rs");
	let analysis = read_to_string("tests/rename/basic_rename.csv");

	match refactor::refactor::rename_variable(&input, &analysis, "hello", "9") {
		Ok(x) => assert_eq!(output.trim(), x.trim()),
		Err(_) => assert!(false)
	}
}

#[test]
fn working_rename_2() {
	let input = read_to_string("tests/rename/basic_rename.rs");
	let output = read_to_string("tests/rename/working_rename_2_out.rs");
	let analysis = read_to_string("tests/rename/basic_rename.csv");

	match refactor::refactor::rename_variable(&input, &analysis, "hello", "17") {
		Ok(x) => assert_eq!(output.trim(), x.trim()),
		Err(_) => assert!(false)
	}
}

#[test]
fn prevented_rename_1() {
	let input = read_to_string("tests/rename/basic_rename.rs");
	let analysis = read_to_string("tests/rename/basic_rename.csv");

	match refactor::refactor::rename_variable(&input, &analysis, "j", "36") {
		Ok(_) => assert!(false),
		Err(x) => assert_eq!(Response::Conflict, x)
	}
}

#[test]
fn prevented_rename_2() {
	let input = read_to_string("tests/rename/basic_rename.rs");
	let analysis = read_to_string("tests/rename/basic_rename.csv");

	match refactor::refactor::rename_variable(&input, &analysis, "x", "36") {
		Ok(_) => assert!(false),
		Err(x) => assert_eq!(Response::Conflict, x)
	}
}
