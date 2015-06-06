mod group {
    pub struct A;
}

use group::A;

struct B;

fn main() {
    let _ = A;
    let _ = B;
}
