use std::fs::File;
use std::str;


fn decompile(filename : &'static str)
{
    let file = File::open(filename);

}

fn main() {
    decompile("simon.bin");
}
