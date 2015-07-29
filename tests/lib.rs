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
    let file = "tests/variable/basic_rename.rs";
    let output = read_to_string("tests/variable/working_rename_1_out.rs");
    let analysis = read_to_string("tests/variable/basic_rename.csv");

    match refactor::refactor::rename_variable(&"tests/variable/basic_rename.rs", &analysis, "hello", "9") {
        Ok(x) => assert_eq!(output.trim(), x.get(file).unwrap().trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn working_variable_2() {
    let file = "tests/variable/basic_rename.rs";
    let output = read_to_string("tests/variable/working_rename_2_out.rs");
    let analysis = read_to_string("tests/variable/basic_rename.csv");

    match refactor::refactor::rename_variable(&"tests/variable/basic_rename.rs", &analysis, "hello", "17") {
        Ok(x) => assert_eq!(output.trim(), x.get(file).unwrap().trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn working_variable_3() {
    let file = "tests/variable/alex_var_test.rs";
    let output = read_to_string("tests/variable/alex_var_test_out.rs");
    let analysis = read_to_string("tests/variable/alex_var_test.csv");

    match refactor::refactor::rename_variable(&"tests/variable/alex_var_test.rs", &analysis, "bar", "14") {
        Ok(x) => assert_eq!(output.trim(), x.get(file).unwrap().trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn working_variable_4() {
    let file = "tests/variable/alex_var_test.rs";
    let output = read_to_string("tests/variable/alex_var_test_out2.rs");
    let analysis = read_to_string("tests/variable/alex_var_test.csv");

    match refactor::refactor::rename_variable(&"tests/variable/alex_var_test.rs", &analysis, "bar", "4") {
        Ok(x) => assert_eq!(output.trim(), x.get(file).unwrap().trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn working_variable_5() {
    let file = "tests/variable/const_rename.rs";
    let output = read_to_string("tests/variable/const_rename_out.rs");
    let analysis = read_to_string("tests/variable/const_rename.csv");

    match refactor::refactor::rename_variable(&"tests/variable/const_rename.rs", &analysis, "BAZ", "8") {
        Ok(x) => assert_eq!(output.trim(), x.get(file).unwrap().trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn working_variable_6() {
    let file = "tests/variable/working_fn_local.rs";
    let output = read_to_string("tests/variable/working_fn_local_out.rs");
    let analysis = read_to_string("tests/variable/working_fn_local.csv");

    match refactor::refactor::rename_variable(&"tests/variable/working_fn_local.rs", &analysis, "Foo", "9") {
        Ok(x) => assert_eq!(output.trim(), x.get(file).unwrap().trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn working_variable_7() {
    let file = "tests/variable/working_nested.rs";
    let output = read_to_string("tests/variable/working_nested_out.rs");
    let analysis = read_to_string("tests/variable/working_nested.csv");

    match refactor::refactor::rename_variable(&"tests/variable/working_nested.rs", &analysis, "b", "16") {
        Ok(x) => assert_eq!(output.trim(), x.get(file).unwrap().trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn working_variable_8() {
    let file = "tests/variable/working_tuple_let.rs";
    let output = read_to_string("tests/variable/working_tuple_let_out.rs");
    let analysis = read_to_string("tests/variable/working_tuple_let.csv");

    match refactor::refactor::rename_variable(file, &analysis, "x", "10") {
        Ok(x) => assert_eq!(output.trim(), x.get(file).unwrap().trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn working_variable_9() {
    let file = "tests/variable/working_mut_tuple_let.rs";
    let output = read_to_string("tests/variable/working_mut_tuple_let_out.rs");
    let analysis = read_to_string("tests/variable/working_mut_tuple_let.csv");

    match refactor::refactor::rename_variable(file, &analysis, "x", "10") {
        Ok(x) => assert_eq!(output.trim(), x.get(file).unwrap().trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn working_variable_10() {
    let file = "tests/variable/working_mut_tuple_let2.rs";
    let output = read_to_string("tests/variable/working_mut_tuple_let2_out.rs");
    let analysis = read_to_string("tests/variable/working_mut_tuple_let2.csv");

    match refactor::refactor::rename_variable(file, &analysis, "x", "11") {
        Ok(x) => assert_eq!(output.trim(), x.get(file).unwrap().trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn working_variable_11() {
    let file = "tests/variable/working_mut_tuple_let3.rs";
    let output = read_to_string("tests/variable/working_mut_tuple_let3_out.rs");
    let analysis = read_to_string("tests/variable/working_mut_tuple_let3.csv");

    match refactor::refactor::rename_variable(file, &analysis, "x", "11") {
        Ok(x) => assert_eq!(output.trim(), x.get(file).unwrap().trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn prevented_variable_1() {
    let file = "tests/variable/basic_rename.rs";
    let analysis = read_to_string("tests/variable/basic_rename.csv");

    match refactor::refactor::rename_variable(&"tests/variable/basic_rename.rs", &analysis, "j", "36") {
        Ok(_) => assert!(false),
        Err(x) => assert_eq!(Response::Conflict, x)
    }
}

#[test]
fn prevented_variable_2() {
    let file = "tests/variable/basic_rename.rs";
    let analysis = read_to_string("tests/variable/basic_rename.csv");

    match refactor::refactor::rename_variable(&"tests/variable/basic_rename.rs", &analysis, "x", "36") {
        Ok(_) => assert!(false),
        Err(x) => assert_eq!(Response::Conflict, x)
    }
}

#[test]
fn prevented_variable_3() {
    let file = "tests/variable/override.rs";
    let analysis = read_to_string("tests/variable/override.csv");

    match refactor::refactor::rename_variable(&"tests/variable/override.rs", &analysis, "v", "9") {
        Ok(_) => assert!(false),
        Err(x) => assert_eq!(Response::Conflict, x)
    }
}

#[test]
fn prevented_variable_4() {
    let file = "tests/variable/name_conflict_method.rs";
    let analysis = read_to_string("tests/variable/name_conflict_method.csv");
    match refactor::refactor::rename_variable(file, &analysis, "foo", "12") {
        Ok(_) => assert!(false),
        Err(x) => assert_eq!(Response::Conflict, x)
    }
}

#[test]
fn prevented_variable_5() {
    let file = "tests/variable/name_conflict_type.rs";
    let analysis = read_to_string("tests/variable/name_conflict_type.csv");
    match refactor::refactor::rename_variable(file, &analysis, "Foo", "12") {
        Ok(_) => assert!(false),
        Err(x) => assert_eq!(Response::Conflict, x)
    }
}

#[test]
fn prevented_variable_6() {
    let file = "tests/variable/name_conflict_type_local.rs";
    let analysis = read_to_string("tests/variable/name_conflict_type_local.csv");
    match refactor::refactor::rename_variable(file, &analysis, "Foo", "13") {
        Ok(_) => assert!(false),
        Err(x) => assert_eq!(Response::Conflict, x)
    }
}

#[test]
fn prevented_variable_7() {
    let file = "tests/variable/name_conflict_type_local2.rs";
    let analysis = read_to_string("tests/variable/name_conflict_type_local2.csv");
    match refactor::refactor::rename_variable(file, &analysis, "Foo", "9") {
        Ok(_) => assert!(false),
        Err(x) => assert_eq!(Response::Conflict, x)
    }
}

#[test]
fn prevented_variable_8() {
    let file = "tests/variable/name_conflict_method_local.rs";
    let analysis = read_to_string("tests/variable/name_conflict_method_local.csv");
    match refactor::refactor::rename_variable(file, &analysis, "foo", "13") {
        Ok(_) => assert!(false),
        Err(x) => assert_eq!(Response::Conflict, x)
    }
}

#[test]
fn prevented_variable_9() {
    let file = "tests/variable/name_conflict_method_local2.rs";
    let analysis = read_to_string("tests/variable/name_conflict_method_local2.csv");
    match refactor::refactor::rename_variable(file, &analysis, "Foo", "9") {
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
    let file = "tests/variable/name_conflict_global.rs";
    let analysis = read_to_string("tests/variable/name_conflict_global.csv");
    match refactor::refactor::rename_variable(file, &analysis, "FOO", "12") {
        Ok(_) => assert!(false),
        Err(x) => assert_eq!(Response::Conflict, x)
    }
}

#[test]
fn prevented_variable_11() {
    let file = "tests/variable/name_conflict_type_global.rs";
    let analysis = read_to_string("tests/variable/name_conflict_type_global.csv");
    match refactor::refactor::rename_variable(file, &analysis, "Foo", "7") {
        Ok(_) => assert!(false),
        Err(x) => assert_eq!(Response::Conflict, x)
    }
}

#[test]
fn prevented_variable_12() {
    let file = "tests/variable/name_conflict_method_global.rs";
    let analysis = read_to_string("tests/variable/name_conflict_method_global.csv");
    match refactor::refactor::rename_variable(file, &analysis, "foo", "4") {
        Ok(_) => assert!(false),
        Err(x) => assert_eq!(Response::Conflict, x)
    }
}

#[test]
fn working_argument_1() {
    let file = "tests/variable/fn_args_1.rs";
    let output = read_to_string("tests/variable/fn_args_1_out.rs");
    let analysis = read_to_string("tests/variable/fn_args_1.csv");
    match refactor::refactor::rename_variable(file, &analysis, "z", "6") {
        Ok(x) => assert_eq!(output.trim(), x.get(file).unwrap().trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn working_argument_2() {
    let file = "tests/variable/fn_args_2.rs";
    let output = read_to_string("tests/variable/fn_args_2_1_out.rs");
    let analysis = read_to_string("tests/variable/fn_args_2.csv");
    match refactor::refactor::rename_variable(file, &analysis, "z", "10") {
        Ok(x) => assert_eq!(output.trim(), x.get(file).unwrap().trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn working_argument_3() {
    let file = "tests/variable/fn_args_2.rs";
    let output = read_to_string("tests/variable/fn_args_2_2_out.rs");
    let analysis = read_to_string("tests/variable/fn_args_2.csv");
    match refactor::refactor::rename_variable(file, &analysis, "z", "15") {
        Ok(x) => assert_eq!(output.trim(), x.get(file).unwrap().trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn prevented_argument_1() {
    let file = "tests/variable/fn_args_1.rs";
    let analysis = read_to_string("tests/variable/fn_args_1.csv");
    match refactor::refactor::rename_variable(file, &analysis, "c", "6") {
        Ok(_) => assert!(false),
        Err(x) => assert_eq!(Response::Conflict, x)
    }
}

#[test]
fn prevented_argument_2() {
    let file = "tests/variable/fn_args_1.rs";
    let analysis = read_to_string("tests/variable/fn_args_1.csv");
    match refactor::refactor::rename_variable(file, &analysis, "foo", "6") {
        Ok(_) => assert!(false),
        Err(x) => assert_eq!(Response::Conflict, x)
    }
}

#[test]
fn working_struct_1() {
    let file = "tests/type/basic_struct.rs";
    let output = read_to_string("tests/type/working_struct_1_out.rs");
    let analysis = read_to_string("tests/type/basic_struct.csv");

    match refactor::refactor::rename_type(&"tests/type/basic_struct.rs", &analysis, "Pointer", "4") {
        Ok(x) => assert_eq!(output.trim(), x.get(file).unwrap().trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn working_struct_2() {
    // ::Point mentioned instead of Point
    let file = "tests/type/scoped_struct.rs";
    let output = read_to_string("tests/type/working_struct_1_out.rs");
    let analysis = read_to_string("tests/type/scoped_struct.csv");

    match refactor::refactor::rename_type(&"tests/type/basic_struct.rs", &analysis, "Pointer", "4") {
        Ok(x) => assert_eq!(output.trim(), x.get(file).unwrap().trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn working_enum_1() {
    let file = "tests/type/basic_enum.rs";
    let output = read_to_string("tests/type/working_enum_1_out.rs");
    let analysis = read_to_string("tests/type/basic_enum.csv");

    match refactor::refactor::rename_type(&"tests/type/basic_enum.rs", &analysis, "YesNo", "4") {
        Ok(x) => assert_eq!(output.trim(), x.get(file).unwrap().trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn working_enum_2() {
    let file = "tests/type/modref_enum.rs";
    let output = read_to_string("tests/type/working_enum_2_out.rs");
    let analysis = read_to_string("tests/type/modref_enum.csv");

    match refactor::refactor::rename_type(&"tests/type/modref_enum.rs", &analysis, "YesNo", "7") {
        Ok(x) => assert_eq!(output.trim(), x.get(file).unwrap().trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn prevented_struct_1() {
    let file = "tests/type/conflict_struct.rs";
    let analysis = read_to_string("tests/type/conflict_struct.csv");

    match refactor::refactor::rename_type(&"tests/type/conflict_struct.rs", &analysis, "P", "4") {
        Ok(_) => assert!(false),
        Err(x) => assert_eq!(Response::Conflict, x)
    }
}

#[test]
fn prevented_struct_2() {
    let file = "tests/type/conflict_mod_struct.rs";
    let analysis = read_to_string("tests/type/conflict_mod_struct.csv");

    match refactor::refactor::rename_type(&"tests/type/conflict_mod_struct.rs", &analysis, "B", "6") {
        Ok(_) => assert!(false),
        Err(x) => assert_eq!(Response::Conflict, x)
    }
}

#[test]
fn prevented_struct_3() {
    let file = "tests/type/conflict_use_mod_struct.rs";
    let analysis = read_to_string("tests/type/conflict_use_mod_struct.csv");

    match refactor::refactor::rename_type(&"tests/type/conflict_use_mod_struct.rs", &analysis, "B", "6") {
        Ok(_) => assert!(false),
        Err(x) => assert_eq!(Response::Conflict, x)
    }
}

#[test]
fn working_method_1() {
    let file = "tests/function/basic_default_method.rs";
    let output = read_to_string("tests/function/working_method_1_out.rs");
    let analysis = read_to_string("tests/function/basic_default_method.csv");

    match refactor::refactor::rename_function(&"tests/function/basic_default_method.rs", &analysis, "func", "5") {
        Ok(x) => assert_eq!(output.trim(), x.get(file).unwrap().trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn working_method_2() {
    let file = "tests/function/impl_override_method.rs";
    let output = read_to_string("tests/function/working_method_2_out.rs");
    let analysis = read_to_string("tests/function/impl_override_method.csv");

    match refactor::refactor::rename_function(&"tests/function/impl_override_method.rs", &analysis, "func", "5") {
        Ok(x) => assert_eq!(output.trim(), x.get(file).unwrap().trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn working_method_3() {
    let file = "tests/function/alex_override_method.rs";
    let output = read_to_string("tests/function/alex_override_method_out2.rs");
    let analysis = read_to_string("tests/function/alex_override_method.csv");

    match refactor::refactor::rename_function(&"tests/function/alex_override_method.rs", &analysis, "grue", "74") {
        Ok(x) => assert_eq!(output.trim(), x.get(file).unwrap().trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn not_working_method_1() {
    let file = "tests/function/alex_override_method.rs";
    let analysis = read_to_string("tests/function/alex_override_method.csv");

    match refactor::refactor::rename_function(&"tests/function/alex_override_method.rs", &analysis, "foo", "74") {
        Ok(_) => assert!(false),
        Err(x) => assert_eq!(Response::Conflict, x)
    }
}

#[test]
fn working_fn_1() {
    let file = "tests/function/basic_function.rs";
    let output = read_to_string("tests/function/basic_function_out.rs");
    let analysis = read_to_string("tests/function/basic_function.csv");

    match refactor::refactor::rename_function(&file, &analysis, "bar", "4") {
        Ok(x) => assert_eq!(output.trim(), x.get(file).unwrap().trim()),
        Err(_) => assert!(false)
    }
}

#[test]
fn not_working_fn_1() {
    let file = "tests/function/basic_function.rs";
    let analysis = read_to_string("tests/function/basic_function.csv");
    match refactor::refactor::rename_function(&file, &analysis, "main", "4") {
        Ok(_) => assert!(false),
        Err(x) => assert_eq!(Response::Conflict, x)
    }
}

#[test]
fn multi_file_1() {
    let file = "tests/multi-file/simple_function_1/main.rs";
    let changed1 = file;
    let changed2 = "tests/multi-file/simple_function_1/foo.rs";
    let output1 = read_to_string("tests/multi-file/simple_function_1/main_out.rs");
    let output2 = read_to_string("tests/multi-file/simple_function_1/foo_out.rs");
    let analysis = read_to_string("tests/multi-file/simple_function_1/main.csv");

    match refactor::refactor::rename_function(&file, &analysis, "boo", "6") {
        Ok(x) => {
            assert_eq!(output1.trim(), x.get(changed1).unwrap().trim());
            assert_eq!(output2.trim(), x.get(changed2).unwrap().trim());
        },
        Err(_) => assert!(false)
    }

}

#[test]
fn multi_file_2() {
    let file = "tests/multi-file/simple_function_2/main.rs";
    let changed1 = file;
    let changed2 = "tests/multi-file/simple_function_2/foo/mod.rs";
    let output1 = read_to_string("tests/multi-file/simple_function_2/main_out.rs");
    let output2 = read_to_string("tests/multi-file/simple_function_2/foo/mod_out.rs");
    let analysis = read_to_string("tests/multi-file/simple_function_2/main.csv");

    match refactor::refactor::rename_function(&file, &analysis, "boo", "6") {
        Ok(x) => {
            assert_eq!(output1.trim(), x.get(changed1).unwrap().trim());
            assert_eq!(output2.trim(), x.get(changed2).unwrap().trim());
        },
        Err(_) => assert!(false)
    }

}
