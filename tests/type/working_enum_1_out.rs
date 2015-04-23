enum YesNo {
    Yes,
    No
}

impl YesNo {
    fn new() -> YesNo {
        YesNo::Yes
    }
}

fn main() {
    let choice = YesNo::new();

}
