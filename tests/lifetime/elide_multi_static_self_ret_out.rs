use std::fmt::Debug;

struct Point;

impl Point {
    fn foo(&self, x: &Debug, y: &'static Debug) -> &Point {
        self
    }
}

fn main() {
}

