use regex::Regex;
use std::fs;

fn reversebits_u4(value : u8) -> u8 {
    return value.reverse_bits() >> 4;
}

fn reversebits_u3(value : u8) -> u8 {
    return value.reverse_bits() >> 5;
}

fn reversebits_u2(value : u8) -> u8 {
    return value.reverse_bits() >> 6;
}


fn compile_instruction_TMS1000(instruction : String, operand : u8) {
  /*  return match instruction {
        "COMX" => 0x00,
        "A8AAC" => 0x01,
        "YNEA" => 0x02,
        "TAM" => 0x03,
        "TAMZA" => 0x04,
        "A10AAC" => 0x05,
        "A6AAC" => 0x06,
        "DAN" => 0x07,
        "TKA" => 0x08,
        "KNEZ" => 0x09,
        "TDO" => 0x0A,
        "CLO" => 0x0B,
        "RSTR" => 0x0C,
        "SETR" => 0x0D,
        "IA" => 0x0E,
        "RETN" => 0x0F,
        "LDP"--
        "TAMIY" => 0x20,
        "TMA" => 0x21,
        "TMY" => 0x22,
        "TYA" => 0x23,
        "TAY" => 0x24
        "AMAAC" => 0x25,
        "MNEZ" => 0x25,
        "SAMAN" => 0x27
        "IMAC" => 0x28,
        "ALEM" => 0x29,
        "DMAN" => 0x2A,
        "IYC" => 0x2B,
        "DYN" => 0x2C,
        "CPAIZ" => 0x2D,
        "XMA" => 0x2E,
        "CLA" => 0x2F,
        "SBIT"--
        "RBIT"--
        "TBIT1"--
        "LDX"--
        "TCY"--
        "YNEC"--
        "TCMIY"--
        "ALEC"--
        "BR"--
        "CALL"--
    } */

}

fn compile_instruction_TMS1100(instruction : String, operand : u8) {
  /*  return match instruction {
        "MNEA" => 0x00,
        "ALEM" => 0x01,
        "XMA" => 0x02,
        "DYN" => 0x03,
        "IYC" => 0x04,
        "AMAAC" => 0x05,
        "DMAN" => 0x06,
        "TKA" => 0x07,
        "COMX" => 0x08,
        "TDO" => 0x09,
        "COMC" => 0x0A,
        "RSTR" => 0x0B,
        "SETR" => 0x0C,
        "KNEZ" => 0x0E,
        "RETN" => 0x0F,
        "LDP"--
        "TAY" => 0x20,
        "TMA" => 0x21,
        "TMY" => 0x22,
        "TYA" => 0x23,
        "TAMDYN" => 0x24,
        "TAMIYC" => 0x25,
        "TAMZA" => 0x26,
        "TAM" => 0x27,
        "LDX"--
        "SBIT"--
        "RBIT"--
        "TBIT1"--
        "SAMAN" => 0x3C,
        "CPAIZ" => 0x3D,
        "IMAC" => 0x3E,
        "MNEZ" => 0x3F,
        "TCY"--
        "YNEC"--
        "TCMIY"--
        "A{}AAC"--
        "CLA"
        "BR"--
        "CALL"--
    } */
}

fn compile_TMS1000(input : String) {
    let asm_regex = Regex::new(r"([0-9]) ([0-9]{2}) ([0-9]{2}) : ([0-9A-Z]{1,8})( [0-9]*)?").unwrap();
    for line in asm_regex.captures_iter(&input) {
        println!("{:?}", line);
    }
 //   println!("{}", input);
}

fn compile_TMS1100(input: String) {


}

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
        compile_TMS1100(data);
    }
    else {
        compile_TMS1000(data);
    }
  //  let src = decompile("simon.bin");
  //  let src = decompile("mp3300.bin");

}
