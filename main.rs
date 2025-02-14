pub mod TMS1000;
//pub use TMS1000::SYSTEM;

fn input() -> String
{
    print!("> ");
    let mut inp = String::new();
    std::io::stdin().read_line(&mut inp).expect("Could not read from stdin");
    return inp;
}

fn get_k_inputs() -> u8 {
    println!("Enter k inputs as zeroes and ones");
    return input().parse().expect("k inputs must be an integer");

}


fn main() {
    let version : u32 = std::env::args().nth(1).expect("No version number specified").parse().expect("Version number must be an integer");
    let ROM_file = std::env::args().nth(2).expect("No ROM file given");
    let instruction_PLA = std::env::args().nth(3).expect("No instruction PLA given");
    let output_PLA = std::env::args().nth(4).expect("No output PLA given");


    let mut system = match TMS1000::SYSTEM::load_system(version, ROM_file, instruction_PLA, output_PLA) {
        Ok(v) => {println!("System loaded successfully");
            v
        },
        Err(e) => {println!("{}", e);
            return ();
            },
    };

    let mut command : String = "".to_string();
    let mut k_inputs : u8 = 0;

    while !(command == "quit") {
        println!("K inputs: {}", k_inputs);
        println!("R outputs: {:?}", system.get_r_outputs());
        println!("O outputs: {}", system.get_o_outputs());
        command = input();
        match command.as_str() {
            "step" => system = system.STEP(k_inputs),
            "cycle" => system = system.instruction_cycle(k_inputs),
            "setk" => k_inputs = get_k_inputs(),
            _ => (),
        }
    }
}
