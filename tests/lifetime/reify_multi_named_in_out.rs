use std::fmt::Debug;

fn foo<'a, 'b>(x: &'a Debug, y: &'b Debug) {
    println!("{:?} {:?}", x, y);
}

fn main() {
    foo(&2, &2);
    foo(&"STRING", &"STRING");
}

