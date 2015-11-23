use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use syntax::ast::NodeId;
use super::{RefactorType, Response};

use strings::src_rope::Rope;

use analysis::AnalysisData;
use compiler;

// Reify the lifetimes in a signature
pub fn restore_fn_lifetime(input_file: &str,
                           analyzed_data: &AnalysisData,
                           rename_var: &str)
                           -> Result<HashMap<String, String>, Response> {
    let dec_map = &analyzed_data.func_map;

    let node: NodeId = rename_var.parse().unwrap();

    let input_file_str = String::from(input_file);

    let mut filename = String::from(input_file);
    if let Some(decl) = dec_map.get(rename_var) {
        if let Some(file) = decl.get("file_name") {
            filename = file.clone();
        }
    }
    let (x,y,z,_) = match compiler::run_resolution(input_file_str, None, Some(filename.clone()), RefactorType::ReifyLifetime,
                                  String::from(rename_var), node, true) {
        Ok(()) => { debug!("Unexpected success!"); return Err(Response::Conflict) },
        Err(x) => { println!("{:?}", x); x }
    };

    let mut new_file = String::new();
    File::open(&filename).expect("Missing file").read_to_string(&mut new_file).unwrap();
    let mut rope = Rope::from_string(new_file);

    debug!("{}", filename);
    debug!("{}", rope.to_string());
    rope.src_remove(x, y);
    rope.src_insert(x, z);

    let mut output = HashMap::new();
    output.insert(filename, rope.to_string());
    debug!("{}", rope.to_string());
    Ok(output)
}

// Elide the lifetimes in a signature
pub fn elide_fn_lifetime(input_file: &str,
                         analyzed_data: &AnalysisData,
                         rename_var: &str)
                         -> Result<HashMap<String, String>, Response> {
    let dec_map = &analyzed_data.func_map;

    let node: NodeId = rename_var.parse().unwrap();

    let input_file_str = String::from(input_file);

    let mut filename = String::from(input_file);
    if let Some(decl) = dec_map.get(rename_var) {
        if let Some(file) = decl.get("file_name") {
            filename = file.clone();
        }
    }
    let filename = filename;
    debug!("{}", filename);
    let (x,y,z,_) = match compiler::run_resolution(input_file_str, None, Some(filename.clone()), RefactorType::ElideLifetime,
                                  String::from(rename_var), node, true) {
        Ok(()) => { debug!("Unexpected success!"); return Err(Response::Conflict) },
        Err(x) => { println!("{:?}", x); x }
    };

    let mut new_file = String::new();
    File::open(&filename).expect("Missing file").read_to_string(&mut new_file).unwrap();
    let mut rope = Rope::from_string(new_file);

    debug!("{}", filename);
    debug!("{}", rope.to_string());
    rope.src_remove(x, y);
    rope.src_insert(x, z);

    let mut output = HashMap::new();
    output.insert(filename, rope.to_string());
    debug!("{}", rope.to_string());
    Ok(output)
}
