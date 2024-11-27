use std::fs::File;
use std::io::Read;
use std::str;


fn decompile(filename : &'static str)
{
    let file = File::open(filename);
    let mut data = vec![];
    let _ = file.expect("REASON").read_to_end(&mut data);
    println!("{:?}", data);
}

fn main() {
    decompile("simon.bin");
}
