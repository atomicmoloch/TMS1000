#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused_parens)]

use std::cmp;

//pub mod TMS1000;
use tms::TMS1000;
use tms::decompiler;
use regex::Regex;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;

fn input() -> String
{
    let mut inp = String::new();
    std::io::stdin().read_line(&mut inp).expect("Could not read from stdin");
    return inp.to_lowercase();
}


fn get_bin_input(max : u32) -> u32 {
    println!("Enter a {}-bit binary number", max);
    let maxval: u32 = 2_u32.pow(max);
    loop {
        match u32::from_str_radix(input().trim(), 2) {
            Ok(num) => {
                println!("You inputted {:b} ({})", num, num);
                if (num < maxval) {
                    return num;
                }
                else {
                    println!("Number greater than {:b} ({})", maxval, maxval);
                }
            },
            Err(_) => println!("Invalid number"),
        }
    }
}

//This entire file was originally just a stopgap to test the TMS1000 emulator
//Hence the very basic user interface and argument parsing
//"There's nothing so permanant as a temporary solution"
fn main() {
    let version : u32 = std::env::args().nth(1).expect("No version number specified").parse().expect("Version number must be an integer");
    let ROM_file = std::env::args().nth(2).expect("No ROM file given");
    let instruction_PLA = std::env::args().nth(3).expect("No instruction PLA given");
    let output_PLA = std::env::args().nth(4).expect("No output PLA given");
    let decompiled_code = decompiler::decompile(ROM_file.clone(), version);

    let mut system = match TMS1000::SYSTEM::load_system(version, ROM_file, instruction_PLA, output_PLA) {
        Ok(v) => {println!("System loaded successfully");
            v
        },
        Err(e) => {println!("{}", e);
            return ();
            },
    };

    let mut prevcommand : String;
    let mut command : String = String::with_capacity(15);
    let mut k_inputs : u8 = 0;
    let mut break_on_alert : bool = true;
    let mut halt : bool = false; //halts program execution

    let breakregex = Regex::new(r"ALERT").unwrap();
    let mut otriggers: HashMap<u32, u8> = HashMap::new();
    let mut rtriggers: HashMap<u32, u8> = HashMap::new();

    let mut o_trigger_on: bool = false;
    let mut r_trigger_on: bool = false;

    let mut log_file = None;
    let mut logout: bool = false;

    let mut auto_run = 0;

    while !(command == "quit\n") {

        k_inputs = match rtriggers.get(&system.get_r_outputs_u32()) {
            Some(v) => {
                r_trigger_on = true;
                v.clone()},
            _ => {
                if r_trigger_on {
                    r_trigger_on = false;
                    0
                }
                else {
                    k_inputs
                }
            }
        };

        k_inputs = match otriggers.get(&system.get_o_outputs()) {
            Some(v) => {
                o_trigger_on = true;
                v.clone()},
            _ => {
                if o_trigger_on {
                    o_trigger_on = false;
                    0
                }
                else {
                    k_inputs
                }
            }
        };
        println!("\n");
        println!("K inputs: {:0>4b}", k_inputs);
        println!("R outputs: {:0>16b}", system.get_r_outputs_u32());
        println!("O outputs: {:0>10b}", system.get_o_outputs());
        println!("{}", format!("Next Instruction: {}", decompiled_code[system.get_rom_index()]));

        prevcommand = command;
      //  command = String::with_capacity(15); //probably slightly better performance than cloning command into prevcommand

        if auto_run > 0 {
            auto_run -= 1;
            command = "cycle\n".into();
        }
        else {
            command = input();
        }
        if command.as_str() == "\n" {
            command = prevcommand;
        }
        println!("\n");
        match command.as_str() {
            "step\n" | "s\n" => {
                if !halt {
                    system.STEP_mut(k_inputs);
                    println!("One step executed");

                }
                else {
                    println!("SYSTEM HALTED");
                }
                },
            "cycle\n" | "c\n" => {
                if !halt {
                    system.instruction_cycle_mut(k_inputs);
                    println!("One instruction cycle executed");
                }
                else {
                    println!("SYSTEM HALTED");
                }
            },
            "setk\n" | "sk\n" => k_inputs = get_bin_input(4) as u8,
            "seenext\n" | "next\n" | "n\n" => {
                let end: usize = cmp::min(system.get_rom_index() + 10, decompiled_code.len() - 1);
                for line in decompiled_code[system.get_rom_index()..end].iter() {
                    println!("{}", line);
                }
            },
            "setbreak\n" | "setb\n" | "sb\n" => {
                break_on_alert = !break_on_alert;
                println!("{}", break_on_alert);},
            "sethalt\n" | "seth\n" | "sn\n" => {
                halt = !halt;
                println!("{}", halt);},
            "printram\n" | "printr\n" | "pr\n" => println!("{:?}\n", system.get_ram_array()),
            "clearotriggers\n" | "clearotrigger\n" | "clot\n" | "cot\n" => otriggers = HashMap::new(),
            "clearrtriggers\n" | "clearrtrigger\n" | "clrt\n" | "crt\n" => rtriggers = HashMap::new(),
            "setotrigger\n" | "setot\n" | "sot\n" => {
                println!("Enter trigger");
                let trig: u32= get_bin_input(10);
                println!("Enter value");
                let val: u8 = get_bin_input(4) as u8;
                otriggers.insert(trig, val);
            },
            "setrtrigger\n" | "setrt\n" | "srt\n" => {
                println!("Enter trigger");
                let trig: u32 = get_bin_input(16);
                println!("Enter value");
                let val: u8 = get_bin_input(4) as u8;
                rtriggers.insert(trig, val);
            },
            "settings\n" | "printsettings\n" | "ps\n" =>
                println!("Break on alert: {}\nHalt status: {}\nSaving log to file: {}\nR triggers: {:?}\nO triggers: {:?}\n", break_on_alert, halt, logout, rtriggers, otriggers),
            "registers\n" | "printregisters\n" | "pn\n" => println!("X register: {}\nY register: {}\nProgram Counter: {}\nSubroutine Register: {}\nPage Address: {}\nPage Buffer: {}\nCall Latch: {}\nChapter Address: {}\nChapter Buffer: {}\nChapter Subroutine Latch: {}\nAccumulator: {}\nStatus: {}\nStatus Latch: {}\n" , system.get_x_reg(), system.get_y_reg(), system.get_pc_reg(), system.get_sr_reg(), system.get_pa_reg(), system.get_pb_reg(), system.get_cl_reg(), system.get_ca_reg(), system.get_cb_reg(), system.get_csl_reg(), system.get_acc_reg(), system.get_status(), system.get_sl_reg()),
            "setlog\n" | "logfile\n" | "logout\n" | "lo\n" => {
                if logout {
                    logout = false;
                    println!("Disabled logging");
                }
                else {
                    println!("Enter file to enter logs in");
                    match OpenOptions::new().write(true).append(true).create(true).open(input().trim()) {
                        Ok(f) => {
                            logout = true;
                            log_file = Some(f);
                            println!("File opened successfully");
                        },
                        Err(_) => println!("File open unsuccessful"),
                    }
                }
            },
            "init\n" | "initialize\n" | "reinitialize\n" => system.INITIALIZE(),
            "quit\n" | "q\n" => {println!("Goodbye");
                command = "quit\n".into();},
            "auto100\n" | "a100\n" => {
                auto_run = 100;
                command = "cycle".into();}
            "auto1000\n" | "a1000\n" => {
                auto_run = 1000;
                command = "cycle\n".into();}
            "auto10000\n" | "a10000\n" => {
                auto_run = 10000;
                command = "cycle".into();}
            "auto100000\n" | "a100000\n" => {
                auto_run = 100000;
                command = "cycle".into();}
            "auto1000000\n" | "a1000000\n" => {
                auto_run = 1000000;
                command = "cycle".into();}
            "auto10000000\n" | "a10000000\n" => {
                auto_run = 1000000;
                command = "cycle".into();}
            _ => println!("Could not interpret command\nValid commands are: step, s, cycle, c, setk, sk, seenext, next, sn, setbreak, setb, sb, sethalt, seth, sh, printram, printr, pr, clearotriggers, clearotrigger, clot, cot, clearrtriggers, clearrtrigger, clrt, crt, setotrigger, setot, sot, setrtrigger, setrt, srt, settings, printsettings, ps, registers, printregisters, pn, setlog, logfile, logout, lo, reinitialize, initialize, init, quit, q, auto100, a100, auto1000, a1000, auto10000, a10000, auto100000, a100000, auto1000000, a1000000, auto10000000, a10000000"),
        }
        let log = system.get_log();
        for entry in log.iter() {
            println!("{}", entry);
            if breakregex.is_match(entry) && break_on_alert {
                halt = true;
                auto_run = 0;
            }
            if logout {
                let _ = log_file.as_mut().unwrap().write_all(format!("{}\n", entry).as_bytes());
                //Errors ignored
            }
        }
        if logout {
            let _ = log_file.as_mut().unwrap().write_all("\n".as_bytes());
        }
    }
}
