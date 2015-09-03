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

###Known issues:
There appears to be issues with running the tool on Windows. The cause appears to be the inability to locate the stdlib, but neither setting the linker flag or the sysroot appears to be of any use.

###TODO:
Renaming enum, variants + struct variants

Renaming traits
