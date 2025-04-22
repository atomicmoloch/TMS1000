fn main() {

    let mut tst: u8 = 0b0000_0010;
    println!("{}", tst);
    println!("{:08b}", tst);
    tst = tst.reverse_bits();
    println!("{}", tst);
    println!("{:08b}", tst);
}
