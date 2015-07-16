fn main() {
    let mut a = 2;
    {
        let mut b = 4;
        {
            let mut a = 8;
            a = 9;
        }
        b = 5;
    }
    a = 3;
}
