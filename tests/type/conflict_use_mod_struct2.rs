mod group {
    pub struct A;
}

struct B;
use group::A;

fn main() {
    let _ = A;
    let _ = B;
}
