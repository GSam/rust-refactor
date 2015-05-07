struct Pointer {
    x: i32,
    y: i32
}

impl Pointer {
    fn new(x: i32, y: i32) -> Pointer {
        Pointer {
            x: x,
            y: y
        }
    }
}

fn main() {
    let p: Pointer = Pointer::new(1, 1);
}
