# rust-refactor
Rust refactoring project

A tool to help refactor rust programs.

Currently supports:
Simple variable renames, function renaming, struct renaming.

```
% refactor.exe var "tests\variable\basic_rename.csv" "tests\variable\basic_rename.rs" x:-1:-1  new_name
```

#TODO:
Multiple file projects, multi-module.

Renaming fields

Renaming enum, variants + struct variants

Renaming traits

Conflict resolution for functions + types

#TESTS TODO:
Simple static dispatch

Simple parameterized static dispatch

Dynamic dispatch
