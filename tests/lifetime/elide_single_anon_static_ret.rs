use std::fmt::Debug;


fn foo(x: &Debug) -> &'static Debug {
    panic!();
}

fn main() {
    foo(&2);
    foo(&"STRING");
}
