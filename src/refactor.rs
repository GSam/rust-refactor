
extern crate csv;

use getopts;
use rustc::ast_map;
use rustc::ast_map::Node::*;
use rustc::session::Session;
use rustc::session::config::{self, Input};
use rustc_driver::{driver, CompilerCalls, Compilation, RustcDefaultCalls, run_compiler};
use rustc::metadata::creader::CrateReader;
use rustc_resolve as resolve;
use rustc::middle::lang_items;
use rustc::middle::ty;
use syntax;
use syntax::{ast, attr, diagnostics, visit};
use syntax::parse::token;
use syntax::ast::NodeId;
use syntax::ast::Item_::ItemImpl;
use syntax::ext::build::AstBuilder;
use syntax::ext::mtwt;
use syntax::codemap::DUMMY_SP;
use std::collections::HashMap;
use std::env;
use std::io::prelude::*;
use std::io::stderr;
use std::path::PathBuf;
use std::path::Path;
use std::fs::File;
use std::thread;

use strings::src_rope::Rope;
use loader::ReplaceLoader;

#[derive(Debug, PartialEq)]
pub enum Response {
    Error,
    Conflict,
}

pub fn rename_variable(input_file: &str,
                       input: &str,
                       analysis: &str,
                       new_name: &str,
                       rename_var: &str)
                       -> Result<HashMap<String, String>, Response> {
    let analyzed_data = init(analysis);

    //for (key, value) in analyzed_data.type_map.iter() {
    //    println!("{}: \"{}\"", *key, value.get("id").unwrap());
    //}

    //for (key, value) in analyzed_data.type_ref_map.iter() {
    //  println!("{}: \"{:?}\"", *key, value);
    //}

    let dec_map = analyzed_data.var_map;
    let ref_map = analyzed_data.var_ref_map;

    let mut output = HashMap::new();

    let input_file_str = String::from_str(input_file);
    let input_str = String::from_str(input);
    let mut filename = String::from_str(input_file);
    if let Some(decl) = dec_map.get(rename_var) {
        if let Some(file) = decl.get("file_name") {
            filename = file.clone();
        }
    }
    let filename = filename;

    output.insert(input_file_str.clone(), input_str.clone());
    // Check if renaming will cause conflicts
    let node: NodeId = rename_var.parse().unwrap();

    match run_compiler_resolution(String::from_str(input_file), None, //Some(filename, String::from_str(input)),
                                  RefactorType::Variable, String::from_str(new_name),
                                  node, false) {
        Ok(()) => {
            debug!("GOOD");
        },
        Err(x) => { debug!("BAD"); return Err(Response::Conflict); }
    }
    match dec_map.get(rename_var) {
        Some(x) => {
            for (key, value) in dec_map.iter() {
                let name = value.get("name").unwrap();
                if (x.get("scopeid") == value.get("scopeid") &&
                    name == &new_name) {
                    // Conflict present:
                    // May still be ok if there is no references to it
                    // However, standalone blocks won't be detected + macros
                    // Will also restrict if reference is on same line as renaming

                    if let Some(references) = ref_map.get(rename_var) {
                        for map in references.iter() {
                            let filename = map.get("file_name").unwrap();

                            let mut file = match File::open(&Path::new(filename)) {
                                Err(why) => panic!("couldn't open file {}", why),
                                    Ok(file) => file,
                            };
                            let mut file_str = String::new();
                            let _ = file.read_to_string(&mut file_str);
                            output.insert(filename.clone(), file_str.clone());
                            let file_str = &file_str[..];
                            let mut ropes: Vec<Rope> = file_str.lines().map(|x| Rope::from_string(String::from_str(x))).collect();
                            let file_col: usize = map.get("file_col").unwrap().parse().unwrap();
                            let file_line: usize = map.get("file_line").unwrap().parse().unwrap();
                            let file_col_end: usize = map.get("file_col_end").unwrap().parse().unwrap();
                            let file_line_end: usize = map.get("file_line_end").unwrap().parse().unwrap();

                            //let _ = writeln!(&mut stderr(), "{} {} {} {}", file_col, file_line, file_col_end, file_line_end);
                            rename(&mut ropes, file_col, file_line, file_col_end, file_line_end, new_name);
                            let mut answer = String::new();
                            let mut count = ropes.len();
                            for rope in &ropes {
                                answer.push_str(&rope.to_string());
                                if count > 1 {
                                    answer.push_str("\n");
                                    count -= 1;
                                }
                            }

                            match run_compiler_resolution(String::from_str(input_file), Some((String::from_str(filename), answer)),
                                                          RefactorType::Variable, String::from_str(new_name),
                                                          node, true) {
                                Ok(()) => {
                                    debug!("Unexpected success!");
                                    // Check for conflicts 
                                    return Err(Response::Conflict); 
                                },
                                Err(x) => { debug!("Expected failure!");}
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

    Ok(rename_dec_and_ref(input, new_name, rename_var, dec_map, ref_map))
}

pub fn rename_type(input_file: &str,
                   input: &str,
                   analysis: &str,
                   new_name: &str,
                   rename_var: &str)
                   -> Result<HashMap<String, String>, Response> {
    let analyzed_data = init(analysis);

    //for (key, value) in analyzed_data.type_map.iter() {
    //  println!("{}: \"{:?}\"", *key, value);
    //}

    let dec_map = analyzed_data.type_map;
    let ref_map = analyzed_data.type_ref_map;
    let node: NodeId = rename_var.parse().unwrap();

    let input_file_str = String::from_str(input_file);
    let input_str = String::from_str(input);
    let mut filename = String::from_str(input_file);
    if let Some(decl) = dec_map.get(rename_var) {
        if let Some(file) = decl.get("file_name") {
            filename = file.clone();
        }
    }
    let filename = filename;
    match run_compiler_resolution(input_file_str, None, RefactorType::Type,
                                  String::from_str(new_name), node, false) {
        Ok(()) => {},
        Err(x) => { debug!("Unexpected failure!"); return Err(Response::Conflict) }
    }

    if let Some(references) = ref_map.get(rename_var) {
        for map in references.iter() {
            let filename = map.get("file_name").unwrap();

            let mut file = match File::open(&Path::new(filename)) {
                Err(why) => panic!("couldn't open file {}", why),
                Ok(file) => file,
            };
            let mut file_str = String::new();
            let _ = file.read_to_string(&mut file_str);
            let file_str = &file_str[..];
            let mut ropes: Vec<Rope> = file_str.lines().map(|x| Rope::from_string(String::from_str(x))).collect();
            let file_col: usize = map.get("file_col").unwrap().parse().unwrap();
            let file_line: usize = map.get("file_line").unwrap().parse().unwrap();
            let file_col_end: usize = map.get("file_col_end").unwrap().parse().unwrap();
            let file_line_end: usize = map.get("file_line_end").unwrap().parse().unwrap();

            //let _ = writeln!(&mut stderr(), "{} {} {} {}", file_col, file_line, file_col_end, file_line_end);
            rename(&mut ropes, file_col, file_line, file_col_end, file_line_end, new_name);
            let mut answer = String::new();
            let mut count = ropes.len();
            for rope in &ropes {
                answer.push_str(&rope.to_string());
                if count > 1 {
                    answer.push_str("\n");
                    count -= 1;
                }
            }

            match run_compiler_resolution(String::from_str(input_file), Some((String::from_str(filename), answer)),
                                          RefactorType::Variable, String::from_str(new_name),
                                          node, true) {
                Ok(()) => {
                    debug!("Unexpected success!");
                    // Check for conflicts
                    return Err(Response::Conflict);
                },
                Err(x) => { debug!("Expected failure!");}
            }
        }
    }

    Ok(rename_dec_and_ref(input, new_name, rename_var, dec_map, ref_map))
}

fn run_compiler_resolution(root: String,
                           file_override: Option<(String, String)>,
                           kind: RefactorType,
                           new_name: String,
                           node: NodeId,
                           full: bool)
                           -> Result<(), i32> {
    let key = "RUST_FOLDER";
    let mut path = String::new();
    let args = match env::var(key) {
        Ok(val) => {
            path.push_str("-L");
            path.push_str(&val[..]);
            vec!["refactor".to_owned(), 
                path,
                root]
        }
        Err(e) => {vec!["refactor".to_owned(), root]},
    };

    thread::catch_panic(move || {
        let mut call_ctxt = RefactorCalls::new(kind, new_name, node, file_override, full);
        run_compiler(&args, &mut call_ctxt);
    }).map_err(|any|
        1
    )
}

pub fn rename_function(input_file: &str,
                       input: &str,
                       analysis: &str,
                       new_name: &str,
                       rename_var: &str)
                       -> Result<HashMap<String, String>, Response> {
    let analyzed_data = init(analysis);
    let analyzed_data = init(analysis);

    // method calls refer to top level trait function in declid

    // rename original function

    // then rename all statically dispatched with refid = id
    // then rename all dynamically dispatched with declid = id
    // then rename all functions with declid = id
    // assuming mutual exclusion, these should all be covered in func_ref_map

    let dec_map = analyzed_data.func_map;
    let ref_map = analyzed_data.func_ref_map;
    let node: NodeId = rename_var.parse().unwrap();

    let input_file_str = String::from_str(input_file);
    let input_str = String::from_str(input);

    let mut filename = String::from_str(input_file);
    if let Some(decl) = dec_map.get(rename_var) {
        if let Some(file) = decl.get("file_name") {
            filename = file.clone();
        }
    }
    let filename = filename;
    match run_compiler_resolution(input_file_str, None, RefactorType::Function,
                                  String::from_str(new_name), node, false) {
        Ok(()) => {},
        Err(x) => { debug!("Unexpected failure!"); return Err(Response::Conflict) }
    }

    if let Some(references) = ref_map.get(rename_var) {
        for map in references.iter() {
            let filename = map.get("file_name").unwrap();

            let mut file = match File::open(&Path::new(filename)) {
                Err(why) => panic!("couldn't open file {}", why),
                Ok(file) => file,
            };
            let mut file_str = String::new();
            let _ = file.read_to_string(&mut file_str);
            let file_str = &file_str[..];

            let mut ropes: Vec<Rope> = file_str.lines().map(|x| Rope::from_string(String::from_str(x))).collect();
            let file_col: usize = map.get("file_col").unwrap().parse().unwrap();
            let file_line: usize = map.get("file_line").unwrap().parse().unwrap();
            let file_col_end: usize = map.get("file_col_end").unwrap().parse().unwrap();
            let file_line_end: usize = map.get("file_line_end").unwrap().parse().unwrap();

            //let _ = writeln!(&mut stderr(), "{} {} {} {}", file_col, file_line, file_col_end, file_line_end);
            rename(&mut ropes, file_col, file_line, file_col_end, file_line_end, new_name);
            let mut answer = String::new();
            let mut count = ropes.len();
            for rope in &ropes {
                answer.push_str(&rope.to_string());
                if count > 1 {
                    answer.push_str("\n");
                    count -= 1;
                }
            }

            match run_compiler_resolution(String::from_str(input_file), Some((String::from_str(filename), answer)),
                                          RefactorType::Variable, String::from_str(new_name),
                                          node, true) {
                Ok(()) => {
                    debug!("Unexpected success!");
                    // Check for conflicts
                    return Err(Response::Conflict);
                },
                Err(x) => { debug!("Expected failure!");}
            }
        }
    }

    Ok(rename_dec_and_ref(input, new_name, rename_var, dec_map, ref_map))
}

struct AnalysisData {
    var_map: HashMap<String, HashMap<String, String>>,
    var_ref_map: HashMap<String, Vec<HashMap<String, String>>>,
    type_map: HashMap<String, HashMap<String, String>>,
    type_ref_map: HashMap<String, Vec<HashMap<String, String>>>,
    func_map: HashMap<String, HashMap<String, String>>,
    func_ref_map: HashMap<String, Vec<HashMap<String, String>>>,
}

fn init(analysis: &str) -> AnalysisData {
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

    return AnalysisData{ var_map: var_map, var_ref_map: var_ref_map, type_map: type_map,
                         type_ref_map: type_ref_map, func_map: func_map, func_ref_map: func_ref_map }
}

fn rename_dec_and_ref(input: &str,
                      new_name: &str,
                      rename_var: &str,
                      dec_map: HashMap<String, HashMap<String, String>>,
                      ref_map: HashMap<String, Vec<HashMap<String, String>>>)
                      -> HashMap<String, String> {
    let mut output = HashMap::new();

    // TODO Failed an attempt to chain the declaration to the other iterator...
    let map = dec_map.get(rename_var).unwrap();
    let file_col: usize = map.get("file_col").unwrap().parse().unwrap();
    let file_line: usize = map.get("file_line").unwrap().parse().unwrap();
    let file_col_end: usize = map.get("file_col_end").unwrap().parse().unwrap();
    let file_line_end: usize = map.get("file_line_end").unwrap().parse().unwrap();
    let filename = map.get("file_name").unwrap();

    let mut new_file = String::new();
    File::open(&filename).expect("Missing file").read_to_string(&mut new_file);

    let mut ropes: Vec<Rope> = new_file.lines().map(|x| Rope::from_string(String::from_str(x))).collect();
    rename(&mut ropes, file_col, file_line, file_col_end, file_line_end, new_name);

    output.insert(filename.clone(), ropes);

    if let Some(references) = ref_map.get(rename_var) {
        for map in references.iter() {
            let file_col: usize = map.get("file_col").unwrap().parse().unwrap();
            let file_line: usize = map.get("file_line").unwrap().parse().unwrap();
            let file_col_end: usize = map.get("file_col_end").unwrap().parse().unwrap();
            let file_line_end: usize = map.get("file_line_end").unwrap().parse().unwrap();
            let filename = map.get("file_name").unwrap();
            if let Some(ref mut ropes) = output.get_mut(filename) {
                rename(ropes, file_col, file_line, file_col_end, file_line_end, new_name);
                continue;
            }
            let mut new_file = String::new();
            File::open(&filename).expect("Missing file").read_to_string(&mut new_file);
            let mut ropes: Vec<Rope> = new_file.lines().map(|x| Rope::from_string(String::from_str(x))).collect();
            rename(&mut ropes, file_col, file_line, file_col_end, file_line_end, new_name);
            output.insert(filename.clone(), ropes);


            // let _ = writeln!(&mut stderr(), "{} {} {} {}", file_col, file_line, file_col_end, file_line_end);
        }
    }

    let mut outmap = HashMap::new();
    for (key, ropes) in output.iter() {
        let mut answer = String::new();
        let mut count = ropes.len();
        for rope in ropes {
            answer.push_str(&rope.to_string());
            if count > 1 {
                answer.push_str("\n");
                count -= 1;
            }
        }

        outmap.insert(key.clone(), answer);
    }

    outmap
}

fn rename(ropes: &mut Vec<Rope>,
          file_col: usize,
          file_line: usize,
          file_col_end: usize,
          file_line_end: usize,
          new_name: &str) {
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

// TODO more efficient, perhaps better indexed and given type of node as arg
// Factor out the init.
pub fn identify_id(input_filename: &str,
                   analysis: &str,
                   rename_var: &str,
                   row: i32,
                   col: i32)
                   -> String {
    let analyzed_data = init(analysis);

    let _ = writeln!(&mut stderr(), "{} {} {}", rename_var, row, col);
    for (key, value) in analyzed_data.var_map {
        if check_match(rename_var, input_filename, row, col, value) {
            return key;
        }
    }

    for (key, value) in analyzed_data.type_map {
        if check_match(rename_var, input_filename, row, col, value) {
            return key;
        }
    }

    for (key, value) in analyzed_data.func_map {
        if check_match(rename_var, input_filename, row, col, value) {
            return key;
        }
    }

    "".to_string()
}

fn check_match(name: &str,
               input_filename: &str,
               row: i32,
               col: i32,
               record: HashMap<String, String>)
               -> bool {

    let c: i32 = record.get("file_col").unwrap().parse().unwrap();
    let r: i32 = record.get("file_line").unwrap().parse().unwrap();
    let r_end: i32 = record.get("file_line_end").unwrap().parse().unwrap();
    let c_end: i32 = record.get("file_col_end").unwrap().parse().unwrap();
    let filename = record.get("file_name").unwrap();
    let n = record.get("name").unwrap();

    if &name == n { //&& filename == &input_filename {
        if !(row < 0) {
            if row >= r && row <= r_end {
                if !(col < 0) {
                    if col >= c && col < c_end {
                        return true;
                    }
                } else {
                    return true;
                }
            }
        } else {
            return true;
        }
    }

    false
}

#[derive(Copy, Clone)]
pub enum RefactorType {
    Variable,
    Function,
    Type,
    Nil,
}

struct RefactorCalls {
    default_calls: RustcDefaultCalls,
    rType: RefactorType,
    new_name: String,
    node_to_find: NodeId,
    input: Option<(String, String)>,
    isFull: bool,
}

impl RefactorCalls {
    fn new(t: RefactorType,
           new_name: String,
           node: NodeId,
           new_file: Option<(String, String)>,
           isFull: bool)
           -> RefactorCalls {
        RefactorCalls {
            default_calls: RustcDefaultCalls,
            rType: t,
            new_name: new_name,
            node_to_find: node,
            input: new_file,
            isFull: isFull,
        }
    }
}

impl<'a> CompilerCalls<'a> for RefactorCalls {
    fn early_callback(&mut self,
                      _: &getopts::Matches,
                      _: &diagnostics::registry::Registry)
                      -> Compilation {
        Compilation::Continue
    }

    fn late_callback(&mut self,
                     m: &getopts::Matches,
                     s: &Session,
                     i: &Input,
                     odir: &Option<PathBuf>,
                     ofile: &Option<PathBuf>)
                     -> Compilation {
        self.default_calls.late_callback(m, s, i, odir, ofile);
        let mut loader = ReplaceLoader::new();
        // If there were multiple renamings per compile run, it would be added here
        match self.input.as_ref() {
            Some(input) => {
                loader.add_file(input.0.clone(), input.1.clone());
            },
            None => ()
        }
        s.codemap().set_file_loader(Box::new(loader));
        Compilation::Continue
    }

    fn some_input(&mut self,
                  input: Input,
                  input_path: Option<PathBuf>)
                  -> (Input, Option<PathBuf>) {
        (input, input_path)
    }

    fn no_input(&mut self,
                m: &getopts::Matches,
                o: &config::Options,
                odir: &Option<PathBuf>,
                ofile: &Option<PathBuf>,
                r: &diagnostics::registry::Registry)
                -> Option<(Input, Option<PathBuf>)> {
        self.default_calls.no_input(m, o, odir, ofile, r);
        // This is not optimal error handling.
        panic!("No input supplied");
    }

    fn build_controller(&mut self, _: &Session) -> driver::CompileController<'a> {
        let rType = self.rType;
        let isFull = self.isFull;

        let mut control = driver::CompileController::basic();
        if isFull {
            control.after_analysis.stop = Compilation::Stop;
            return control;
        }

        let node_to_find = self.node_to_find;
        let new_name = self.new_name.clone();

        control.after_write_deps.stop = Compilation::Stop;
        control.after_write_deps.callback = box move |state| {
            let krate =  state.expanded_crate.unwrap().clone();
            let ast_map = state.ast_map.unwrap();
            let krate = ast_map.krate();
            CrateReader::new(&state.session).read_crates(krate);
            let lang_items = lang_items::collect_language_items(krate, &state.session);
            let resolve::CrateMap {
                def_map,
                freevars,
                export_map,
                trait_map,
                external_exports,
                glob_map,
            } = resolve::resolve_crate(&state.session, &ast_map, resolve::MakeGlobMap::No);
            debug!("{:?}", def_map);

            // According to some condition !
            //let ps = syntax::parse::ParseSess::new();
            let ps = &state.session.parse_sess;
            let cratename = match attr::find_crate_name(&krate.attrs[..]) {
                Some(name) => name.to_string(),
                None => String::from_str("unknown_crate"),
            };

            debug!("{:?}", token::str_to_ident(&new_name[..]));
            let mut cx = syntax::ext::base::ExtCtxt::new(ps, krate.config.clone(), //vec![],
                                                         syntax::ext::expand::ExpansionConfig::default(cratename));
            debug!("{:?}", token::str_to_ident(&new_name[..]));
            //let ast_node = ast_map.get(ast_map.get_parent(node_to_find));
            //println!("{:?}", ast_node);
            let ast_node = ast_map.get(node_to_find);
            debug!("{:?}", ast_node);
            debug!("{}", node_to_find);
            debug!("{:?}", token::str_to_ident(&new_name[..]));

            // find current path and syntax context
            let mut syntax_ctx = 0;
            match ast_node {
                NodeLocal(pat) => {
                    match pat.node {
                        ast::PatIdent(_, path, _) => {
                            syntax_ctx = path.node.ctxt;
                        },
                        _ => {}
                    }
                },

                _ => {}
            }

            let path = cx.path(DUMMY_SP, vec![token::str_to_ident(&new_name)]);
            // create resolver
            let mut resolver = resolve::create_resolver(&state.session, &ast_map, krate, resolve::MakeGlobMap::No,
                                                        Some(Box::new(|_,_| { true })));
            debug!("{:?}", token::str_to_ident(&new_name[..]));

            match rType {
                RefactorType::Type => {
                    let mut h = HashMap::new();
                    h.insert(String::new(), String::new());
                    debug!("{:?}", token::str_to_ident(&new_name[..]));
                    
                    let mut idens = ast_map.with_path(node_to_find, |path| {
                    let itr = token::get_ident_interner();

                    path.fold(Vec::new(), |mut s, e| {
                        let e = itr.get(e.name());
                        s.push(token::str_to_ident(&e[..]));
                        s
                    })
                    //ast_map::path_to_string(path)

                    });

                    let mut resolver = resolve::create_resolver(&state.session, &ast_map, krate, resolve::MakeGlobMap::No,
                    Some(Box::new(move |node: ast_map::Node, resolved: &mut bool| {
                        if *resolved {
                            return true;
                        }
                        //debug!("Entered resolver callback");
                        match node {
                            NodeLocal(pat) => {
                                if pat.id == node_to_find {
                                    debug!("Found node");
                                    *resolved = true;
                                    return true;
                                }
                            },
                            NodeItem(item) => {
                                println!("{:?}", item);
                                if item.id == node_to_find {
                                    debug!("Found node");
                                    *resolved = true;
                                    return true;
                                }
                            },
                            _ => {}
                        }

                        false
                    })));

                    visit::walk_crate(&mut resolver, krate);

                    let new_iden = token::str_to_ident(&new_name[..]);
                    idens.pop();
                    idens.push(new_iden);

                    token::str_to_ident(&new_name[..]);
                    let path = cx.path(DUMMY_SP, idens);

                    // resolver resolve node id
                    println!("{:?}", path);
                    if resolver.resolve_path(node_to_find, &path, 0, resolve::Namespace::TypeNS, true).is_some() {
                        // unwind at this location
                        panic!(h);
                    }
                },
                RefactorType::Variable => {
                    let mut t = token::str_to_ident(&new_name[..]);
                    t.ctxt = syntax_ctx;
                    debug!("{:?}", mtwt::resolve(t));
                    let path = cx.path(DUMMY_SP, vec![t]);
                    let mut resolver = resolve::create_resolver(&state.session, &ast_map, krate, resolve::MakeGlobMap::No,
                    Some(Box::new(move |node: ast_map::Node, resolved: &mut bool| {
                        if *resolved {
                            return true;
                        }
                        //debug!("Entered resolver callback");
                        //println!("{:?}", node);
                        match node {
                            NodeLocal(pat) => {
                                if pat.id == node_to_find {
                                    debug!("Found node");
                                    *resolved = true;
                                    return true;
                                }
                            },
                            _ => {}
                        }

                        false
                    })));

                    visit::walk_crate(&mut resolver, krate);

                    let mut h = HashMap::new();
                    h.insert(String::new(), String::new());
                    debug!("{:?}", token::str_to_ident(&new_name[..]));
                    
                    // resolver resolve node id
                    //if resolver.resolve_path(node_to_find, &path) {
                    if resolver.resolve_path(node_to_find, &path, 0, resolve::Namespace::ValueNS, true).is_some() {
                        // unwind at this location
                        panic!(h);
                    }
                    //println!("{:?}", mtwt::resolve( token::str_to_ident(&new_name[..])));

                },
                RefactorType::Function => {
                    let mut idens = ast_map.with_path(node_to_find, |path| {
                    let itr = token::get_ident_interner();

                    path.fold(Vec::new(), |mut s, e| {
                        let e = itr.get(e.name());
                        s.push(token::str_to_ident(&e[..]));
                        s
                    })
                    //ast_map::path_to_string(path)

                    });

                    let new_iden = token::str_to_ident(&new_name[..]);
                    idens.pop();
                    idens.push(new_iden);

                    let mut resolver = resolve::create_resolver(&state.session, &ast_map, krate, resolve::MakeGlobMap::No,
                    Some(Box::new(move |node: ast_map::Node, resolved: &mut bool| {
                        if *resolved {
                            return true;
                        }
                        //debug!("Entered resolver callback");
                        match node {
                            NodeLocal(pat) => {
                                if pat.id == node_to_find {
                                    debug!("Found node");
                                    *resolved = true;
                                    return true;
                                }
                            },
                            NodeItem(item) => {
                                match item.node {
                                    ItemImpl(_, _, _, _, _, ref impls) => {
                                        for i in impls.iter() {
                                            if i.id == node_to_find {
                                                debug!("{:?}", i);
                                                debug!("Found node");
                                                *resolved = true;
                                                return true;
                                            }
                                        }
                                    },
                                    _ => {}

                                }
                                if item.id == node_to_find {
                                    debug!("Found node");
                                    *resolved = true;
                                    return true;
                                }
                            },
                            _ => {}
                        }

                        false
                    })));

                    visit::walk_crate(&mut resolver, krate);

                    let mut h = HashMap::new();
                    h.insert(String::new(), String::new());
                    debug!("{:?}", token::str_to_ident(&new_name[..]));
                    
                    // TODO 
                    // let path = cx.path(DUMMY_SP, idens);
                    // resolver resolve node id
                    //if resolver.resolve_path(node_to_find, &path) {
                    if resolver.resolve_path(node_to_find, &path, 0, resolve::Namespace::ValueNS, true).is_some() {
                        // unwind at this location
                        debug!("BAD");
                        panic!(h);
                    }

                    if resolver.resolve_path(node_to_find, &path, 0, resolve::Namespace::TypeNS, true).is_some() {
                        // unwind at this location
                        debug!("BAD");
                        panic!(h);
                    }
                    debug!("OK");
                    //println!("{:?}", mtwt::resolve( token::str_to_ident(&new_name[..])));
                },
                _ => {}              
            }
        };

        control
    }
}

// Compare output to input.
fn handle_result(result: HashMap<String, String>) {
    let mut failures = HashMap::new();

    for (file_name, fmt_text) in result {
        failures.insert(String::new(), String::new());
    }
    if !failures.is_empty() {
        panic!(failures);
    }
}
