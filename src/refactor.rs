extern crate csv;

use std::collections::HashMap;

use rope::Rope;

pub fn rename_variable(input: &str, analysis: &str, new_name: &str, rename_var: &str) -> String {
	let analyzed_data = init(analysis);
	
	for (key, value) in analyzed_data.type_map.iter() {
		println!("{}: \"{}\"", *key, value.get("id").unwrap());
	}

	//for (key, value) in analyzed_data.type_ref_map.iter() {
	//	println!("{}: \"{:?}\"", *key, value);
	//}
	let dec_map = analyzed_data.var_map;
	let ref_map = analyzed_data.var_ref_map;

	return rename_dec_and_ref(input, new_name, rename_var, dec_map, ref_map);
}

pub fn rename_type(input: &str, analysis: &str, new_name: &str, rename_var: &str) -> String {
	let analyzed_data = init(analysis);

	for (key, value) in analyzed_data.type_map.iter() {
		println!("{}: \"{:?}\"", *key, value);
	}

	let dec_map = analyzed_data.type_map;
	let ref_map = analyzed_data.type_ref_map;

	return rename_dec_and_ref(input, new_name, rename_var, dec_map, ref_map);
}

struct AnalysisData {
	var_map: HashMap<String, HashMap<String, String>>,
	var_ref_map: HashMap<String, Vec<HashMap<String, String>>>,
	type_map: HashMap<String, HashMap<String, String>>,
	type_ref_map: HashMap<String, Vec<HashMap<String, String>>>,
}

fn init(analysis: &str) -> AnalysisData {
	let mut var_map = HashMap::new();
	let mut var_ref_map = HashMap::new();
	let mut type_map = HashMap::new();
	let mut type_ref_map = HashMap::new();
	let mut ctor_map = HashMap::new();
	let mut qual_type_map = HashMap::new();

	for line in analysis.lines() {
		//println!("{}", line);
		let mut rdr = csv::Reader::from_string(line).has_headers(false);
		for row in rdr.records() {
			let row = row.unwrap();
			let mut map_record = HashMap::new();
			//println!("{:?}", row);

			let mut it = row.iter();
			it.next(); // discard first value
			while let Some(key) = it.next() {
				if let Some(val) = it.next() {
					// has pair of values as expected
					if key.to_string() == "qualname" {
						let new_val = val.trim_left_matches(':');
						map_record.insert(key.clone(), new_val.to_string());
					} else {
						map_record.insert(key.clone(), val.clone());
					}
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

					if !var_ref_map.contains_key(&key) {
						let v = vec![map_record];
						var_ref_map.insert(key, v);
					} else {
						let vec = var_ref_map.get_mut(&key);
						vec.unwrap().push(map_record);
					
					}
				},
				"struct" | "enum" => {
					let rec = map_record.clone();
					let key = rec.get("id").unwrap();
					let c_key = rec.get("ctor_id").unwrap();
					let q_key = rec.get("qualname").unwrap();
					type_map.insert(key.clone(), map_record);
					ctor_map.insert(c_key.clone(), key.clone());
					qual_type_map.insert(q_key.clone(), key.clone());
				},
				"type_ref" | "struct_ref" => {
					let key = map_record.get("refid").unwrap().clone();

					if !type_ref_map.contains_key(&key) {
						let v = vec![map_record];
						type_ref_map.insert(key, v);
					} else {
						let vec = type_ref_map.get_mut(&key);
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

	// Fixup type_refs with refid = 0 and ctor_id references
	let mut to_add = Vec::new();
	for (key, value) in type_ref_map.iter() {
		if *key == "0" {
			for i in value.iter() {
				// must check qualname
				let name = i.get("qualname").unwrap();
				if qual_type_map.contains_key(name) {
					let mut modified = i.clone();
					modified.insert("refid".to_string(), qual_type_map.get(name).unwrap().clone());
					to_add.push(modified);
				}
			}
		} else if let Some(ctor) = ctor_map.get(key) {
			for i in value.iter() {
				let mut modified = i.clone();
				modified.insert("refid".to_string(), ctor.clone());
				to_add.push(modified);
			}
		}
	}

	for add in to_add.iter() {
		let key = add.get("refid").unwrap().clone();
		if !type_ref_map.contains_key(&key) {
			let v = vec![add.clone()];
			type_ref_map.insert(key, v);
		} else {
			let vec = type_ref_map.get_mut(&key);
			vec.unwrap().push(add.clone());
		
		}
	}

	return AnalysisData{ var_map: var_map, var_ref_map: var_ref_map, type_map: type_map, type_ref_map: type_ref_map }
}

fn rename_dec_and_ref(input: &str, new_name: &str, rename_var: &str,
					  dec_map: HashMap<String, HashMap<String, String>>, 
					  ref_map: HashMap<String, Vec<HashMap<String, String>>>) -> String {
	let mut ropes: Vec<Rope> = input.lines().map(|x| Rope::from_string(String::from_str(x))).collect();

	// TODO Failed an attempt to chain the declaration to the other iterator...
	let map = dec_map.get(rename_var).unwrap();
	let file_col: usize = map.get("file_col").unwrap().parse().unwrap();
	let file_line: usize = map.get("file_line").unwrap().parse().unwrap();
	let file_col_end: usize = map.get("file_col_end").unwrap().parse().unwrap();
	let file_line_end: usize = map.get("file_line_end").unwrap().parse().unwrap();
	rename(&mut ropes, file_col, file_line, file_col_end, file_line_end, new_name);

	for map in ref_map.get(rename_var).unwrap().iter() {
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
