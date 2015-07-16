#![feature(io)]
#![feature(box_syntax)]
#![feature(box_patterns)]
#![feature(collections)]
#![feature(rustc_private)]
#![feature(core)]
#![feature(path)]

#[macro_use]
extern crate log;

extern crate refactor;

use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::collections::HashMap;

fn main() {
    let args = env::args();

    if args.len() < 6 {
        let _ = writeln!(&mut std::io::stderr(), "Not enough args: <var|type|fn> <analysis> <src> <var> <outvar>");
        let _ = writeln!(&mut std::io::stderr(), "var: <nodeid> | <name>:<row or -1>:<col or -1>");
        return;
    }

    let args: Vec<_> = args.collect();
    let path = Path::new(&args[3]);
    let mut s;
    let mut rename_var = &args[4];

    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open file {}", why),
        Ok(file) => file,
    };
    let mut file_str = String::new();
    let _ = file.read_to_string(&mut file_str);

    let mut analysis = match File::open(&args[2]) {
        Err(why) => panic!("couldn't open file {}", why),
        Ok(file) => file,
    };
    let mut analysis_str = String::new();
    let _ = analysis.read_to_string(&mut analysis_str);


    let v: Vec<_> = args[4].split(":").collect();
    if v.len() == 3 {
        s = refactor::refactor::identify_id(path.file_name().unwrap().to_str().unwrap(), &analysis_str,
                                            v[0], v[1].parse().unwrap(), 
                                            v[2].parse().unwrap());
        let _ = writeln!(&mut std::io::stderr(), "NODE ID: {}", s);
        rename_var = &s;
    }

    match &*args[1] {
        "var" => {
            let result = refactor::refactor::rename_variable(&args[3], &file_str, &analysis_str, &args[5], rename_var);
            match result {
                Ok(x) => println!("{}", better_string(x)),
                Err(x) => println!("{:?}", x)
            }
        },
        "type" => {
            let result = refactor::refactor::rename_type(&args[3], &file_str, &analysis_str, &args[5], rename_var);
            match result {
                Ok(x) => println!("{}", better_string(x)),
                Err(x) => println!("{:?}", x)
            }
        },
        "fn" => {
            let result = refactor::refactor::rename_function(&args[3], &file_str, &analysis_str, &args[5], rename_var);
            match result {
                Ok(x) => println!("{}", better_string(x)),
                Err(x) => println!("{:?}", x)
            }
        },
        _ => {
            let _ = writeln!(&mut std::io::stderr(), "Unknown rename function.");
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
