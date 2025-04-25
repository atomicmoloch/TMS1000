use tms::compiler;
use std::fs;

fn main() {
    let version : u32 = std::env::args().nth(1).expect("No version number specified").parse().expect("Version number must be an integer");
    let input_file = std::env::args().nth(2).expect("No input file given");
    let output_file = match std::env::args().nth(3) {
        Some(v) => v,
        None => format!("{}.out", input_file),
    };
    let data: String =  match fs::read_to_string(&input_file) {
        Ok(v) => v,
        Err(_) => panic!("Problem opening or reading input file"), //Nothing as permanant as a temporary solution
    };

    if (version == 1100) || (version == 1300) {
        match std::fs::write(output_file, compiler::compile_TMS1100(data)) {
            Ok(_) => println!("Success!"),
            _ => println!("An error occured"),
        }
    }
    else {
        match std::fs::write(output_file, compiler::compile_TMS1000(data)) {
            Ok(_) => println!("Success!"),
            _ => println!("An error occured"),
        }
    }

}
