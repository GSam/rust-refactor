fn main() {
    let mut a = 2;
    {
        let mut a = 4;
        {
            let mut a = 8;
            a = 9;
        }
        a = 5;
    }
    a = 3;
}
