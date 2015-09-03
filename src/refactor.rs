
extern crate csv;

use getopts;
use rustc_front::fold::Folder;
use rustc::front::map as ast_map;
use rustc::front::map::Node;
use rustc::front::map::Node::*;
use rustc::session::{self, Session};
use rustc::session::config::{self, Input};
use rustc_driver::{CompilerCalls, Compilation, diagnostics_registry, driver,
                   handle_options, monitor, RustcDefaultCalls};
use rustc::metadata::creader::LocalCrateReader;
use rustc_resolve as resolve;
use rustc::lint;
use rustc_lint;
use rustc::middle::def_id::DefId;
use rustc::middle::lang_items;
use rustc::middle::infer::region_inference::SameRegions;
use rustc::middle::ty::BoundRegion::*;
use syntax::{self, diagnostic, diagnostics};
use rustc_front::{attr, visit};
use rustc_front::hir as ast;
use rustc_front::hir::ExplicitSelf;
use rustc_front::hir::Item_::{ItemImpl, ItemStruct};
use syntax::ast::{Name, NodeId, Ident};
use syntax::codemap::{self, DUMMY_SP, FileLoader, Pos, Span, Spanned};
use syntax::ext::build::AstBuilder;
use syntax::ext::mtwt;
use syntax::owned_slice::OwnedSlice;
use syntax::parse::token;
use rustc_front::print::pprust::{self, State};
use syntax::print::pp::eof;
use syntax::ptr::P;
use std::collections::{HashMap, HashSet};
use std::env;
use std::io;
use std::io::prelude::*;
use std::io::stderr;
use std::iter::FromIterator;
use std::path::PathBuf;
use std::path::Path;
use std::fs::File;
use std::thread;

use strings::src_rope::Rope;

use folder::{InlineFolder, LifetimeFolder};
use loader::ReplaceLoader;
use rebuilder::{Rebuilder, LifeGiver};
use lifetime_walker::LifetimeWalker;

#[derive(Debug, PartialEq)]
pub enum Response {
    Error,
    Conflict,
    Possible,
}

type ErrorCode = i32;

