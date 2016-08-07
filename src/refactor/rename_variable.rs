use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use syntax::ast::NodeId;use super::{RefactorType, Response};

use strings::src_rope::Rope;

use analysis::AnalysisData;
use compiler;
use refactor;

// Renames a variable (or variable-like construct).
// This includes constants, fields, and function parameters.
pub fn rename_variable(input_file: &str,
                       analyzed_data: &AnalysisData,
                       new_name: &str,
                       rename_var: &str)
                       -> Result<HashMap<String, String>, Response> {
    let dec_map = &analyzed_data.var_map;
    let ref_map = &analyzed_data.var_ref_map;
    
    let mut filename = String::from(input_file);
    if let Some(decl) = dec_map.get(rename_var) {
        if let Some(file) = decl.get("file_name") {
            // FIXME: what's the point of this? We never use this value...
            filename = file.clone();
        }
    }

    // Check if renaming will cause conflicts
    let node: NodeId = rename_var.parse().unwrap();

    match compiler::run_resolution(String::from(input_file), None,
            None, RefactorType::Variable, String::from(new_name), node, false)
    {
        Ok(()) => {
            debug!("GOOD");
        },
        Err(_) => { debug!("BAD"); return Err(Response::Conflict); }
    }
    match dec_map.get(rename_var) {
        Some(x) => {
            for (_, value) in dec_map.iter() {
                let name = value.get("name").unwrap();
                if x.get("scopeid") == value.get("scopeid") &&
                    name == &new_name {
                    // Conflict present:
                    // May still be ok if there is no references to it
                    // However, standalone blocks won't be detected + macros
                    // Will also restrict if reference is on same line as renaming

                    if let Some(references) = ref_map.get(rename_var) {
                        for map in references.iter() {
                            let filename = map.get("file_name").unwrap();

                            let mut file = match File::open(&filename) {
                                Err(why) => panic!("couldn't open file {}", why),
                                    Ok(file) => file,
                            };
                            let mut file_str = String::new();
                            let _ = file.read_to_string(&mut file_str);
                            let file_str = &file_str[..];
                            let mut ropes: Vec<Rope> = file_str.lines().map(|x| Rope::from_string(String::from(x))).collect();
                            let file_col: usize = map.get("file_col").unwrap().parse().unwrap();
                            let file_line: usize = map.get("file_line").unwrap().parse().unwrap();
                            let file_col_end: usize = map.get("file_col_end").unwrap().parse().unwrap();
                            let file_line_end: usize = map.get("file_line_end").unwrap().parse().unwrap();

                            //let _ = writeln!(&mut stderr(), "{} {} {} {}", file_col, file_line, file_col_end, file_line_end);
                            refactor::rename(&mut ropes, file_col, file_line, file_col_end, file_line_end, new_name);
                            let mut answer = String::new();
                            let mut count = ropes.len();
                            for rope in &ropes {
                                answer.push_str(&rope.to_string());
                                if count > 1 {
                                    answer.push_str("\n");
                                    count -= 1;
                                }
                            }

                            match compiler::run_resolution(String::from(input_file), Some(vec![(String::from(filename.clone()), answer)]),
                                                          None, RefactorType::Variable, String::from(new_name),
                                                          node, true) {
                                Ok(()) => {
                                    debug!("Unexpected success!");
                                    // Check for conflicts 
                                    return Err(Response::Conflict); 
                                },
                                Err(_) => { debug!("Expected failure!");}
                            }
                        }
                    }

                    /*let id = value.get("id").unwrap();
                    let line_no: i32 = value.get("file_line").unwrap().parse().unwrap();
                    if let Some(refs) = ref_map.get(id) {
                        for record in refs.iter() {
                            let line: i32 = record.get("file_line").unwrap().parse().unwrap();
                            if line >= line_no {
                                // Affected reference
                                return Err(Response::Conflict); //input.to_string();
                            }
                        }
                    }*/
                }
            }
        },
        _ => { return Err(Response::Conflict); } //input.to_string(); }
    }

    let output = refactor::rename_dec_and_ref(new_name, rename_var, dec_map, ref_map);

    try!(compiler::check_reduced_graph(String::from(input_file),
                             output.iter().map(|(x,y)|
                             (x.clone(), y.clone())).collect(),
                             String::from(new_name), node));

    Ok(output)
}
