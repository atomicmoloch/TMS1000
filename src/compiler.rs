#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused_parens)]
#![allow(non_upper_case_globals)]

use regex::Regex;

fn reversebits_u4(value : u8) -> u8 {
    return value.reverse_bits() >> 4;
}

fn reversebits_u3(value : u8) -> u8 {
    return value.reverse_bits() >> 5;
}

fn reversebits_u2(value : u8) -> u8 {
    return value.reverse_bits() >> 6;
}


pub fn compile_instruction_TMS1000(instruction : String, operand : u8) -> Option<u8> {
   return match instruction.as_str() {
        "COMX" => Some(0x00),
        "A8AAC" => Some(0x01),
        "YNEA" => Some(0x02),
        "TAM" => Some(0x03),
        "TAMZA" => Some(0x04),
        "A10AAC" => Some(0x05),
        "A6AAC" => Some(0x06),
        "DAN" => Some(0x07),
        "TKA" => Some(0x08),
        "KNEZ" => Some(0x09),
        "TDO" => Some(0x0A),
        "CLO" => Some(0x0B),
        "RSTR" => Some(0x0C),
        "SETR" => Some(0x0D),
        "IA" => Some(0x0E),
        "RETN" => Some(0x0F),
        "LDP" => Some(0x10 + reversebits_u4(operand)),
        "TAMIY" => Some(0x20),
        "TMA" => Some(0x21),
        "TMY" => Some(0x22),
        "TYA" => Some(0x23),
        "TAY" => Some(0x24),
        "AMAAC" => Some(0x25),
        "MNEZ" => Some(0x26),
        "SAMAN" => Some(0x27),
        "IMAC" => Some(0x28),
        "ALEM" => Some(0x29),
        "DMAN" => Some(0x2A),
        "IYC" => Some(0x2B),
        "DYN" => Some(0x2C),
        "CPAIZ" => Some(0x2D),
        "XMA" => Some(0x2E),
        "CLA" => Some(0x2F),
        "SBIT" => Some(0x30 + reversebits_u2(operand)),
        "RBIT" => Some(0x34 + reversebits_u2(operand)),
        "TBIT1" => Some(0x38 + reversebits_u2(operand)),
        "LDX" => Some(0x3C + reversebits_u2(operand)),
        "TCY" => Some(0x40 + reversebits_u4(operand)),
        "YNEC" => Some(0x50 + reversebits_u4(operand)),
        "TCMIY" => Some(0x60 + reversebits_u4(operand)),
        "ALEC" => Some(0x70 + reversebits_u4(operand)),
        "BR" => Some(0x80 + operand),
        "CALL" => Some(0xC0 + operand),
        _ => None,
    }

}

pub fn compile_instruction_TMS1100(instruction : String, operand : u8) -> Option<u8> {
      return match instruction.as_str() {
        "MNEA" => Some(0x00),
        "ALEM" => Some(0x01),
        "YNEA" => Some(0x02),
        "XMA" => Some(0x03),
        "DYN" => Some(0x04),
        "IYC" => Some(0x05),
        "AMAAC" => Some(0x06),
        "DMAN" => Some(0x07),
        "TKA" => Some(0x08),
        "COMX" => Some(0x09),
        "TDO" => Some(0x0A),
        "COMC" => Some(0x0B),
        "RSTR" => Some(0x0C),
        "SETR" => Some(0x0D),
        "KNEZ" => Some(0x0E),
        "RETN" => Some(0x0F),
        "LDP" => Some(0x10 + reversebits_u4(operand)),
        "TAY" => Some(0x20),
        "TMA" => Some(0x21),
        "TMY" => Some(0x22),
        "TYA" => Some(0x23),
        "TAMDYN" => Some(0x24),
        "TAMIYC" => Some(0x25),
        "TAMZA" => Some(0x26),
        "TAM" => Some(0x27),
        "LDX" => Some(0x28 + reversebits_u3(operand)),
        "SBIT" => Some(0x30 + reversebits_u2(operand)),
        "RBIT" => Some(0x34 + reversebits_u2(operand)),
        "TBIT1" => Some(0x38 + reversebits_u2(operand)),
        "SAMAN" => Some(0x3C),
        "CPAIZ" => Some(0x3D),
        "IMAC" => Some(0x3E),
        "MNEZ" => Some(0x3F),
        "TCY" => Some(0x40 + reversebits_u4(operand)),
        "YNEC" => Some(0x50 + reversebits_u4(operand)),
        "TCMIY"=> Some(0x60 + reversebits_u4(operand)),
        "CLA" => Some(0x7F),
        "BR" => Some(0x80 + operand),
        "CALL" => Some(0xC0 + operand),
        _ => {
            let add_const_re = Regex::new(r"^A([0-9]{1,2})AAC$").unwrap();
            if let Some(v) = add_const_re.captures(instruction.as_str()) {
                if let Some(op_str) = v.get(1) {
                    if let Ok(op) = op_str.as_str().parse::<u8>() {
                        return Some(0x70 + reversebits_u4(op - 1))
                    };}}
            None
        }
    }
}

pub fn compile_TMS1000(input : String) -> [u8; 64 * 16] {
    let mut results: [u8; 64 * 16] = [0; 1024];
    let asm_regex = Regex::new(r"([0-9]{1,2}) ([0-9]{1,2}) : ([0-9A-Z]{1,8})( [0-9]{1,2})?").unwrap();
    for line in asm_regex.captures_iter(&input) {
        let error: String = format!("Compiler: Error in line {:?}", line);
        let page: usize = usize::from_str_radix(line[1].as_ref(), 10).expect(&error);
        let word: usize = usize::from_str_radix(line[2].as_ref(), 10).expect(&error);
        let instruction : String = line[3].to_string();
        let operand: u8 = match line.get(4).is_none() {
            false => u8::from_str_radix(line[4].trim().as_ref(), 10).unwrap_or(0),
            _ => 0,
        };
        println!("{} {}", instruction, operand);
        results[(page * 64) + word] = compile_instruction_TMS1000(instruction, operand).expect(&error);
    }
    return results;
}

pub fn compile_TMS1100(input: String) -> [u8; 64 * 16 * 2] {
    let mut results: [u8; 64 * 16 * 2] = [0; 2048];
    let asm_regex = Regex::new(r"([0-9]) ([0-9]{1,2}) ([0-9]{1,2}) : ([0-9A-Z]{1,8})( [0-9]{1,2})?").unwrap();
    for line in asm_regex.captures_iter(&input) {
        let error: String = format!("Compiler: Error in line {:?}", line);
        let chapter: usize = usize::from_str_radix(line[1].as_ref(), 10).expect(&error);
        let page: usize = usize::from_str_radix(line[2].as_ref(), 10).expect(&error);
        let word: usize = usize::from_str_radix(line[3].as_ref(), 10).expect(&error);
        let instruction : String= line[4].to_string();
        let operand: u8 = match line.get(5).is_none() {
            false => u8::from_str_radix(line[5].trim().as_ref(), 10).unwrap_or(0),
            _ => 0,
        };
        println!("{} {}", instruction, operand);
        results[(chapter * 1024) + (page * 64) + word] = compile_instruction_TMS1100(instruction, operand).expect(&error);
    }
    return results;
}
