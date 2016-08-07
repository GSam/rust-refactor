use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use strings::src_rope::Rope;

// Convenience function for replacing the declaration and all references.
pub fn rename_dec_and_ref(new_name: &str,
                      rename_var: &str,
                      dec_map: &HashMap<String, HashMap<String, String>>,
                      ref_map: &HashMap<String, Vec<HashMap<String, String>>>)
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
    File::open(&filename).expect("Missing file").read_to_string(&mut new_file).unwrap();

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
            File::open(&filename).expect("Missing file").read_to_string(&mut new_file).unwrap();
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

pub fn rename(ropes: &mut Vec<Rope>,
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
