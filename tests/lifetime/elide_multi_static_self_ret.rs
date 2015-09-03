use std::fmt::Debug;

struct Point;

impl Point {
    fn foo<'a, 'b>(&'a self, x: &'b Debug, y: &'static Debug) -> &'a Point {
        self
    }
}

fn main() {
}

