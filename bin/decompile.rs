#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused_parens)]

use tms::decompiler;

fn main() {
    let version : u32 = std::env::args().nth(1).expect("No version number specified").parse().expect("Version number must be an integer");
    let input_file = std::env::args().nth(2).expect("No input file given");
    if (version == 1100) || (version == 1300) {
        decompiler::display_TMS1100(input_file);
    }
    else {
        decompiler::display_TMS1000(input_file);
    }
}
