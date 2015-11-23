use std::collections::HashMap;
use std::io::{self, Write};
use analysis::AnalysisData;

// Find the id associated with the row and column given
// (or a -1 wildcard is given and any with a matching name will be returned).
// TODO more efficient, perhaps better indexed and given type of node as arg
pub fn identify_id(input_filename: &str,
                   analyzed_data: &AnalysisData,
                   rename_var: &str,
                   row: i32,
                   col: i32,
                   file: Option<&str>)
                   -> String {
    let _ = writeln!(&mut io::stderr(), "{} {} {}", rename_var, row, col);
    for (key, value) in &analyzed_data.var_map {
        if check_match(rename_var, input_filename, row, col, value, file) {
            return key.clone();
        }
    }

    for (key, value) in &analyzed_data.type_map {
        if check_match(rename_var, input_filename, row, col, value, file) {
            return key.clone();
        }
    }

    for (key, value) in &analyzed_data.func_map {
        if check_match(rename_var, input_filename, row, col, value, file) {
            return key.clone();
        }
    }

    "".to_string()
}

fn check_match(name: &str,
               _input_filename: &str,
               row: i32,
               col: i32,
               record: &HashMap<String, String>,
               file: Option<&str>)
               -> bool {

    let c: i32 = record.get("file_col").unwrap().parse().unwrap();
    let r: i32 = record.get("file_line").unwrap().parse().unwrap();
    let r_end: i32 = record.get("file_line_end").unwrap().parse().unwrap();
    let c_end: i32 = record.get("file_col_end").unwrap().parse().unwrap();
    let filename = record.get("file_name").unwrap();
    let n = record.get("name").unwrap();

    if &name == n && (!file.is_some() || filename == file.unwrap()) {
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
