#![feature(io)]

extern crate refactor;

use std::fs::File;
use std::io::prelude::*;
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
    let input = read_to_string("tests/variable/basic_rename.rs");
    let output = read_to_string("tests/variable/working_rename_1_out.rs");
    let analysis = read_to_string("tests/variable/basic_rename.csv");

    match refactor::refactor::rename_variable(&"tests/variable/basic_rename.rs", &input, &analysis, "hello", "9") {
        Ok(x) => assert_eq!(output.trim(), x.trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn working_variable_2() {
    let input = read_to_string("tests/variable/basic_rename.rs");
    let output = read_to_string("tests/variable/working_rename_2_out.rs");
    let analysis = read_to_string("tests/variable/basic_rename.csv");

    match refactor::refactor::rename_variable(&"tests/variable/basic_rename.rs", &input, &analysis, "hello", "17") {
        Ok(x) => assert_eq!(output.trim(), x.trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn working_variable_3() {
    let input = read_to_string("tests/variable/alex_var_test.rs");
    let output = read_to_string("tests/variable/alex_var_test_out.rs");
    let analysis = read_to_string("tests/variable/alex_var_test.csv");

    match refactor::refactor::rename_variable(&"tests/variable/alex_var_test.rs", &input, &analysis, "bar", "14") {
        Ok(x) => assert_eq!(output.trim(), x.trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn working_variable_4() {
    let input = read_to_string("tests/variable/alex_var_test.rs");
    let output = read_to_string("tests/variable/alex_var_test_out2.rs");
    let analysis = read_to_string("tests/variable/alex_var_test.csv");

    match refactor::refactor::rename_variable(&"tests/variable/alex_var_test.rs", &input, &analysis, "bar", "4") {
        Ok(x) => assert_eq!(output.trim(), x.trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn working_variable_5() {
    let input = read_to_string("tests/variable/const_rename.rs");
    let output = read_to_string("tests/variable/const_rename_out.rs");
    let analysis = read_to_string("tests/variable/const_rename.csv");

    match refactor::refactor::rename_variable(&"tests/variable/const_rename.rs", &input, &analysis, "BAZ", "8") {
        Ok(x) => assert_eq!(output.trim(), x.trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn prevented_variable_1() {
    let input = read_to_string("tests/variable/basic_rename.rs");
    let analysis = read_to_string("tests/variable/basic_rename.csv");

    match refactor::refactor::rename_variable(&"tests/variable/basic_rename.rs", &input, &analysis, "j", "36") {
        Ok(_) => assert!(false),
        Err(x) => assert_eq!(Response::Conflict, x)
    }
}

#[test]
fn prevented_variable_2() {
    let input = read_to_string("tests/variable/basic_rename.rs");
    let analysis = read_to_string("tests/variable/basic_rename.csv");

    match refactor::refactor::rename_variable(&"tests/variable/basic_rename.rs", &input, &analysis, "x", "36") {
        Ok(_) => assert!(false),
        Err(x) => assert_eq!(Response::Conflict, x)
    }
}

#[test]
fn prevented_variable_3() {
    let input = read_to_string("tests/variable/override.rs");
    let analysis = read_to_string("tests/variable/override.csv");

    match refactor::refactor::rename_variable(&"tests/variable/override.rs", &input, &analysis, "v", "9") {
        Ok(_) => assert!(false),
        Err(x) => assert_eq!(Response::Conflict, x)
    }
}

#[test]
fn prevented_variable_4() {
    let file = "tests/variable/name_conflict_method.rs";
    let input = read_to_string(file);
    let analysis = read_to_string("tests/variable/name_conflict_method.csv");
    match refactor::refactor::rename_variable(file, &input, &analysis, "foo", "12") {
        Ok(_) => assert!(false),
        Err(x) => assert_eq!(Response::Conflict, x)
    }
}

#[test]
fn prevented_variable_5() {
    let file = "tests/variable/name_conflict_type.rs";
    let input = read_to_string(file);
    let analysis = read_to_string("tests/variable/name_conflict_type.csv");
    match refactor::refactor::rename_variable(file, &input, &analysis, "Foo", "12") {
        Ok(_) => assert!(false),
        Err(x) => assert_eq!(Response::Conflict, x)
    }
}

#[test]
fn working_struct_1() {
    let input = read_to_string("tests/type/basic_struct.rs");
    let output = read_to_string("tests/type/working_struct_1_out.rs");
    let analysis = read_to_string("tests/type/basic_struct.csv");

    match refactor::refactor::rename_type(&"tests/type/basic_struct.rs", &input, &analysis, "Pointer", "4") {
        Ok(x) => assert_eq!(output.trim(), x.trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn working_struct_2() {
    // ::Point mentioned instead of Point
    let input = read_to_string("tests/type/scoped_struct.rs");
    let output = read_to_string("tests/type/working_struct_1_out.rs");
    let analysis = read_to_string("tests/type/scoped_struct.csv");

    match refactor::refactor::rename_type(&"tests/type/basic_struct.rs", &input, &analysis, "Pointer", "4") {
        Ok(x) => assert_eq!(output.trim(), x.trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn working_enum_1() {
    let input = read_to_string("tests/type/basic_enum.rs");
    let output = read_to_string("tests/type/working_enum_1_out.rs");
    let analysis = read_to_string("tests/type/basic_enum.csv");

    match refactor::refactor::rename_type(&"tests/type/basic_enum.rs", &input, &analysis, "YesNo", "4") {
        Ok(x) => assert_eq!(output.trim(), x.trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn working_enum_2() {
    let input = read_to_string("tests/type/modref_enum.rs");
    let output = read_to_string("tests/type/working_enum_2_out.rs");
    let analysis = read_to_string("tests/type/modref_enum.csv");

    match refactor::refactor::rename_type(&"tests/type/modref_enum.rs", &input, &analysis, "YesNo", "7") {
        Ok(x) => assert_eq!(output.trim(), x.trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn prevented_struct_1() {
    let input = read_to_string("tests/type/conflict_struct.rs");
    let analysis = read_to_string("tests/type/conflict_struct.csv");

    match refactor::refactor::rename_type(&"tests/type/conflict_struct.rs", &input, &analysis, "P", "4") {
        Ok(_) => assert!(false),
        Err(x) => assert_eq!(Response::Conflict, x)
    }
}

#[test]
fn prevented_struct_2() {
    let input = read_to_string("tests/type/conflict_mod_struct.rs");
    let analysis = read_to_string("tests/type/conflict_mod_struct.csv");

    match refactor::refactor::rename_type(&"tests/type/conflict_mod_struct.rs", &input, &analysis, "B", "6") {
        Ok(_) => assert!(false),
        Err(x) => assert_eq!(Response::Conflict, x)
    }
}

#[test]
fn prevented_struct_3() {
    let input = read_to_string("tests/type/conflict_use_mod_struct.rs");
    let analysis = read_to_string("tests/type/conflict_use_mod_struct.csv");

    match refactor::refactor::rename_type(&"tests/type/conflict_use_mod_struct.rs", &input, &analysis, "B", "6") {
        Ok(_) => assert!(false),
        Err(x) => assert_eq!(Response::Conflict, x)
    }
}

#[test]
fn working_method_1() {
    let input = read_to_string("tests/function/basic_default_method.rs");
    let output = read_to_string("tests/function/working_method_1_out.rs");
    let analysis = read_to_string("tests/function/basic_default_method.csv");

    match refactor::refactor::rename_function(&"tests/function/basic_default_method.rs", &input, &analysis, "func", "5") {
        Ok(x) => assert_eq!(output.trim(), x.trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn working_method_2() {
    let input = read_to_string("tests/function/impl_override_method.rs");
    let output = read_to_string("tests/function/working_method_2_out.rs");
    let analysis = read_to_string("tests/function/impl_override_method.csv");

    match refactor::refactor::rename_function(&"tests/function/impl_override_method.rs", &input, &analysis, "func", "5") {
        Ok(x) => assert_eq!(output.trim(), x.trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn working_method_3() {
    let input = read_to_string("tests/function/alex_override_method.rs");
    let output = read_to_string("tests/function/alex_override_method_out2.rs");
    let analysis = read_to_string("tests/function/alex_override_method.csv");

    match refactor::refactor::rename_function(&"tests/function/alex_override_method.rs", &input, &analysis, "grue", "74") {
        Ok(x) => assert_eq!(output.trim(), x.trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn not_working_method_1() {
    let input = read_to_string("tests/function/alex_override_method.rs");
    let output = read_to_string("tests/function/alex_override_method_out.rs");
    let analysis = read_to_string("tests/function/alex_override_method.csv");

    match refactor::refactor::rename_function(&"tests/function/alex_override_method.rs", &input, &analysis, "foo", "74") {
        Ok(_) => assert!(false),
        Err(x) => assert_eq!(Response::Conflict, x)
    }
}
