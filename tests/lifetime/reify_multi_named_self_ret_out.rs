use std::fmt::Debug;

struct Point;

impl Point {
    fn foo<'a,'b,'c>(&'a self, x: &'b Debug, y: &'c Debug) -> &'a Point {
        println!("{:?} {:?}", x, y);
        self
    }
}

fn main() {
    let p = Point;
    p.foo(&2, &2);
    p.foo(&"STRING", &"STRING");
}

