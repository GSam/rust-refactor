use std::fmt::Debug;

struct Point;

impl Point {
    fn foo(&self, x: &Debug, y: &Debug) -> &Point {
        println!("{:?} {:?}", x, y);
        self
    }
}

fn main() {
    let p = Point;
    p.foo(&2, &2);
    p.foo(&"STRING", &"STRING");
}

