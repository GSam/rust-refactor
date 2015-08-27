use std::fmt::Debug;


fn foo<'a>(x: &'a Debug) -> &'static Debug {
    panic!();
}

fn main() {
    foo(&2);
    foo(&"STRING");
}
