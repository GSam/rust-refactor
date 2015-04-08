extern crate csv;

use std::collections::HashMap;

use rope::Rope;

pub fn rename_variable(input: &str, analysis: &str, new_name: &str, rename_var: &str) -> String {
	let mut ropes: Vec<Rope> = input.lines().map(|x| Rope::from_string(String::from_str(x))).collect();

	let mut var_map = HashMap::new();
	let mut var_ref_map = HashMap::new();
	let mut type_map = HashMap::new();
	let mut type_ref_map = HashMap::new();

	for line in analysis.lines() {
		println!("{}", line);
		let mut rdr = csv::Reader::from_string(line).has_headers(false);
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
				"struct" | "enum" => {
					let key = map_record.get("id").unwrap().clone();
					type_map.insert(key, map_record);
				},
				"type_ref" => {
					let key = map_record.get("refid").unwrap().clone();

					if !type_ref_map.contains_key("refid") {
						let v = vec![map_record];
						type_ref_map.insert(key, v);
					} else {
						let vec = var_ref_map.get_mut(&key);
						vec.unwrap().push(map_record);
					
					}
				},
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
	
	// TODO Failed an attempt to chain the declaration to the other iterator...
	let map = var_map.get(rename_var).unwrap();
	let file_col: usize = map.get("file_col").unwrap().parse().unwrap();
	let file_line: usize = map.get("file_line").unwrap().parse().unwrap();
	let file_col_end: usize = map.get("file_col_end").unwrap().parse().unwrap();
	let file_line_end: usize = map.get("file_line_end").unwrap().parse().unwrap();
	rename(&mut ropes, file_col, file_line, file_col_end, file_line_end, new_name);


	for map in var_ref_map.get(rename_var).unwrap().iter() {
		let file_col: usize = map.get("file_col").unwrap().parse().unwrap();
		let file_line: usize = map.get("file_line").unwrap().parse().unwrap();
		let file_col_end: usize = map.get("file_col_end").unwrap().parse().unwrap();
		let file_line_end: usize = map.get("file_line_end").unwrap().parse().unwrap();
		
		println!("{} {} {} {}", file_col, file_line, file_col_end, file_line_end);
		rename(&mut ropes, file_col, file_line, file_col_end, file_line_end, new_name);
	}

	let mut answer = String::new();
	for rope in &ropes {
		answer.push_str(&rope.to_string());
		answer.push_str("\n");
	}

	return answer;
}

fn rename(ropes: &mut Vec<Rope>, file_col:usize , file_line:usize, file_col_end: usize, file_line_end: usize, new_name: &str) {
	let to_change = &mut ropes[file_line-1..file_line_end];
	let length = to_change.len();

	if file_line == file_line_end {
		to_change[0].src_remove(file_col, file_col_end);
	} else {
		for i in 0..length {
			let len = to_change[i].len();
			let line = &mut to_change[i];
			match i {
				0 => line.src_remove(file_col, len),
				x if x == length => line.src_remove(0, file_col_end),
				_ => line.src_remove(0, len)
			}
		}
	}

	to_change[0].src_insert(file_col, new_name.to_string());
}