pub fn rename_variable(input_file: &str,
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

    let input_file_str = String::from(input_file);
    let mut filename = String::from(input_file);
    if let Some(decl) = dec_map.get(rename_var) {
        if let Some(file) = decl.get("file_name") {
            filename = file.clone();
        }
    }
    let filename = filename;

    // Check if renaming will cause conflicts
    let node: NodeId = rename_var.parse().unwrap();

    match run_compiler_resolution(String::from(input_file), None, //Some(filename, String::from(input)),
                                  None, RefactorType::Variable, String::from(new_name),
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
                            let file_str = &file_str[..];
                            let mut ropes: Vec<Rope> = file_str.lines().map(|x| Rope::from_string(String::from(x))).collect();
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

                            match run_compiler_resolution(String::from(input_file), Some(vec![(String::from(filename.clone()), answer)]),
                                                          None, RefactorType::Variable, String::from(new_name),
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

    let output = rename_dec_and_ref(new_name, rename_var, dec_map, ref_map);

    try!(check_reduced_graph(String::from(input_file),
                             output.iter().map(|(x,y)|
                             (x.clone(), y.clone())).collect(),
                             String::from(new_name), node));

    Ok(output)
}

pub fn rename_type(input_file: &str,
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

    let input_file_str = String::from(input_file);
    let mut filename = String::from(input_file);
    if let Some(decl) = dec_map.get(rename_var) {
        if let Some(file) = decl.get("file_name") {
            filename = file.clone();
        }
    }
    let filename = filename;
    match run_compiler_resolution(input_file_str, None, None, RefactorType::Type,
                                  String::from(new_name), node, false) {
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
            let mut ropes: Vec<Rope> = file_str.lines().map(|x| Rope::from_string(String::from(x))).collect();
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

            match run_compiler_resolution(String::from(input_file), Some(vec![(String::from(filename.clone()), answer)]),
                                          None, RefactorType::Variable, String::from(new_name),
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

    Ok(rename_dec_and_ref(new_name, rename_var, dec_map, ref_map))
}

pub fn rename_function(input_file: &str,
                       analysis: &str,
                       new_name: &str,
                       rename_var: &str)
                       -> Result<HashMap<String, String>, Response> {
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

    let input_file_str = String::from(input_file);

    let mut filename = String::from(input_file);
    if let Some(decl) = dec_map.get(rename_var) {
        if let Some(file) = decl.get("file_name") {
            filename = file.clone();
        }
    }
    let filename = filename;
    match run_compiler_resolution(input_file_str, None, None, RefactorType::Function,
                                  String::from(new_name), node, false) {
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

            let mut ropes: Vec<Rope> = file_str.lines().map(|x| Rope::from_string(String::from(x))).collect();
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

            match run_compiler_resolution(String::from(input_file), Some(vec![(String::from(filename.clone()), answer)]),
                                          None, RefactorType::Variable, String::from(new_name),
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

    let output = rename_dec_and_ref(new_name, rename_var, dec_map, ref_map);

    println!("{:?}", output);
    try!(check_reduced_graph(String::from(input_file),
                             output.iter().map(|(x,y)|
                             (x.clone(), y.clone())).collect(),
                             String::from(new_name), node));

    Ok(output)
}

fn lifetimes_in_scope(map: &ast_map::Map,
                      scope_id: NodeId)
                      -> Vec<ast::LifetimeDef> {
    let mut taken = Vec::new();
    let method_id_opt = match map.find(scope_id) {
        Some(node) => match node {
            ast_map::NodeItem(item) => match item.node {
                ast::ItemFn(_, _, _, _, ref gen, _) => {
                    taken.push_all(&gen.lifetimes);
                    None
                },
                _ => None
            },
            ast_map::NodeImplItem(ii) => {
                match ii.node {
                    ast::MethodImplItem(ref sig, _) => {
                        taken.push_all(&sig.generics.lifetimes);
                        Some(ii.id)
                    }
                    //ast::MacImplItem(_) => tcx.sess.bug("unexpanded macro"),
                    _ => None,
                }
            }
            _ => None
        },
        None => None
    };
    if method_id_opt.is_some() {
        let method_id = method_id_opt.unwrap();
        let parent = map.get_parent(method_id);
        match map.find(parent) {
            Some(node) => match node {
                ast_map::NodeItem(item) => match item.node {
                    ast::ItemImpl(_, _, ref gen, _, _, _) => {
                        taken.push_all(&gen.lifetimes);
                    }
                    _ => ()
                },
                _ => ()
            },
            None => ()
        }
    }
    return taken;
}

pub fn restore_fn_lifetime(input_file: &str,
                           analysis: &str,
                           rename_var: &str)
                           -> Result<HashMap<String, String>, Response> {
    let analyzed_data = init(analysis);

    let dec_map = analyzed_data.func_map;
    let ref_map = analyzed_data.func_ref_map;

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
    let (x,y,z,_) = match run_compiler_resolution(input_file_str, None, Some(filename.clone()), RefactorType::ReifyLifetime,
                                  String::from(rename_var), node, true) {
        Ok(()) => { debug!("Unexpected success!"); return Err(Response::Conflict) },
        Err(x) => { println!("{:?}", x); x }
    };

    let mut new_file = String::new();
    File::open(&filename).expect("Missing file").read_to_string(&mut new_file);
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

pub fn elide_fn_lifetime(input_file: &str,
                         analysis: &str,
                         rename_var: &str)
                         -> Result<HashMap<String, String>, Response> {
    let analyzed_data = init(analysis);

    let dec_map = analyzed_data.func_map;
    let ref_map = analyzed_data.func_ref_map;

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
    let (x,y,z,_) = match run_compiler_resolution(input_file_str, None, Some(filename.clone()), RefactorType::ElideLifetime,
                                  String::from(rename_var), node, true) {
        Ok(()) => { debug!("Unexpected success!"); return Err(Response::Conflict) },
        Err(x) => { println!("{:?}", x); x }
    };

    let mut new_file = String::new();
    File::open(&filename).expect("Missing file").read_to_string(&mut new_file);
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


pub fn inline_local(input_file: &str,
                    analysis: &str,
                    rename_var: &str)
                    -> Result<HashMap<String, String>, Response> {
    let analyzed_data = init(analysis);

    let dec_map = analyzed_data.var_map;
    let ref_map = analyzed_data.var_ref_map;
    let node: NodeId = rename_var.parse().unwrap();

    let input_file_str = String::from(input_file);

    let mut filename = String::from(input_file);
    if let Some(decl) = dec_map.get(rename_var) {
        if let Some(file) = decl.get("file_name") {
            filename = file.clone();
        }
    }
    let filename = filename;
    let (x,y,z,_) = match run_compiler_resolution(input_file_str, None, Some(filename.clone()), RefactorType::InlineLocal,
                                  String::from(rename_var), node, true) {
        Ok(()) => { debug!("Unexpected success!"); return Err(Response::Conflict) },
        Err(x) => { println!("{:?}", x); x }
    };

    let mut new_file = String::new();
    File::open(&filename).expect("Missing file").read_to_string(&mut new_file);
    let mut rope = Rope::from_string(new_file);

    rope.src_remove(x, y);
    rope.src_insert(x, z);

    let mut output = HashMap::new();
    output.insert(filename, rope.to_string());
    Ok(output)
}

fn check_reduced_graph(root: String,
                       files: Vec<(String, String)>,
                       new_name: String,
                       node: NodeId)
                       -> Result<(), Response> {

    match run_compiler_resolution(root, Some(files), None, RefactorType::Reduced, new_name, node, false) {
        Ok(()) => Ok(()),
        Err(x) => Err(Response::Conflict)

    }
}

fn run_compiler_resolution(root: String,
                           file_override: Option<Vec<(String, String)>>,
                           working_file: Option<String>,
                           kind: RefactorType,
                           new_name: String,
                           node: NodeId,
                           full: bool)
                           -> Result<(), (usize, usize, String, ErrorCode)> {
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

    let mut loader = ReplaceLoader::new();
    match file_override.as_ref() {
        Some(input) => {
            for file in input.iter() {
                loader.add_file(file.0.clone(), file.1.clone());
            }
        },
        None => ()
    }

    thread::catch_panic(move || {
        let mut call_ctxt = RefactorCalls::new(kind, new_name, node, file_override,
                                               working_file, full);
        // Calling monitor fixes a bug where this process is put into an
        // invalid (logging) state.
        match kind {
            RefactorType::InlineLocal |
            RefactorType::ReifyLifetime |
            RefactorType::ElideLifetime => run_compiler(&args, &mut call_ctxt, Box::new(loader)),
            _ => monitor(move || run_compiler(&args, &mut call_ctxt, Box::new(loader)))
        }
    }).map_err(|any|
        *any.downcast().ok().unwrap_or_default()
    )
}

fn run_compiler<'a>(args: &[String], callbacks: &mut CompilerCalls<'a>, loader: Box<FileLoader>) {
    macro_rules! do_or_return {($expr: expr) => {
        match $expr {
            Compilation::Stop => return,
            Compilation::Continue => {}
        }
    }}

    let matches = match handle_options(args.to_vec()) {
        Some(matches) => matches,
        None => return
    };

    let sopts = config::build_session_options(&matches);

    let descriptions = diagnostics_registry();

    do_or_return!(callbacks.early_callback(&matches, &descriptions, sopts.color));

    let (odir, ofile) = make_output(&matches);
    let (input, input_file_path) = match make_input(&matches.free) {
        Some((input, input_file_path)) => callbacks.some_input(input, input_file_path),
        None => match callbacks.no_input(&matches, &sopts, &odir, &ofile, &descriptions) {
            Some((input, input_file_path)) => (input, input_file_path),
            None => return
        }
    };

    let can_print_warnings = sopts.lint_opts
        .iter()
        .filter(|&&(ref key, _)| *key == "warnings")
        .map(|&(_, ref level)| *level != lint::Allow)
        .last()
        .unwrap_or(true);


    let codemap = codemap::CodeMap::with_file_loader(loader);
    let diagnostic_handler =
        diagnostic::Handler::new(sopts.color, Some(descriptions), can_print_warnings);
    let span_diagnostic_handler =
        diagnostic::SpanHandler::new(diagnostic_handler, codemap);

    let mut sess = session::build_session_(sopts, input_file_path, span_diagnostic_handler);
    rustc_lint::register_builtins(&mut sess.lint_store.borrow_mut(), Some(&sess));
    if sess.unstable_options() {
        sess.opts.show_span = matches.opt_str("show-span");
    }
    let cfg = config::build_configuration(&sess);

    do_or_return!(callbacks.late_callback(&matches, &sess, &input, &odir, &ofile));

    let plugins = sess.opts.debugging_opts.extra_plugins.clone();
    let control = callbacks.build_controller(&sess);
    driver::compile_input(sess, cfg, &input, &odir, &ofile, Some(plugins), control);
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

fn rename_dec_and_ref(new_name: &str,
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

    let mut ropes: Vec<Rope> = new_file.lines().map(|x| Rope::from_string(String::from(x))).collect();
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
            let mut ropes: Vec<Rope> = new_file.lines().map(|x| Rope::from_string(String::from(x))).collect();
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

#[derive(Copy, Clone, PartialEq)]
pub enum RefactorType {
    Variable,
    Function,
    Type,
    InlineLocal,
    Reduced,
    ReifyLifetime,
    ElideLifetime,
    Nil,
}

struct RefactorCalls {
    default_calls: RustcDefaultCalls,
    r_type: RefactorType,
    new_name: String,
    node_to_find: NodeId,
    input: Option<Vec<(String, String)>>,
    working_file: Option<String>,
    is_full: bool,
}

impl RefactorCalls {
    fn new(t: RefactorType,
           new_name: String,
           node: NodeId,
           new_file: Option<Vec<(String, String)>>,
           working_file: Option<String>,
           is_full: bool)
           -> RefactorCalls {
        RefactorCalls {
            default_calls: RustcDefaultCalls,
            r_type: t,
            new_name: new_name,
            node_to_find: node,
            input: new_file,
            working_file: working_file,
            is_full: is_full,
        }
    }
}

impl<'a> CompilerCalls<'a> for RefactorCalls {
    fn late_callback(&mut self,
                     m: &getopts::Matches,
                     s: &Session,
                     i: &Input,
                     odir: &Option<PathBuf>,
                     ofile: &Option<PathBuf>)
                     -> Compilation {
        self.default_calls.late_callback(m, s, i, odir, ofile);
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
        let r_type = self.r_type;
        let is_full = self.is_full;
        let node_to_find = self.node_to_find;
        let input = self.working_file.clone().unwrap_or_default();

        let mut control = driver::CompileController::basic();
        if is_full {
            control.after_analysis.stop = Compilation::Stop;
            control.after_analysis.callback = box move |state| {
                let krate = state.krate.unwrap().clone();
                let tcx = state.tcx.unwrap();
                let anal = state.analysis.unwrap();
                let ast_map = &tcx.map;
                if r_type == RefactorType::InlineLocal {

                    debug!("{:?}", ast_map.get(ast_map.get_parent(node_to_find)));
                    debug!("{:?}", ast_map.get(node_to_find));

                    match ast_map.get(node_to_find) {
                        NodeLocal(ref pat) => {
                        },
                        _ => { panic!(); }
                    }

                    let mut parent = None;
                    let mut other = None;
                    match ast_map.get(ast_map.get_parent(node_to_find)) {
                        NodeItem(ref item) => {
                            parent = Some(P((*item).clone()));
                            other = Some((*item).clone());
                            debug!("{:?}", pprust::item_to_string(item));
                        },
                        _ => {}
                    }

                    let src;
                    src = state.session.codemap().get_filemap(&input[..])
                                                 .src
                                                 .as_ref()
                                                 .unwrap()
                                                 .as_bytes()
                                                 .to_vec();
                    // need to distinguish internal errors

                    let mut rdr = &src[..];
                    let mut out = Vec::new();
                    let ann = pprust::NoAnn;
                    {
                        let out_borrow: &mut Write = &mut out;
                        let mut pp_state = State::new_from_input(state.session.codemap(), state.session.diagnostic(), input.clone(), &mut rdr, box out_borrow, &ann, true);

                        if let Some(other) = other {
                            let v = pp_state.print_item(&other);
                            //debug!("{:?}", v);
                            //pp_state.print_mod(&krate.module, &krate.attrs);
                        }
                        eof(&mut pp_state.s);
                    }
                    let _ = out.flush();
                    debug!("{:?}", out);
                    /*{
                        let mut out_borrow: &mut Write = &mut out;
                        let ann = pprust::NoAnn;
                        pprust::print_crate(state.session.codemap(), state.session.diagnostic(), &krate, input.clone(), &mut rdr, box out_borrow, &ann, true);
                    }
                    debug!("{:?}", out);*/

                    // Build save walker
                    let src2;
                    src2 = state.session.codemap().get_filemap(&input[..])
                                                 .src
                                                 .as_ref()
                                                 .unwrap()
                                                 .as_bytes()
                                                 .to_vec();
                    let mut rdr2 = &src2[..];

                    if let Some(par) = parent {
                        let outer_span = par.span;
                        //let mut visitor = DumpCsvVisitor::new(tcx, anal, output_file);
                        let mut folder = InlineFolder::new(tcx, anal, node_to_find);
                        debug!("{:?}", Vec::from_iter(folder.fold_item(par.clone()).into_iter()));
                        debug!("Number of usages: {}", folder.usages);

                        // First we want to ignore destructuring locals, this has issues with lifetimes + type info.
                        // It should also actually BE a local, not just some variable-like item.
                        // TODO
                        // What about sensible destructors, operator overloading?
                        // BUT if we get a 'consider using a let binding error', then, we cannot inline.
                        if folder.usages <= 1 {
                            // This is generally OK, unless the expression contains an impure function/constructor
                            // e.g. let a = <changes external state>
                            //      <change external state some other way>
                            // Now if we move the first change after the second change, behaviour might change.
                            // If doesn't matter here if we have copy, move, borrow etc.
                            //
                            // Due to uniqueness constraints in Rust, if there is just a single usage, there really
                            // is just a single usage without any aliases.

                            // If any variables composing the initializer were redeclared in the meantime, return
                            if folder.changed_paths {
                                return;
                            }


                        } else {
                            // Otherwise, multiple references:

                            // Mutable case:
                            // If the variable is mutable, inlining is a bad idea!!!
                            // e.g. let mut a = 2;
                            // a = 3; // Now the expression is made the lvalue, but this holds no meaning
                            // Same again with *&mut a modifying the internal value.
                            let used_mutables = tcx.used_mut_nodes.borrow();
                            // CAVEAT:
                            // If the mutable was never used, then it should be considered mutable.
                            if folder.mutable && used_mutables.contains(&node_to_find) {
                                debug!("IS MUTABLE");
                                return;
                            }
                            // CAVEAT:
                            // If there is a refcell, or interior mutability, then it really is mutable.
                            let ty_cache = tcx.ast_ty_to_ty_cache.borrow();
                            let interior_unsafe = 0b0000_0000__0000_0000__0010;
                            if let Some(node_ctx) = ty_cache.get(&folder.type_node_id) {
                                debug!("BITS: {:?}", node_ctx.type_contents(tcx).bits);
                                if node_ctx.type_contents(tcx).bits & interior_unsafe != 0 {
                                    debug!("IS MUTABLE (REFCELL)");
                                    return;
                                }
                            }

                            // If the variable is a direct alias, then it might be alright.
                            // In this case, movements or borrows are irrelevant.
                            // e.g. let a = 2;
                            //      let b = a; // it doesn't matter if this is a copy
                            //   or let a = &2;
                            //      let b = a; // this duplicates the reference
                            //   or let a = &mut 2;
                            //      let b = a; // this moves &mut into b
                            //   or let a = vec![0];
                            //      let b = a; // this moves a into b
                            // Whether or not a is inlined, it must follow the normal lifetime rules.
                            // Whatever a refers to must exists for the right scopes.
                            // However, you must check no one redeclares a in the meantime!
                            if let Some(ref to_replace) = folder.to_replace {
                                match (**to_replace).node {
                                    ast::ExprPath(..) => {
                                        // Alias case:
                                    },
                                    _ => {
                                    }
                                }
                            }


                            debug!("IS NOT MUTABLE");
                            // Immutable case:
                            // If the final type implements the copy trait, then this should always be OK!
                            // Either way check which paths compose the initializer and ensure they resolve
                            // to the same item at the new call site.
                            // e.g. b = 2;
                            // let a = b + c
                            // let b = 3;
                            // println!("{}", a);

                            // If any variables composing the initializer were redeclared in the meantime, return
                            if folder.changed_paths {
                                return;
                            }

                            // If any variables composing the initializer mutated in the meantime, return
                            // TODO

                        }

                        let mut out = Vec::new();
                        {
                            let out_borrow: &mut Write = &mut out;
                            let mut pp_state = State::new_from_input(state.session.codemap(), state.session.diagnostic(), input.clone(), &mut rdr2, box out_borrow, &ann, true);

                            pp_state.print_item(&folder.fold_item(par).get(0));
                            //debug!("{:?}", v);
                            //pp_state.print_mod(&krate.module, &krate.attrs);
                            //pp_state.print_remaining_comments();
                            eof(&mut pp_state.s);
                        }
                        let _ = out.flush();
                        debug!("{:?}", out);
                        let hi_pos = state.session.codemap().lookup_byte_offset(outer_span.hi).pos.to_usize();
                        let lo_pos = state.session.codemap().lookup_byte_offset(outer_span.lo).pos.to_usize();
                        panic!((lo_pos, hi_pos, String::from_utf8(out).ok().expect("Pretty printer didn't output UTF-8"), 0));
                        //pprust::item_to_string(folder.fold_item(par).get(0))
                        //visit::walk_crate(&mut visitor, &krate);
                    }
                } else if r_type == RefactorType::ReifyLifetime || r_type == RefactorType::ElideLifetime {
                    debug!("{:?}", ast_map.get(node_to_find));

                    let taken = lifetimes_in_scope(&tcx.map, node_to_find);
                    let life_giver = LifeGiver::with_taken(&taken[..]);
                    let node_inner = match ast_map.find(node_to_find) {
                        Some(ref node) => match *node {
                            ast_map::NodeItem(ref item) => {
                                match item.node {
                                    ast::ItemFn(ref fn_decl, unsafety, constness, _, ref gen, ref body) => {
                                        Some((fn_decl, gen, unsafety, constness,
                                              item.ident, None, item.span, body.span))
                                    },
                                    _ => None
                                }
                            }
                            ast_map::NodeImplItem(item) => {
                                match item.node {
                                    ast::MethodImplItem(ref sig, ref body) => {
                                        Some((&sig.decl,
                                              &sig.generics,
                                              sig.unsafety,
                                              sig.constness,
                                              item.ident,
                                              Some(&sig.explicit_self.node),
                                              item.span, body.span))
                                    }
                                    //ast::MacImplItem(_) => self.tcx.sess.bug("unexpanded macro"),
                                    _ => None,
                                }
                            },
                            ast_map::NodeTraitItem(item) => {
                                match item.node {
                                    ast::MethodTraitItem(ref sig, Some(ref body)) => {
                                        Some((&sig.decl,
                                              &sig.generics,
                                              sig.unsafety,
                                              sig.constness,
                                              item.ident,
                                              Some(&sig.explicit_self.node),
                                              item.span, body.span))
                                    }
                                    _ => None
                                }
                            }
                            _ => None
                        },
                        None => None
                    };
                    let mut a = Vec::new();
                    let (fn_decl, generics, unsafety, constness, ident, expl_self, span, body_span)
                                                = node_inner.expect("expect item fn");

                    let mut folder = LifetimeFolder{ has_bounds: false, expl_self: Name(0) };
                    let elided_fn_decl = folder.fold_fn_decl(fn_decl.clone());
                    let elided_expl_self_tmp;
                    let mut elided_expl_self = None;

                    // Count input lifetimes and count output lifetimes.
                    let mut in_walker = LifetimeWalker::new();
                    let mut out_walker = LifetimeWalker::new();

                    if let Some(expl_self) = expl_self {
                        visit::walk_explicit_self(&mut in_walker, &Spanned {node: expl_self.clone(), span: DUMMY_SP});
                        elided_expl_self_tmp = folder.fold_explicit_self(Spanned {node: expl_self.clone(), span: DUMMY_SP});
                        elided_expl_self = Some(&elided_expl_self_tmp.node);
                    }

                    for argument in fn_decl.inputs.iter() {
                        debug!("FN DECL: {:?}", argument);
                        visit::walk_ty(&mut in_walker, &*argument.ty);
                    }

                    visit::walk_fn_ret_ty(&mut out_walker, &fn_decl.output);

                    if r_type == RefactorType::ElideLifetime {
                        let elided_generics = folder.fold_generics(generics.clone());
                        let mut parameterized = HashSet::new();
                        for lifetimes in generics.lifetimes.iter() {
                            parameterized.insert(lifetimes.lifetime.name);
                        }

                        // Can't elide if returning multiple lifetimes
                        if out_walker.names.len() > 1 {
                            return;
                        }

                        // Don't elide if return doesn't appear in generics (trait lifetime?)
                        let intersect: HashSet<_> = out_walker.names.intersection(&parameterized).cloned().collect();
                        if out_walker.names.len() > 0 && intersect.len() == 0 {
                            return;
                        }

                        // Make sure that each input lifetime is never used more than once
                        if in_walker.names.len() as u32 + in_walker.anon != in_walker.total {
                            return;
                        }

                        // If you have a return, either it has the same name as the only input, or that of self
                        let intersect: HashSet<_> = out_walker.names.intersection(&in_walker.names).cloned().collect();
                        if out_walker.names.len() > 0 && !out_walker.names.contains(&folder.expl_self)
                                           && (in_walker.names.len() > 1 || intersect.len() == 0) {
                            return;
                        }

                        // Make sure that input lifetimes are all parameterized
                        // TODO delete only unparameterized?
                        if !in_walker.names.is_subset(&parameterized) {
                            return;
                        }

                        // TODO move has_bounds out of the folder
                        if folder.has_bounds {
                            return;
                        }

                        let mut answer = pprust::fun_to_string(&elided_fn_decl, unsafety, constness, ident, elided_expl_self, &elided_generics);

                        // Add some likely spacing
                        answer.push_str(" ");

                        let hi_pos = state.session.codemap().lookup_byte_offset(body_span.lo).pos.to_usize();
                        let lo_pos = state.session.codemap().lookup_byte_offset(span.lo).pos.to_usize();
                        panic!((lo_pos, hi_pos, answer, 0));
                    }

                    // Count anonymous and count total.
                    // CASE 1: fn <'a> (x: &'a) -> &out
                    // CASE 2: fn (x: &a) -> &out
                    // If there is exactly 1 input lifetime, then reuse that lifetime for output (possibly multiple).
                    if in_walker.total == 1 {
                        let mut regions = Vec::new();
                        if in_walker.anon == 0 {
                            // CASE 1
                            regions.push(BrNamed(DefId{krate: 0, node: 0}, generics.lifetimes.get(0).unwrap().lifetime.name));
                            for x in 0..out_walker.anon {
                                regions.push(BrAnon(x));
                            }
                        } else {
                            // CASE 2
                            regions.push(BrAnon(0));
                            for x in 0..out_walker.anon {
                                regions.push(BrAnon(x+1));
                            }
                        }
                        a.push(SameRegions{scope_id: 0, regions: regions});

                    } else if in_walker.total > 1 {
                        // If there is more than one lifetime, then:
                        // fn <'a, 'b> (x: &'a, y: &'b, z: &) -> &'a out
                        // We can make concrete the anonymous input lifetimes but not the output.
                        for x in 0..in_walker.anon {
                            a.push(SameRegions{scope_id: 0, regions: vec![BrAnon(x)]});
                        }

                        // Unless, there is a self lifetime, then:
                        // fn <'a, 'b> (self: &'a, ...) -> &out
                        // We can make concrete the output lifetime as well (which may be multiple).
                        if let Some(expl_self) = expl_self {
                            match *expl_self {
                                ast::SelfRegion(ref life, _, _) => {
                                    if life.is_some() {
                                        // self has a named lifetime
                                        let mut regions = Vec::new();
                                        regions.push(BrNamed(DefId{krate: 0, node: 0}, life.unwrap().name));
                                        for x in 0..out_walker.anon {
                                            regions.push(BrAnon(in_walker.anon + x));
                                        }
                                        a.push(SameRegions{scope_id: 0, regions: regions});
                                    } else {
                                        // self is anonymous
                                        // TODO remove expl_self
                                        let mut regions = &mut a.get_mut(in_walker.expl_self as usize).expect("Missing expl self").regions;
                                        for x in 0..out_walker.anon {
                                            regions.push(BrAnon(in_walker.anon + x));
                                        }
                                    }
                                }
                                _ => ()
                            }
                        }

                    }

                    let rebuilder = Rebuilder::new(tcx, fn_decl, expl_self,
                                                   generics, &a/*same_regions*/, &life_giver);
                    let (fn_decl, expl_self, generics) = rebuilder.rebuild();
                    //self.give_expl_lifetime_param(&fn_decl, unsafety, constness, ident,
                    //                              expl_self.as_ref(), &generics, span);
                    debug!("{}", pprust::fun_to_string(&fn_decl, unsafety, constness, ident, expl_self.as_ref(), &generics));
                    println!("{}", pprust::fun_to_string(&fn_decl, unsafety, constness, ident, expl_self.as_ref(), &generics));
                    //debug!("{:?}", tcx.region_maps);
                    debug!("{:?}", tcx.named_region_map);
                    //debug!("{:?}", tcx.free_region_maps.borrow());
                    let mut answer = pprust::fun_to_string(&fn_decl, unsafety, constness, ident, expl_self.as_ref(), &generics);

                    // Add some likely spacing
                    answer.push_str(" ");

                    let hi_pos = state.session.codemap().lookup_byte_offset(body_span.lo).pos.to_usize();
                    let lo_pos = state.session.codemap().lookup_byte_offset(span.lo).pos.to_usize();
                    panic!((lo_pos, hi_pos, answer, 0));
                }
            };

            return control;
        }

        let new_name = self.new_name.clone();

        control.after_write_deps.stop = Compilation::Stop;
        control.after_write_deps.callback = box move |state| {
            //let krate = state.krate.unwrap().clone();
            let ast_map = state.ast_map.unwrap();
            let krate = ast_map.krate();
            LocalCrateReader::new(&state.session, &ast_map).read_crates(krate);
            let _ = lang_items::collect_language_items(krate, &state.session);
            /*let resolve::CrateMap {
                def_map,
                freevars,
                export_map,
                trait_map,
                external_exports,
                glob_map,
            } = resolve::resolve_crate(&state.session, &ast_map, resolve::MakeGlobMap::No);
            debug!("{:?}", def_map);*/

            // According to some condition !
            //let ps = syntax::parse::ParseSess::new();
            //let ps = &state.session.parse_sess;
            let cratename = match attr::find_crate_name(&krate.attrs[..]) {
                Some(name) => name.to_string(),
                None => String::from("unknown_crate"),
            };

            debug!("{:?}", token::str_to_ident(&new_name[..]));
            debug!("{:?}", token::str_to_ident(&new_name[..]));
            //let ast_node = ast_map.get(ast_map.get_parent(node_to_find));
            //println!("{:?}", ast_node);
            debug!("{}", node_to_find);
            let ast_node = ast_map.find(node_to_find);
            debug!("{:?}", ast_node);
            debug!("{:?}", token::str_to_ident(&new_name[..]));

            // find current path and syntax context
            let mut syntax_ctx = 0;
            // If None, then it is probably a field.
            // TODO fields have no super/sub-block conflict
            // Can we remove the compiler runs afterwards?
            if let Some(ast_node) = ast_node {
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
            }

            let path = build_path(DUMMY_SP, vec![token::str_to_ident(&new_name)]);
            // create resolver
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
                            ItemStruct(ref def, _) => {
                                for field in def.fields.iter() {
                                    if field.node.id == node_to_find {
                                        *resolved = true;
                                        return true;
                                    }
                                }
                            },
                            _ => {}

                        }
                        if item.id == node_to_find {
                            debug!("Found node");
                            debug!("{:?}", item);
                            *resolved = true;
                            return true;
                        }
                    },
                    _ => {}
                }

                false
            })));

            match r_type {
                RefactorType::Type => {
                    let mut h = HashMap::new();
                    h.insert(String::new(), String::new());
                    debug!("{:?}", token::str_to_ident(&new_name[..]));
                    
                    let mut idens = ast_map.with_path(node_to_find, |path| {
                    let itr = token::get_ident_interner();

                    path.fold(Vec::new(), |mut s, e| {
                        let e = itr.get(e.name());
                        s.push(token::str_to_ident(&e[..]));
                        s })
                        //ast_map::path_to_string(path)
                    });


                    visit::walk_crate(&mut resolver, krate);

                    let new_iden = token::str_to_ident(&new_name[..]);
                    idens.pop();
                    idens.push(new_iden);

                    token::str_to_ident(&new_name[..]);
                    let path = build_path(DUMMY_SP, idens);

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
                    let path = build_path(DUMMY_SP, vec![t]);

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
                        debug!("BAD ValueNS");
                        panic!(h);
                    }

                    // Is it possible for type namespace to ever conflict with functions?
                    /*if resolver.resolve_path(node_to_find, &path, 0, resolve::Namespace::TypeNS, true).is_some() {
                        // unwind at this location
                        debug!("BAD TypeNS");
                        panic!(h);
                    }*/

                    debug!("OK");
                    //println!("{:?}", mtwt::resolve( token::str_to_ident(&new_name[..])));
                },
                _ => { debug!("HERE"); /* Reduced graph check falls here */ }
            }
        };

        control
    }
}

// Extract output directory and file from matches.
fn make_output(matches: &getopts::Matches) -> (Option<PathBuf>, Option<PathBuf>) {
    let odir = matches.opt_str("out-dir").map(|o| PathBuf::from(&o));
    let ofile = matches.opt_str("o").map(|o| PathBuf::from(&o));
    (odir, ofile)
}

// Extract input (string or file and optional path) from matches.
fn make_input(free_matches: &[String]) -> Option<(Input, Option<PathBuf>)> {
    if free_matches.len() == 1 {
        let ifile = &free_matches[0][..];
        if ifile == "-" {
            let mut src = String::new();
            io::stdin().read_to_string(&mut src).unwrap();
            Some((Input::Str(src), None))
        } else {
            Some((Input::File(PathBuf::from(ifile)), Some(PathBuf::from(ifile))))
        }
    } else {
        None
    }
}

pub fn build_path(span: Span, strs: Vec<Ident> ) -> ast::Path {
    path_all(span, false, strs, Vec::new(), Vec::new(), Vec::new())
}

pub fn build_path_ident(span: Span, id: Ident) -> ast::Path {
    build_path(span, vec!(id))
}

pub fn build_path_global(span: Span, strs: Vec<Ident> ) -> ast::Path {
    path_all(span, true, strs, Vec::new(), Vec::new(), Vec::new())
}

fn path_all(sp: Span,
            global: bool,
            mut idents: Vec<Ident> ,
            lifetimes: Vec<ast::Lifetime>,
            types: Vec<P<ast::Ty>>,
            bindings: Vec<P<ast::TypeBinding>> )
            -> ast::Path {
    let last_identifier = idents.pop().unwrap();
    let mut segments: Vec<ast::PathSegment> = idents.into_iter()
                                                  .map(|ident| {
        ast::PathSegment {
            identifier: ident,
            parameters: ast::PathParameters::none(),
        }
    }).collect();
    segments.push(ast::PathSegment {
        identifier: last_identifier,
        parameters: ast::AngleBracketedParameters(ast::AngleBracketedParameterData {
            lifetimes: lifetimes,
            types: OwnedSlice::from_vec(types),
            bindings: OwnedSlice::from_vec(bindings),
        })
    });
    ast::Path {
        span: sp,
        global: global,
        segments: segments,
    }
}
