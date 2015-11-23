use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use syntax::ast::NodeId;

use strings::src_rope::Rope;

use super::{RefactorType, Response};
use analysis::AnalysisData;
use compiler;
use refactor;

// Rename functions and methods.
pub fn rename_function(input_file: &str,
                       analyzed_data: &AnalysisData,
                       new_name: &str,
                       rename_var: &str)
                       -> Result<HashMap<String, String>, Response> {
    // method calls refer to top level trait function in declid

    // rename original function

    // then rename all statically dispatched with refid = id
    // then rename all dynamically dispatched with declid = id
    // then rename all functions with declid = id
    // assuming mutual exclusion, these should all be covered in func_ref_map

    let dec_map = &analyzed_data.func_map;
    let ref_map = &analyzed_data.func_ref_map;
    let node: NodeId = rename_var.parse().unwrap();

    let mut filename = String::from(input_file);
    if let Some(decl) = dec_map.get(rename_var) {
        if let Some(file) = decl.get("file_name") {
            // FIXME: what's the point of this? We never use this value...
            filename = file.clone();
        }
    }
    
    match compiler::run_resolution(input_file.to_owned(), None, None, RefactorType::Function,
                                  String::from(new_name), node, false) {
        Ok(()) => {},
        Err(_) => { debug!("Unexpected failure!"); return Err(Response::Conflict) }
    }

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

    let output = refactor::rename_dec_and_ref(new_name, rename_var, dec_map, ref_map);

    //println!("{:?}", output);
    try!(compiler::check_reduced_graph(String::from(input_file),
                             output.iter().map(|(x,y)|
                             (x.clone(), y.clone())).collect(),
                             String::from(new_name), node));

    Ok(output)
}
