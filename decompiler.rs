use std::fs::File;
use std::io::Read;
use std::str;

const PC_SEQ: [u8; 64] = [0x00, 0x01, 0x03, 0x07, 0x0F, 0x1F, 0x3F, 0x3E, 0x3D, 0x3B, 0x37, 0x2F, 0x1E, 0x3C, 0x39, 0x33, 0x27, 0x0E, 0x1D, 0x3A, 0x35, 0x2B, 0x16, 0x2C, 0x18, 0x30, 0x21, 0x02, 0x05, 0x0B, 0x17, 0x2E, 0x1C, 0x38, 0x31, 0x23, 0x06, 0x0D, 0x1B, 0x36, 0x2D, 0x1A, 0x34, 0x29, 0x12, 0x24, 0x08, 0x11, 0x22, 0x04, 0x09, 0x13, 0x26, 0x0C, 0x19, 0x32, 0x25, 0x0A, 0x15, 0x2A, 0x14, 0x28, 0x10, 0x20];


//limited implementation of u4 function for reversing bits on instructions with MSB on the right
//default: MSB on left
fn reversebits_u4(value : u8) -> u8 {
    return value.reverse_bits() >> 4;
}

fn reversebits_u2(value : u8) -> u8 {
    return value.reverse_bits() >> 2;
}


fn decodeinstruction(instruction : u8) -> String{
    match instruction {
        0x00 => return String::from("COMX"),
        0x01 => return String::from("A8AAC"),
        0x02 => return String::from("YNEA"),
        0x03 => return String::from("TAM"),
        0x04 => return String::from("TAMZA"),
        0x05 => return String::from("A10AAC"),
        0x06 => return String::from("A6AAC"),
        0x07 => return String::from("DAN"),
        0x08 => return String::from("TKA"),
        0x09 => return String::from("KNEZ"),
        0x0A => return String::from("TDO"),
        0x0B => return String::from("CLO"),
        0x0C => return String::from("RSTR"),
        0x0D => return String::from("SETR"),
        0x0E => return String::from("IA"),
        0x0F => return String::from("RETN"),
        0x10..=0x1F => return String::from("LDP ".to_owned() + &(reversebits_u4(instruction)).to_string()),
        0x20 => return String::from("TAMIY"),
        0x21 => return String::from("TMA"),
        0x22 => return String::from("TMY"),
        0x23 => return String::from("TYA"),
        0x24 => return String::from("TAY"),
        0x25 => return String::from("AMAAC"),
        0x26 => return String::from("MNEZ"),
        0x27 => return String::from("SAMAN"),
        0x28 => return String::from("IMAC"),
        0x29 => return String::from("ALEM"),
        0x2A => return String::from("DMAN"),
        0x2B => return String::from("IYC"),
        0x2C => return String::from("DYN"),
        0x2D => return String::from("CPAIZ"),
        0x2E => return String::from("XMA"),
        0x2F => return String::from("CLA"),
        0x30..=0x33 => return String::from("SBIT ".to_owned() + &(reversebits_u2(instruction)).to_string()),
        0x34..=0x37 => return String::from("RBIT ".to_owned() + &(reversebits_u2(instruction)).to_string()),
        0x38..=0x3A => return String::from("TBIT 1 ".to_owned() + &(reversebits_u2(instruction)).to_string()),
        0x3B..=0x3F => return String::from("LDX ".to_owned() + &(reversebits_u2(instruction)).to_string()),
        0x40..=0x4F => return String::from("TCY ".to_owned() + &(reversebits_u4(instruction)).to_string()),
        0x50..=0x5F => return String::from("YNEC ".to_owned() + &(reversebits_u4(instruction)).to_string()),
        0x60..=0x6F => return String::from("TCMIY ".to_owned() + &(reversebits_u4(instruction)).to_string()),
        0x70..=0x7F => return String::from("ALEC ".to_owned() + &(reversebits_u4(instruction)).to_string()),
        0x80..=0xBF => return String::from("BR ".to_owned() + &(instruction % 64).to_string() + " (" + &(PC_SEQ.iter().position(|&i| i == (instruction % 64)).unwrap()).to_string() + ")"),
        0xC0..=0xFF => return String::from("CALL ".to_owned() + &(instruction % 64).to_string() + " (" + &(PC_SEQ.iter().position(|&i| i == (instruction % 64)).unwrap()).to_string() + ")"),
      //  _ => return instruction.to_string(),
    }
}

fn decompile(filename : &'static str) -> [String; 64 * 16]
{
    let file = File::open(filename);
    let mut data: Vec<u8> = vec![];
    let _ = file.expect("REASON").read_to_end(&mut data);
    let mut pcvalue: u8 = 0;
    let mut pavalue: u8 = 0;
   // println!("{:?}", data);
    let mut results: [String; 64 * 16] = [const {String::new()}; 64 * 16];
    for i in data.iter_mut() {
        //Reorders instructions in order of execution
        //(TMS1000 uses a pseudo-random program counter order, seen in PC_SEQ)
        let execorder = PC_SEQ.iter().position(|&i| i == pcvalue).unwrap();
        results[64 * <u8 as Into<usize>>::into(pavalue) + execorder] = pavalue.to_string() + " : " + &pcvalue.to_string() + " " + &decodeinstruction(i.clone());
        pcvalue += 1;
        if pcvalue == 64 {
            pcvalue = pcvalue % 64;
            pavalue += 1;
        }
    }
    return results;
}

fn main() {
  //  let src = decompile("simon.bin");
    let src = decompile("mp3300.bin");
    for (idx, val) in src.iter().enumerate() {
        println!("{idx} - {val}");
    }
}
