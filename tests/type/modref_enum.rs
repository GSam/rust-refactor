use Choice::{Yes, No};

enum Choice {
    Yes,
    No
}

impl Choice {
    fn new() -> Choice {
        Yes
    }
}

fn main() {
    let choice = Choice::new();

}
