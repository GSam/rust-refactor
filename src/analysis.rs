use std::collections::HashMap;

use csv;

// Essentially the CSV file in a more accessible manner.
pub struct AnalysisData {
    pub var_map: HashMap<String, HashMap<String, String>>,
    pub var_ref_map: HashMap<String, Vec<HashMap<String, String>>>,
    pub type_map: HashMap<String, HashMap<String, String>>,
    pub type_ref_map: HashMap<String, Vec<HashMap<String, String>>>,
    pub func_map: HashMap<String, HashMap<String, String>>,
    pub func_ref_map: HashMap<String, Vec<HashMap<String, String>>>,
}

impl AnalysisData {
    pub fn new(analysis: &str) -> AnalysisData {
        let mut var_map = HashMap::new();
        let mut var_ref_map = HashMap::new();
        let mut type_map = HashMap::new();
        let mut type_ref_map = HashMap::new();
        let mut ctor_map = HashMap::new();
        let mut qual_type_map = HashMap::new();
        let mut func_map = HashMap::new();
        let mut func_ref_map = HashMap::new();

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
                            if !map_record.contains_key("name") {
                                let name: Vec<&str> = new_val.split("::").collect();
                                map_record.insert("name".to_string(), name[name.len()-1].to_string());
                            }
                        } else {
                            map_record.insert(key.clone(), val.clone());
                        }
                    } else {
                        break;
                    }
                }

                match &row[0][..] {
                    "crate" => {},
                    "external_crate" => {},
                    "end_external_crates" => {},
                    "function" | "function_impl" | "method_decl" => {
                        let rec = map_record.clone();
                        let copy = map_record.clone();
                        let key = rec.get("id").unwrap();
                        func_map.insert(key.clone(), map_record);

                        // Treat method impl as a function ref
                        let declid = rec.get("declid");
                        match declid {
                            Some(x) if *x != "" => {
                                if !func_ref_map.contains_key(x) {
                                    let v = vec![copy];
                                    func_ref_map.insert(x.clone(), v);
                                } else {
                                    let vec = func_ref_map.get_mut(x);
                                    vec.unwrap().push(copy);
                                }
                            },
                            _ => {}
                        }
                    },
                    "fn_ref" | "fn_call" | "method_call" => {
                        let rec = map_record.clone();
                        let refid = rec.get("refid");
                        let declid = rec.get("declid");
                        let mut key = "".to_string();

                        match refid {
                            Some(x) if *x != "" && *x != "0" => {
                                key = x.clone();
                            },
                            _ => {
                                match declid {
                                    Some(x) if *x != "" => {
                                        key = x.clone();
                                    },
                                    None | _ => {}
                                }
                            }
                        }

                        if !func_ref_map.contains_key(&key) {
                            let v = vec![map_record];
                            func_ref_map.insert(key, v);
                        } else {
                            let vec = func_ref_map.get_mut(&key);
                            vec.unwrap().push(map_record);
                        
                        }
                    },
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
                    "enum" => {
                        let rec = map_record.clone();
                        let key = rec.get("id").unwrap();
                        let q_key = rec.get("qualname").unwrap();
                        type_map.insert(key.clone(), map_record);
                        qual_type_map.insert(q_key.clone(), key.clone());
                    },
                    "struct"  => {
                        let rec = map_record.clone();
                        let key = rec.get("id").unwrap();
                        let c_key = rec.get("ctor_id").unwrap();
                        let q_key = rec.get("qualname").unwrap();
                        type_map.insert(key.clone(), map_record);
                        ctor_map.insert(c_key.clone(), key.clone());
                        qual_type_map.insert(q_key.clone(), key.clone());
                    },
                    "type_ref" | "struct_ref" | "mod_ref" => {
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

        AnalysisData{ var_map: var_map, var_ref_map: var_ref_map, type_map: type_map,
                             type_ref_map: type_ref_map, func_map: func_map, func_ref_map: func_ref_map }
    }
}
