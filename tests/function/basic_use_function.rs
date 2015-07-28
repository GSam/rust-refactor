mod bar {
    pub fn foo() {
    }
}

use bar::foo;

fn main() {
    foo();
}
