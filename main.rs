pub mod TMS1000;
//pub use TMS1000::SYSTEM;


fn main() {
    let version : u32 = std::env::args().nth(1).expect("No version number specified").parse().expect("Version number must be an integer");
    let ROM_file = std::env::args().nth(2).expect("No ROM file given");
    let instruction_PLA = std::env::args().nth(3).expect("No instruction PLA given");
    let output_PLA = std::env::args().nth(4).expect("No output PLA given");


    let system = match TMS1000::SYSTEM::load_system(version, ROM_file, instruction_PLA, output_PLA) {
        Ok(v) => {println!("System loaded successfully");
            v
        },
        Err(e) => {println!("{}", e);
            return ();
            },
    };
}
