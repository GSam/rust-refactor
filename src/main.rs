#![feature(rustc_private)]

#[macro_use]
extern crate log;
extern crate refactor;

use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::collections::HashMap;

use refactor::analysis::AnalysisData;

// Simple command line interface to the library.
// Should probably use the actual getopt to be more practical.
fn main() {
    let args: Vec<_> = env::args().collect();

    if args.len() < 5 {
        let _ = writeln!(&mut std::io::stderr(), "Not enough args: <var|type|fn|inline|reify|elide> <analysis> <src> <var> [<outvar>]");
        let _ = writeln!(&mut std::io::stderr(), "var: <nodeid> | <name>:<row or -1>:<col or -1>");
        return;
    }

    if args.len() < 6 && &*args[1] != "reify" && &*args[1] != "elide" && &*args[1] != "inline" {
        let _ = writeln!(&mut std::io::stderr(), "Not enough args: <var|type|fn> <analysis> <src> <var> <outvar>");
        let _ = writeln!(&mut std::io::stderr(), "var: <nodeid> | <name>:<row or -1>:<col or -1>");
        return;
    }

    let path = Path::new(&args[3]);
    let input_id;
    let mut rename_var = &args[4];

    let mut analysis = match File::open(&args[2]) {
        Err(why) => panic!("couldn't open file {}", why),
        Ok(file) => file,
    };
    let mut analysis_str = String::new();
    let _ = analysis.read_to_string(&mut analysis_str);
    let analysis_data = AnalysisData::new(&analysis_str);


    let v: Vec<_> = args[4].split(":").collect();
    if v.len() == 3 {
        input_id = refactor::identify_id(path.file_name().unwrap().to_str().unwrap(), &analysis_data,
                                            v[0], v[1].parse().unwrap(), 
                                            v[2].parse().unwrap(), None);
        let _ = writeln!(&mut std::io::stderr(), "NODE ID: {}", input_id);
        rename_var = &input_id;
    }

    match &*args[1] {
        "var" => {
            let result = refactor::rename_variable(&args[3], &analysis_data, &args[5], rename_var);
            match result {
                Ok(x) => println!("{}", better_string(x)),
                Err(x) => println!("{:?}", x)
            }
        },
        "type" => {
            let result = refactor::rename_type(&args[3], &analysis_data, &args[5], rename_var);
            match result {
                Ok(x) => println!("{}", better_string(x)),
                Err(x) => println!("{:?}", x)
            }
        },
        "fn" => {
            let result = refactor::rename_function(&args[3], &analysis_data, &args[5], rename_var);
            match result {
                Ok(x) => println!("{}", better_string(x)),
                Err(x) => println!("{:?}", x)
            }
        },
        "inline" => {
            let result = refactor::inline_local(&args[3], &analysis_data, rename_var);
            match result {
                Ok(x) => println!("{}", better_string(x)),
                Err(x) => println!("{:?}", x)
            }
        },
        "reify" => {
            let result = refactor::restore_fn_lifetime(&args[3], &analysis_data, rename_var);
            match result {
                Ok(x) => println!("{}", better_string(x)),
                Err(x) => println!("{:?}", x)
            }
        },
        "elide" => {
            let result = refactor::elide_fn_lifetime(&args[3], &analysis_data, rename_var);
            match result {
                Ok(x) => println!("{}", better_string(x)),
                Err(x) => println!("{:?}", x)
            }
        },
        _ => {
            let _ = writeln!(&mut std::io::stderr(), "Unknown refactoring function.");
        }
    }
}

fn better_string(input: HashMap<String, String>) -> String {
    let mut ans = String::new();
    for (key, value) in input.iter() {
        ans.push_str(key);
        ans.push_str("\n");
        ans.push_str(value);
        ans.push_str("\n\n");
    }
    ans
}
