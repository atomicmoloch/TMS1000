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


fn compile_instruction_TMS1000(instruction : String, operand : u8) -> u8 {
   return match instruction.as_str() {
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
        "LDP" => 0x10 + reversebits_u4(operand),
        "TAMIY" => 0x20,
        "TMA" => 0x21,
        "TMY" => 0x22,
        "TYA" => 0x23,
        "TAY" => 0x24,
        "AMAAC" => 0x25,
        "MNEZ" => 0x25,
        "SAMAN" => 0x27,
        "IMAC" => 0x28,
        "ALEM" => 0x29,
        "DMAN" => 0x2A,
        "IYC" => 0x2B,
        "DYN" => 0x2C,
        "CPAIZ" => 0x2D,
        "XMA" => 0x2E,
        "CLA" => 0x2F,
        "SBIT" => 0x30 + reversebits_u2(operand),
        "RBIT" => 0x34 + reversebits_u2(operand),
        "TBIT1" => 0x38 + reversebits_u2(operand),
        "LDX" => 0x3C + reversebits_u2(operand),
        "TCY" => 0x40 + reversebits_u4(operand),
        "YNEC" => 0x50 + reversebits_u4(operand),
        "TCMIY" => 0x60 + reversebits_u4(operand),
        "ALEC" => 0x70 + reversebits_u4(operand),
        "BR" => 0x80 + operand,
        "CALL" => 0xC0 + operand,
        _ => 255
    }

}

fn compile_instruction_TMS1100(instruction : String, operand : u8) -> u8 {
     return match instruction.as_str() {
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
        "LDP" => 0x10 + reversebits_u4(operand),
        "TAY" => 0x20,
        "TMA" => 0x21,
        "TMY" => 0x22,
        "TYA" => 0x23,
        "TAMDYN" => 0x24,
        "TAMIYC" => 0x25,
        "TAMZA" => 0x26,
        "TAM" => 0x27,
        "LDX" => 0x28 + reversebits_u3(operand),
        "SBIT" => 0x30 + reversebits_u2(operand),
        "RBIT" => 0x34 + reversebits_u2(operand),
        "TBIT1" => 0x38 + reversebits_u2(operand),
        "SAMAN" => 0x3C,
        "CPAIZ" => 0x3D,
        "IMAC" => 0x3E,
        "MNEZ" => 0x3F,
        "TCY" => 0x40 + reversebits_u4(operand),
        "YNEC" => 0x50 + reversebits_u4(operand),
        "TCMIY"=> 0x60 + reversebits_u4(operand),
        "CLA" => 0x7F,
        "BR" => 0x80 + operand,
        "CALL" => 0xC0 + operand,
        _ => {
            let add_const_re = Regex::new(r"^A([0-9]{1,2})AAC$").unwrap();
            if let Some(v) = add_const_re.captures(instruction.as_str()) {
                if let Some(op_str) = v.get(1) {
                    if let Ok(op) = op_str.as_str().parse::<u8>() {
                        return 0x70 + op
                    };}}
            255
        }
    }
}

pub fn compile_TMS1000(input : String) {
    let asm_regex = Regex::new(r"([0-9]) ([0-9]{2}) ([0-9]{2}) : ([0-9A-Z]{1,8})( [0-9]*)?").unwrap();
    for line in asm_regex.captures_iter(&input) {

    }

}

pub fn compile_TMS1100(input: String) {


}
