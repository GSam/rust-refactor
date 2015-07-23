#![feature(io)]

extern crate refactor;

use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use refactor::refactor::Response;

fn read_to_string(filename: &str) -> String {
    let mut file = match File::open(filename) {
        Err(why) => panic!("couldn't open file {}", why),
        Ok(file) => file,
    };

    let mut output = String::new();
    file.read_to_string(&mut output);

    return output;
}

#[test]
fn working_variable_1() {
    let file = "basic_rename.rs";
    let input = read_to_string(file);
    let output = read_to_string("working_rename_1_out.rs");
    let analysis = read_to_string("basic_rename.csv");

    match refactor::refactor::rename_variable(&"basic_rename.rs", &input, &analysis, "hello", "9") {
        Ok(x) => assert_eq!(output.trim(), x.get(file).unwrap().trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn working_variable_2() {
    let file = "basic_rename.rs";
    let input = read_to_string(file);
    let output = read_to_string("working_rename_2_out.rs");
    let analysis = read_to_string("basic_rename.csv");

    match refactor::refactor::rename_variable(&"basic_rename.rs", &input, &analysis, "hello", "17") {
        Ok(x) => assert_eq!(output.trim(), x.get(file).unwrap().trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn working_variable_3() {
    let file = "alex_var_test.rs";
    let input = read_to_string(file);
    let output = read_to_string("alex_var_test_out.rs");
    let analysis = read_to_string("alex_var_test.csv");

    match refactor::refactor::rename_variable(&"alex_var_test.rs", &input, &analysis, "bar", "14") {
        Ok(x) => assert_eq!(output.trim(), x.get(file).unwrap().trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn working_variable_4() {
    let file = "alex_var_test.rs";
    let input = read_to_string(file);
    let output = read_to_string("alex_var_test_out2.rs");
    let analysis = read_to_string("alex_var_test.csv");

    match refactor::refactor::rename_variable(&"alex_var_test.rs", &input, &analysis, "bar", "4") {
        Ok(x) => assert_eq!(output.trim(), x.get(file).unwrap().trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn working_variable_5() {
    let file = "const_rename.rs";
    let input = read_to_string(file);
    let output = read_to_string("const_rename_out.rs");
    let analysis = read_to_string("const_rename.csv");

    match refactor::refactor::rename_variable(&"const_rename.rs", &input, &analysis, "BAZ", "8") {
        Ok(x) => assert_eq!(output.trim(), x.get(file).unwrap().trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn working_variable_6() {
    let file = "working_fn_local.rs";
    let input = read_to_string(file);
    let output = read_to_string("working_fn_local_out.rs");
    let analysis = read_to_string("working_fn_local.csv");

    match refactor::refactor::rename_variable(&"working_fn_local.rs", &input, &analysis, "Foo", "9") {
        Ok(x) => assert_eq!(output.trim(), x.get(file).unwrap().trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn working_variable_7() {
    let file = "working_nested.rs";
    let input = read_to_string(file);
    let output = read_to_string("working_nested_out.rs");
    let analysis = read_to_string("working_nested.csv");

    match refactor::refactor::rename_variable(&"working_nested.rs", &input, &analysis, "b", "16") {
        Ok(x) => assert_eq!(output.trim(), x.get(file).unwrap().trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn working_variable_8() {
    let file = "working_tuple_let.rs";
    let input = read_to_string(file);
    let output = read_to_string("working_tuple_let_out.rs");
    let analysis = read_to_string("working_tuple_let.csv");

    match refactor::refactor::rename_variable(file, &input, &analysis, "x", "10") {
        Ok(x) => assert_eq!(output.trim(), x.get(file).unwrap().trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn working_variable_9() {
    let file = "working_mut_tuple_let.rs";
    let input = read_to_string(file);
    let output = read_to_string("working_mut_tuple_let_out.rs");
    let analysis = read_to_string("working_mut_tuple_let.csv");

    match refactor::refactor::rename_variable(file, &input, &analysis, "x", "10") {
        Ok(x) => assert_eq!(output.trim(), x.get(file).unwrap().trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn working_variable_10() {
    let file = "working_mut_tuple_let2.rs";
    let input = read_to_string(file);
    let output = read_to_string("working_mut_tuple_let2_out.rs");
    let analysis = read_to_string("working_mut_tuple_let2.csv");

    match refactor::refactor::rename_variable(file, &input, &analysis, "x", "11") {
        Ok(x) => assert_eq!(output.trim(), x.get(file).unwrap().trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn working_variable_11() {
    let file = "working_mut_tuple_let3.rs";
    let input = read_to_string(file);
    let output = read_to_string("working_mut_tuple_let3_out.rs");
    let analysis = read_to_string("working_mut_tuple_let3.csv");

    match refactor::refactor::rename_variable(file, &input, &analysis, "x", "11") {
        Ok(x) => assert_eq!(output.trim(), x.get(file).unwrap().trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn prevented_variable_1() {
    let input = read_to_string("basic_rename.rs");
    let analysis = read_to_string("basic_rename.csv");

    match refactor::refactor::rename_variable(&"basic_rename.rs", &input, &analysis, "j", "36") {
        Ok(_) => assert!(false),
        Err(x) => assert_eq!(Response::Conflict, x)
    }
}

#[test]
fn prevented_variable_2() {
    let input = read_to_string("basic_rename.rs");
    let analysis = read_to_string("basic_rename.csv");

    match refactor::refactor::rename_variable(&"basic_rename.rs", &input, &analysis, "x", "36") {
        Ok(_) => assert!(false),
        Err(x) => assert_eq!(Response::Conflict, x)
    }
}

#[test]
fn prevented_variable_3() {
    let input = read_to_string("override.rs");
    let analysis = read_to_string("override.csv");

    match refactor::refactor::rename_variable(&"override.rs", &input, &analysis, "v", "9") {
        Ok(_) => assert!(false),
        Err(x) => assert_eq!(Response::Conflict, x)
    }
}

#[test]
fn prevented_variable_4() {
    let file = "name_conflict_method.rs";
    let input = read_to_string(file);
    let analysis = read_to_string("name_conflict_method.csv");
    match refactor::refactor::rename_variable(file, &input, &analysis, "foo", "12") {
        Ok(_) => assert!(false),
        Err(x) => assert_eq!(Response::Conflict, x)
    }
}

#[test]
fn prevented_variable_5() {
    let file = "name_conflict_type.rs";
    let input = read_to_string(file);
    let analysis = read_to_string("name_conflict_type.csv");
    match refactor::refactor::rename_variable(file, &input, &analysis, "Foo", "12") {
        Ok(_) => assert!(false),
        Err(x) => assert_eq!(Response::Conflict, x)
    }
}

#[test]
fn prevented_variable_6() {
    let file = "name_conflict_type_local.rs";
    let input = read_to_string(file);
    let analysis = read_to_string("name_conflict_type_local.csv");
    match refactor::refactor::rename_variable(file, &input, &analysis, "Foo", "13") {
        Ok(_) => assert!(false),
        Err(x) => assert_eq!(Response::Conflict, x)
    }
}

#[test]
fn prevented_variable_7() {
    let file = "name_conflict_type_local2.rs";
    let input = read_to_string(file);
    let analysis = read_to_string("name_conflict_type_local2.csv");
    match refactor::refactor::rename_variable(file, &input, &analysis, "Foo", "9") {
        Ok(_) => assert!(false),
        Err(x) => assert_eq!(Response::Conflict, x)
    }
}

#[test]
fn prevented_variable_8() {
    let file = "name_conflict_method_local.rs";
    let input = read_to_string(file);
    let analysis = read_to_string("name_conflict_method_local.csv");
    match refactor::refactor::rename_variable(file, &input, &analysis, "foo", "13") {
        Ok(_) => assert!(false),
        Err(x) => assert_eq!(Response::Conflict, x)
    }
}

#[test]
fn prevented_variable_9() {
    let file = "name_conflict_method_local2.rs";
    let input = read_to_string(file);
    let analysis = read_to_string("name_conflict_method_local2.csv");
    match refactor::refactor::rename_variable(file, &input, &analysis, "Foo", "9") {
        Ok(_) => assert!(false),
        Err(x) => assert_eq!(Response::Conflict, x)
    }

    // fn main() {
    //     let a = 2;
    //     fn foo() {}
    //     foo();
    // }
    //
    // Unlike the type case, this is not detected by the resolve_path
    // This test is slightly modified, using a, to make sure only resolving occurs
    // (Rather than a full run)

}

#[test]
fn prevented_variable_10() {
    let file = "name_conflict_global.rs";
    let input = read_to_string(file);
    let analysis = read_to_string("name_conflict_global.csv");
    match refactor::refactor::rename_variable(file, &input, &analysis, "FOO", "12") {
        Ok(_) => assert!(false),
        Err(x) => assert_eq!(Response::Conflict, x)
    }
}

#[test]
fn prevented_variable_11() {
    let file = "name_conflict_type_global.rs";
    let input = read_to_string(file);
    let analysis = read_to_string("name_conflict_type_global.csv");
    match refactor::refactor::rename_variable(file, &input, &analysis, "Foo", "7") {
        Ok(_) => assert!(false),
        Err(x) => assert_eq!(Response::Conflict, x)
    }
}

#[test]
fn prevented_variable_12() {
    let file = "name_conflict_method_global.rs";
    let input = read_to_string(file);
    let analysis = read_to_string("name_conflict_method_global.csv");
    match refactor::refactor::rename_variable(file, &input, &analysis, "foo", "4") {
        Ok(_) => assert!(false),
        Err(x) => assert_eq!(Response::Conflict, x)
    }
}

#[test]
fn working_struct_1() {
    let file = "basic_struct.rs";
    let input = read_to_string(file);
    let output = read_to_string("working_struct_1_out.rs");
    let analysis = read_to_string("basic_struct.csv");

    match refactor::refactor::rename_type(&"basic_struct.rs", &input, &analysis, "Pointer", "4") {
        Ok(x) => assert_eq!(output.trim(), x.get(file).unwrap().trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn working_struct_2() {
    // ::Point mentioned instead of Point
    let file = "scoped_struct.rs";
    let input = read_to_string(file);
    let output = read_to_string("working_struct_1_out.rs");
    let analysis = read_to_string("scoped_struct.csv");

    match refactor::refactor::rename_type(&"basic_struct.rs", &input, &analysis, "Pointer", "4") {
        Ok(x) => assert_eq!(output.trim(), x.get(file).unwrap().trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn working_enum_1() {
    let file = "basic_enum.rs";
    let input = read_to_string(file);
    let output = read_to_string("working_enum_1_out.rs");
    let analysis = read_to_string("basic_enum.csv");

    match refactor::refactor::rename_type(&"basic_enum.rs", &input, &analysis, "YesNo", "4") {
        Ok(x) => assert_eq!(output.trim(), x.get(file).unwrap().trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn working_enum_2() {
    let file = "modref_enum.rs";
    let input = read_to_string(file);
    let output = read_to_string("working_enum_2_out.rs");
    let analysis = read_to_string("modref_enum.csv");

    match refactor::refactor::rename_type(&"modref_enum.rs", &input, &analysis, "YesNo", "7") {
        Ok(x) => assert_eq!(output.trim(), x.get(file).unwrap().trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn prevented_struct_1() {
    let input = read_to_string("conflict_struct.rs");
    let analysis = read_to_string("conflict_struct.csv");

    match refactor::refactor::rename_type(&"conflict_struct.rs", &input, &analysis, "P", "4") {
        Ok(_) => assert!(false),
        Err(x) => assert_eq!(Response::Conflict, x)
    }
}

#[test]
fn prevented_struct_2() {
    let input = read_to_string("conflict_mod_struct.rs");
    let analysis = read_to_string("conflict_mod_struct.csv");

    match refactor::refactor::rename_type(&"conflict_mod_struct.rs", &input, &analysis, "B", "6") {
        Ok(_) => assert!(false),
        Err(x) => assert_eq!(Response::Conflict, x)
    }
}

#[test]
fn prevented_struct_3() {
    let input = read_to_string("conflict_use_mod_struct.rs");
    let analysis = read_to_string("conflict_use_mod_struct.csv");

    match refactor::refactor::rename_type(&"conflict_use_mod_struct.rs", &input, &analysis, "B", "6") {
        Ok(_) => assert!(false),
        Err(x) => assert_eq!(Response::Conflict, x)
    }
}

#[test]
fn working_method_1() {
    let file = "basic_default_method.rs";
    let input = read_to_string(file);
    let output = read_to_string("working_method_1_out.rs");
    let analysis = read_to_string("basic_default_method.csv");

    match refactor::refactor::rename_function(&"basic_default_method.rs", &input, &analysis, "func", "5") {
        Ok(x) => assert_eq!(output.trim(), x.get(file).unwrap().trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn working_method_2() {
    let file = "impl_override_method.rs";
    let input = read_to_string(file);
    let output = read_to_string("working_method_2_out.rs");
    let analysis = read_to_string("impl_override_method.csv");

    match refactor::refactor::rename_function(&"impl_override_method.rs", &input, &analysis, "func", "5") {
        Ok(x) => assert_eq!(output.trim(), x.get(file).unwrap().trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn working_method_3() {
    let file = "alex_override_method.rs";
    let input = read_to_string(file);
    let output = read_to_string("alex_override_method_out2.rs");
    let analysis = read_to_string("alex_override_method.csv");

    match refactor::refactor::rename_function(&"alex_override_method.rs", &input, &analysis, "grue", "74") {
        Ok(x) => assert_eq!(output.trim(), x.get(file).unwrap().trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn not_working_method_1() {
    let input = read_to_string("alex_override_method.rs");
    let analysis = read_to_string("alex_override_method.csv");

    match refactor::refactor::rename_function(&"alex_override_method.rs", &input, &analysis, "foo", "74") {
        Ok(_) => assert!(false),
        Err(x) => assert_eq!(Response::Conflict, x)
    }
}
