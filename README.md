# rust-refactor
Rust refactoring project

A tool to help refactor rust programs.

Currently supports:
Simple variable renames, function renaming, struct renaming.

```
% refactor var "tests/variable/basic_rename.csv" "tests/variable/basic_rename.rs" x:-1:-1  new_name
```

Requires a modified compiler at:
https://github.com/GSam/rust

Before running the tool, an additional environment variable is required for the internal compilation using stdlibs, RUST_FOLDER needs to be set to either the stage2 lib folder or the stage2/rustlib/XXX/lib folder currently.

#TODO:
Renaming fields

Renaming enum, variants + struct variants

Renaming traits

#TESTS TODO:
Simple static dispatch

Simple parameterized static dispatch

Dynamic dispatch
