use YesNo::{Yes, No};

enum YesNo {
    Yes,
    No
}

impl YesNo {
    fn new() -> YesNo {
        Yes
    }
}

fn main() {
    let choice = YesNo::new();

}
