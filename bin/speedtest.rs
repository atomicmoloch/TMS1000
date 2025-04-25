#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused_parens)]

use std::cmp;

//pub mod TMS1000;
use tms::TMS1000;
use std::io::Write;
use std::time::SystemTime;


fn speedtest_300k(mut system : TMS1000::SYSTEM, k_inp : u8) {
    system.set_logging();
    let start = SystemTime::now();
    for _ in 0..300000 {
        system.instruction_cycle_mut(k_inp);
    }
    let end = SystemTime::now();
    let duration = end.duration_since(start).unwrap();
    println!("Executed 300,000 instructions in {} milliseconds", duration.as_millis());
    println!("{} microseconds per instruction", duration.as_micros() / 300000);
    println!("{:?}", system.get_o_outputs());
}

fn speedtest_500k(mut system : TMS1000::SYSTEM, k_inp : u8) {
    system.set_logging();
    let start = SystemTime::now();
    for _ in 0..500000 {
        system.instruction_cycle_mut(k_inp);
    }
    let end = SystemTime::now();
    let duration = end.duration_since(start).unwrap();
    println!("Executed 500000 instructions in {} milliseconds", duration.as_millis());
    println!("{} microseconds per instruction", duration.as_micros() / 500000);
    println!("{:?}", system.get_o_outputs());
}

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
    println!("K-inputs 0, 300000 instructions (1 second @ 300 khz)");
    speedtest_300k(system.clone(), 0);
    println!("K-inputs 1111, 300000 instructions (1 second @ 300 khz)");
    speedtest_300k(system.clone(), 15);
    println!("K-inputs 0, 500000 instructions (1 second @ 500 khz, max speed)");
    speedtest_500k(system.clone(), 0);
    println!("K-inputs 1111, 500000 instructions (1 second @ 500 khz)");
    speedtest_500k(system.clone(), 15);

}
