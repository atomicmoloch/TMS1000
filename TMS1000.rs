#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
//using all caps to denote actual system variables
//and camelcase to denote handler elements

use std::fs;
use std::io::Read;
use std::collections::HashMap;
use regex::Regex;



//HELPER FUNCTIONS
//imitates smaller than u8
fn u1(value : u8) -> u8 {
    return value % 1;
}

fn u2(value : u8) -> u8 {
    return value % 4;
}

fn u2_usize(value : usize) -> usize {
    return value % 4;
}

fn u3_usize(value : usize) -> usize {
    return value % 8;
}

fn u4(value : u8) -> u8 {
    return value % 16;
}

fn u5(value : u8) -> u8 {
    return value % 32;
}

fn u5_u32(value : u32) -> u32 {
    return value % 32;
}

fn u6(value : u8) -> u8 {
    return value % 64;
}

fn u6_usize(value : usize) -> usize {
    return value % 64;
}

fn reversebits_u4(value : u8) -> u8 {
    return value.reverse_bits() >> 4;
}

fn reversebits_u3(value : u8) -> u8 { //only used for tms1100/1300 RAM addressing
    return value.reverse_bits() >> 5;
}

fn reversebits_u2(value : u8) -> u8 {
    return value.reverse_bits() >> 6;
}

static PC_SEQ: [u8; 64] = [0x00, 0x01, 0x03, 0x07, 0x0F, 0x1F, 0x3F, 0x3E, 0x3D, 0x3B, 0x37, 0x2F, 0x1E, 0x3C, 0x39, 0x33, 0x27, 0x0E, 0x1D, 0x3A, 0x35, 0x2B, 0x16, 0x2C, 0x18, 0x30, 0x21, 0x02, 0x05, 0x0B, 0x17, 0x2E, 0x1C, 0x38, 0x31, 0x23, 0x06, 0x0D, 0x1B, 0x36, 0x2D, 0x1A, 0x34, 0x29, 0x12, 0x24, 0x08, 0x11, 0x22, 0x04, 0x09, 0x13, 0x26, 0x0C, 0x19, 0x32, 0x25, 0x0A, 0x15, 0x2A, 0x14, 0x28, 0x10, 0x20];

//Main body

#[derive(Clone)]
struct SYSTEM_STATE {
    INSTRUCTION: u8, //u8 current instruction
    INSTRUCTION_DECODED: u32,
    STEP: usize,

    X_REGISTER: usize, //U2 X, storage register; ram page address
    Y_REGISTER: usize, //U4 Y, storage register; ram word address and R output address

    PROGRAM_COUNTER: u8, //u6 PC, shift register
    PC_INDEX: usize, //u6, index for pseudo-random program counter
    SUBROUTINE_RETURN: u8, //u6 SR, storage register

    PAGE_ADDRESS: u8, //u4 PA, storage register; contains 4-bit page address of rom instructions
    PAGE_BUFFER: u8, //U4 PB storage register, used to set up page changes. also contains 4-bit return page address during call state
    CALL_LATCH: u8, //u1, CL, latch, stores call state

    //Chapter addressing for TMS1100/1300
    //All are U1
    CHAPTER_ADDRESS: usize, //Stores current chapter data
    CHAPTER_BUFFER: usize, //Stores succeeding chapter data and transfers to CA pending successful execution of a subsequent branch or call instruction
    CHAPTER_SUBROUTINE_LATCH: usize, //Stores return address after successfully executing call instruction

    //RAM array
    //eight files with 16 * U4 each
    //Only 4 files are used on non TMS1100/1300 devices
    RAM_ARRAY: [[u8; 16]; 8],

    P_MUX_LOGIC: u8, //u4, P-MUX: Data multiplexxer. Selects input to adder from Y register, CKI logic, or RAM array (0, 1, or 2)
    N_MUX_LOGIC: u8,//u5, N-MUX: Data multiplexxer. Selects N input to adder (0) RAM, (1) CKI, (2) accumulator, (3) not-accumulator or (4) F16

    ACCUMULATOR: u8, //U4 A, storage register
    ADDER_INC: u8, //u1 - whether to increment the adder - set by C8 microinstruction and should be reset to 0 every cycle
    STATUS: u8, //1-bit S, gates. conditional branch control. Normal state - 1. Branches are taken if S = 1. Selectively outputs a zero when carry is false or when logical compare is true. A zero lasts for one instruction cycle only.
    STATUS_LIFETIME : u8, //Facilitates Status, as described above
    STATUS_LATCH: u8, //1-bit SL, latch, selectively stores status output. Transfers to O register w/ acc bits when TDO is executed

    //Outputs:
    R_OUTPUT: Vec<u8>, //R output register - single bit RAM cells, latches for output to R buffers. Used to control external devices, display scans, input encoding, status logic outputs. Can be strobed to scan a key matrix. Using u8 instead of bool here costs a little memory but maintains consistency with the rest of the conventions. May change later.
    O_OUTPUT: u32, //U5, O output register. Used to transmit data

    K_INPUT: u8, //K input registers, K1, K2, K4, and K8
    //In order to maintain persistence across instruction cycles, like an analog button press would, I thought of giving it a lifespan variable (like Status)
    //But decided that's more germane to the physical layer
}

impl SYSTEM_STATE {
    fn ToString(&self) -> String {
        "{self.STATE.X_REGISTER}\n{self.STATE.Y_REGISTER}\n{self.STATE.X_REGISTER}\n{self.STATE.PROGRAM_COUNTER}\n{self.STATE.PC_INDEX}\n{self.STATE.SUBROUTINE_RETURN}\n{self.STATE.PAGE_ADDRESS}\n{self.STATE.PAGE_BUFFER}\n{self.STATE.CALL_LATCH}\n{self.STATE.RAM_ARRAY}\n{self.STATE.CKI_VALUE}\n{self.STATE.P_MUX_LOGIC}\n{self.STATE.N_MUX_LOGIC}\n{self.STATE.ACCUMULATOR}\n{self.STATE.ADDER_INC}\n{self.STATE.STATUS}\n{self.STATE.STATUS_LATCH}\n{self.STATE.R_OUTPUT}\n{self.STATE.O_OUTPUT}\n{self.STATE.K_INPUT}".to_string()
    }

}

#[derive(Clone)]
pub struct SYSTEM {
    VERSION: u32,
    STATE: SYSTEM_STATE,
    ROM_ARRAY: Vec<u8>,
    INSTRUCTION_PLA: HashMap<u32, u32>,
    OUTPUT_PLA: HashMap<u32, u32>,
    //   PC_SEQ: [U6; 64], not sure if changable
}

impl SYSTEM {

//System components

    fn CKI(&mut self) -> u8 {
        //selects either constant field, k input to enter cki data bus, or bit mask
        match self.STATE.INSTRUCTION {
            0x00..=0x07 => return reversebits_u4(self.STATE.INSTRUCTION), //constant
            0x08..=0x0F => return self.STATE.K_INPUT,
            0x20..=0x2F => return 0,
            0x30..=0x3A => return 15 - reversebits_u2(self.STATE.INSTRUCTION), //bit mask
            0x40..=0x7F => return reversebits_u4(self.STATE.INSTRUCTION),
            _ => return 255, //does nothing on LDP, LDX, BR and CALL instructions
        }
    }

    fn P_MUX(&mut self) -> u8 {
    //(0) Y register, (1) CKI, (2) RAM page
        match self.STATE.N_MUX_LOGIC {
            0 => return self.STATE.Y_REGISTER as u8,
            1 => return self.CKI(),
            _ => return self.STATE.RAM_ARRAY[self.STATE.X_REGISTER][self.STATE.Y_REGISTER],
        }
    }

    fn N_MUX(&mut self) -> u8 {
    //(0) RAM, (1) CKI, (2) accumulator, (3) not-accumulator or (4) F16
        match self.STATE.P_MUX_LOGIC {
            0 => return self.STATE.RAM_ARRAY[self.STATE.X_REGISTER][self.STATE.Y_REGISTER],
            1 => return self.CKI(),
            2 => return self.STATE.ACCUMULATOR,
            3 => return u4(1 + !(self.STATE.ACCUMULATOR)),
            _ => return 15,
        }
    }

    fn ADDER(&mut self) -> (u8, u8) {
        let value = self.P_MUX() + self.N_MUX() + self.STATE.ADDER_INC;
        return (u1(value >> 4 & 1), u4(value));
    }

    fn INCREMENT_PC(&mut self) {
        self.STATE.PC_INDEX = u6_usize(self.STATE.PC_INDEX + 1);
        self.STATE.PROGRAM_COUNTER = PC_SEQ[self.STATE.PC_INDEX];
    }

    // fn DECREMENT_PC(&mut self) {
    //     if self.STATE.PC_INDEX == 0 {
    //         self.STATE.PC_INDEX = 64;
    //     }
    //     else {
    //         self.STATE.PC_INDEX = u6_usize(self.STATE.PC_INDEX - 1);
    //     }
    //     self.STATE.PROGRAM_COUNTER = PC_SEQ[self.STATE.PC_INDEX];
    // }

    fn SET_PC(&mut self, value : u8) {
        self.STATE.PROGRAM_COUNTER = value;
        self.STATE.PC_INDEX = PC_SEQ.iter().position(|&i| i == value).unwrap(); //this should be guarenteed; thus the use of unwrap()
    }

//Microinstructions

    //Branch on status = one
    fn BR (&mut self) {
        //On status: changes PC to br value and if call latch not active, moved PB to PA
        //If not status: increments PC and changes status to 1
        if (self.STATE.STATUS == 1) {
            if (self.STATE.CALL_LATCH == 0) {
                self.STATE.PAGE_ADDRESS = self.STATE.PAGE_BUFFER;
            }
            self.STATE.CHAPTER_ADDRESS = self.STATE.CHAPTER_BUFFER;
            self.SET_PC(u6(self.STATE.INSTRUCTION));
    //          self.DECREMENT_PC(); //Since Step() will increment it again
        }
        else {
            self.STATE.STATUS = 1;
        }
    }

    //Call subroutine on status = one
    fn CALL (&mut self) {
        if (self.STATE.STATUS == 1) {
            if (self.STATE.CALL_LATCH == 0) {
                self.STATE.SUBROUTINE_RETURN = PC_SEQ[self.STATE.PC_INDEX]; //removed the +1 for now, expecting Step to increment
                (self.STATE.PAGE_ADDRESS, self.STATE.PAGE_BUFFER) = (self.STATE.PAGE_BUFFER, self.STATE.PAGE_ADDRESS);
                self.STATE.CHAPTER_SUBROUTINE_LATCH = self.STATE.CHAPTER_ADDRESS;
                self.STATE.CHAPTER_ADDRESS = self.STATE.CHAPTER_BUFFER;
                self.STATE.CALL_LATCH = 1;
            }
            else {
                self.STATE.CHAPTER_ADDRESS = self.STATE.CHAPTER_BUFFER;
                self.STATE.PAGE_BUFFER = self.STATE.PAGE_ADDRESS;
            }
            self.SET_PC(u6(self.STATE.INSTRUCTION));
    //         self.DECREMENT_PC();
        }
        else {
            self.STATE.STATUS = 1;
        }
    }

    //Return from subroutine
    fn RETN(&mut self) {
        self.STATE.PAGE_ADDRESS = self.STATE.PAGE_BUFFER;
        if (self.STATE.CALL_LATCH == 1) {
            self.SET_PC(self.STATE.SUBROUTINE_RETURN);
            self.STATE.CHAPTER_ADDRESS = self.STATE.CHAPTER_SUBROUTINE_LATCH;
            self.STATE.CALL_LATCH = 0;
        }
        //Step will increment PC
    }

    //Load page buffer with constant
    fn LDP (&mut self) {
        self.STATE.PAGE_BUFFER = reversebits_u4(self.STATE.INSTRUCTION); //MSB on right
    }

    //Load X register with constant
    fn LDX_TMS1000(&mut self) {
        self.STATE.X_REGISTER = reversebits_u2(self.STATE.INSTRUCTION) as usize;
    }

    fn LDX_TMS1100(&mut self) {
        self.STATE.X_REGISTER = reversebits_u3(self.STATE.INSTRUCTION) as usize;
    }

    //Complement X
    fn COMX (&mut self) {
        //Should flip bits of X register (1s compliment)
        self.STATE.X_REGISTER = u2_usize(!self.STATE.X_REGISTER);
    }

    fn COMX_TMS1000 (&mut self) {
        //Changes MSB of X register
        self.STATE.X_REGISTER ^= 0b1 << 2;
    }

    //Transfer data from accumulator and status latch to O outputs
    fn TDO (&mut self) {
        //Acc and SL transferred to O-output register
        self.STATE.O_OUTPUT = u5_u32((self.STATE.ACCUMULATOR + (self.STATE.STATUS_LATCH << 4)).into());
    }

    //Clear O-output register
    fn CLO (&mut self) {
        //zeroes O-register
        self.STATE.O_OUTPUT = 0;
    }

    fn COMC (&mut self) {
        //Toggles chapter buffer
        self.STATE.CHAPTER_BUFFER = (self.STATE.CHAPTER_BUFFER + 1) % 1;
    }

    //Set R output addressed by Y
    fn SETR (&mut self) {
        //sets R(Y) to 1; if Y out of range, no-op
        if (self.STATE.Y_REGISTER < self.STATE.R_OUTPUT.len()) && (self.STATE.X_REGISTER < 4) {
            self.STATE.R_OUTPUT[self.STATE.Y_REGISTER] = 1;
        }
    }

    //Reset R output addressed by Y
    fn RSTR (&mut self) {
        //sets R(Y) to 0; if Y out of range, no-op
        if (self.STATE.Y_REGISTER < self.STATE.R_OUTPUT.len()) && (self.STATE.X_REGISTER < 4) {
            self.STATE.R_OUTPUT[self.STATE.Y_REGISTER] = 0;
        }
    }

    //Set memory bit
    fn SBIT (&mut self) {
        //sets BIT of RAM(X,Y) to 1
        let BIT_U8 = reversebits_u2(self.STATE.INSTRUCTION);
        let IS_SET = self.STATE.RAM_ARRAY[self.STATE.X_REGISTER][self.STATE.Y_REGISTER] & (1_u8 << BIT_U8) != 0;
        if !(IS_SET) {
            self.STATE.RAM_ARRAY[self.STATE.X_REGISTER][self.STATE.Y_REGISTER] = u4(self.STATE.RAM_ARRAY[self.STATE.X_REGISTER][self.STATE.Y_REGISTER] + (1_u8 << BIT_U8));
        }
    }

    //Reset memory bit
    fn RBIT (&mut self) {
        //sets BIT of RAM(X,Y) to 0
        let BIT_U8 = reversebits_u2(self.STATE.INSTRUCTION);
        let IS_SET = self.STATE.RAM_ARRAY[self.STATE.X_REGISTER][self.STATE.Y_REGISTER] & (1_u8 << BIT_U8) != 0;
        if (IS_SET) {
            self.STATE.RAM_ARRAY[self.STATE.X_REGISTER][self.STATE.Y_REGISTER] = u4(self.STATE.RAM_ARRAY[self.STATE.X_REGISTER][self.STATE.Y_REGISTER] - (1_u8 << BIT_U8));
        }
    }

    //P-MUX instructions

    //CKI to P-adder input
    fn CKP(&mut self) {
        self.STATE.P_MUX_LOGIC = 1;
    }

    //Y-register to P-adder input
    fn YTP(&mut self) {
        self.STATE.P_MUX_LOGIC = 0;
    }

    //Memory (X, Y) to N-adder input
    fn MTP(&mut self) {
        self.STATE.P_MUX_LOGIC = 2;
    }

    //N-MUX instructions

    //Accumulator to N-adder input
    fn ATN(&mut self) {
        self.STATE.N_MUX_LOGIC = 2;
    }

    //not-accumulator to N-adder input
    fn NATN(&mut self) {
        self.STATE.N_MUX_LOGIC = 3;
    }

    //Memory (X, Y) to N-adder input
    fn MTN(&mut self) {
        self.STATE.N_MUX_LOGIC = 0;
    }

    //F16 to N-adder input
    fn TN15(&mut self) { //_ required to parse as function
        self.STATE.N_MUX_LOGIC = 4;
    }

    //CKI to N-adder input
    fn CKN(&mut self) {
        self.STATE.N_MUX_LOGIC = 1;
    }


    //Adder/status instructions

    //One is added to the sum of P plus N inputs (P + N + 1)
    fn CIN(&mut self) {
        self.STATE.ADDER_INC = 1;
    }

    //Adder compares P and N inputs. If they are identical, status is set to zero
    fn NE(&mut self) {
        if (self.N_MUX() == self.P_MUX()) {
            self.STATE.STATUS = 0;
        }
        else {
            self.STATE.STATUS = 1;
        }
        self.STATE.STATUS_LIFETIME = 1 - self.STATE.STATUS;
    }

    //Carry is sent to status (MSB only)
    fn C8(&mut self) {
        self.STATE.STATUS = self.ADDER().0;
        self.STATE.STATUS_LIFETIME = 1 - self.STATE.STATUS;
    }

    //Write MUX instructions

    //Accumulator data to memory
    fn STO(&mut self) {
        self.STATE.RAM_ARRAY[self.STATE.X_REGISTER][self.STATE.Y_REGISTER] = self.STATE.ACCUMULATOR;
    }

    //CKI to memory
    fn CKM(&mut self) {
        self.STATE.RAM_ARRAY[self.STATE.X_REGISTER][self.STATE.Y_REGISTER] = self.CKI();
    }

    //AU Select/Status latch instructions

    //Adder result stored into accumulator
    fn AUTA(&mut self) {
        self.STATE.ACCUMULATOR = self.ADDER().1;
    }

    //Adder result stored into Y-register
    fn AUTY(&mut self) {
        self.STATE.Y_REGISTER = self.ADDER().1 as usize;
    }

    //Status is stored into status latch
    fn STSL(&mut self) {
        self.STATE.STATUS_LATCH = self.STATE.STATUS;
    }

//Hardware meta-instructions
//Instruction PLA and decoding

    const TMS1000_instructions : [fn(&mut SYSTEM); 16] = [SYSTEM::STO, SYSTEM::CKM, SYSTEM::CKP, SYSTEM::YTP, SYSTEM::MTP, SYSTEM::ATN, SYSTEM::NATN, SYSTEM::MTN, SYSTEM::TN15, SYSTEM::CKN, SYSTEM::NE, SYSTEM::C8, SYSTEM::CIN, SYSTEM::AUTA, SYSTEM::AUTY, SYSTEM::STSL];
    const TMS1000_mask : u32 = 0b0011111111001000;


    fn get_o_outputs(&mut self) -> u32 {
        let rval = match self.INSTRUCTION_PLA.get(&self.STATE.O_OUTPUT) {
            Some(v) => *v,
            None => 0,
        };
        return rval;
    }

    fn get_r_outputs(&mut self) -> Vec<u8>  {
        return self.STATE.R_OUTPUT.clone();
    }

    //Rom Address
    //Read RAM
    //ALU input
    //K-input value
    fn step_1(&mut self) {
        for i in 2..12 {
            if (self.STATE.INSTRUCTION_DECODED & (1 << i) != 0) {
                SYSTEM::TMS1000_instructions[i](self);
            }
        }

        match self.STATE.INSTRUCTION { //Based on timing table, RSTR appears to occur at the falling edge of this osc pulse
            0x0C => SYSTEM::RSTR(self),
            _ => ()
        }
    }

    //K-input valid
    fn step_2(&mut self) {

    }

    //Write RAM
    fn step_3(&mut self) {
        match self.STATE.INSTRUCTION {
            0x34..=0x37 => SYSTEM::RBIT(self),
            0x30..=0x33 => SYSTEM::SBIT(self),
            _ => ()
        }
        for i in 0..1 {
            if (self.STATE.INSTRUCTION_DECODED & (1 << i) != 0) {
                SYSTEM::TMS1000_instructions[i](self);
            }
        }
    }

    //Register store
    //Update PC
    //RAM Address
    //R-output register addressing takes place at the same time as RAM addressing (9-3.2)
    fn step_4(&mut self) {
        match self.STATE.INSTRUCTION {
            0x0D => SYSTEM::SETR(self),
            0x0A => SYSTEM::TDO(self),
            0x0B => (match self.VERSION {
                1100 | 1300 => SYSTEM::COMC(self),
                _ => SYSTEM::CLO(self),
            }) ,
            0x10..=0x1F => SYSTEM::LDP(self),
            0x28..=0x3F => (match self.VERSION {
                1100 | 1300 => SYSTEM::LDX_TMS1100(self),
                _ => SYSTEM::LDX_TMS1000(self),
            }) ,
            0x00 => (if !((1100 == self.VERSION) || (1300 == self.VERSION)) {
                SYSTEM::COMX(self)
            }),
            0x09 => (if ((1100 == self.VERSION) || (1300 == self.VERSION)) {
                SYSTEM::COMX(self)
            }),
            _ => ()
        }
        for i in 13..15 {
            if (self.STATE.INSTRUCTION_DECODED & (1 << i) != 0) {
                SYSTEM::TMS1000_instructions[i](self);
            }
        }
        self.INCREMENT_PC();
    }

    fn step_5(&mut self) {

    }

    //Instruction decode
    //Execute BR/CALL
    fn step_6(&mut self) {
        match self.STATE.INSTRUCTION {
            0x80..=0xBF => SYSTEM::BR(self),
            0xC0..=0xFF => SYSTEM::CALL(self),
            0x0F => SYSTEM::RETN(self), //Assuming that RETN executes at the same time as BR and CALL, for symmetry
            _ => ()
        }
        if self.STATE.STATUS == 0 {
            if self.STATE.STATUS_LIFETIME == 0 {
                self.STATE.STATUS = 1;
            }
            else {
                self.STATE.STATUS_LIFETIME -= 1;
            }
        }
        self.STATE.INSTRUCTION = self.ROM_ARRAY[(1024 * self.STATE.CHAPTER_ADDRESS) + (64 * self.STATE.PAGE_ADDRESS as usize) + self.STATE.PC_INDEX];
        self.STATE.INSTRUCTION_DECODED = (match self.INSTRUCTION_PLA.get(&(self.STATE.INSTRUCTION as u32)) {
            Some(output) => *output ^ SYSTEM::TMS1000_mask,
            None => SYSTEM::TMS1000_mask //Should be effectively a No-Op
        });
    }

    const steps : [fn(&mut SYSTEM); 6] = [SYSTEM::step_1, SYSTEM::step_2, SYSTEM::step_3, SYSTEM::step_4, SYSTEM::step_5, SYSTEM::step_6, ];


    pub fn STEP(&mut self, k_inp : u8) -> Self {
        self.STATE.K_INPUT = k_inp;
        SYSTEM::TMS1000_instructions[self.STATE.STEP](self);
        self.STATE.STEP = (self.STATE.STEP + 1 ) % 6;
        return self.clone();

    }

    //completes one full instruction cycle
    pub fn instruction_cycle(&mut self, k_inp : u8) -> Self {
        while (self.STATE.STEP < 5) {
            self.STEP(k_inp);
        }
        return self.STEP(k_inp);
    }

    //Replicates INIT pin behavior
    pub fn initialize(&mut self) {
        self.STATE.PAGE_ADDRESS = 15;
        self.STATE.PAGE_BUFFER = 15;
        self.STATE.PROGRAM_COUNTER = 0;
        self.STATE.PC_INDEX = 0;
        self.STATE.CHAPTER_ADDRESS = 0;
        self.STATE.CHAPTER_BUFFER = 0;
        self.STATE.CHAPTER_SUBROUTINE_LATCH = 0;
        self.STATE.CALL_LATCH = 0;
        self.STATE.R_OUTPUT = (match self.VERSION {
                    1200 | 1270 => vec![0; 13],
                    1300 => vec![0; 16],
                    _ => vec![0; 11],
                    });
        self.STATE.O_OUTPUT = 0;
        self.STATE.CALL_LATCH = 0;
    }

    //Reads PLA into a HashMap
    fn read_PLA(filename : String) -> Result<HashMap<u32, u32>, &'static str> {
        let data: String =  match fs::read_to_string(filename) {
            Ok(v) => v,
            Err(_) => return Err("Problem opening or reading PLA file"),
        };
        let re = Regex::new(r"([\-0-1]+) ([\-0-1]+)").unwrap(); //Unwrapping a static valid regex should be safe
        let mut pla_table = HashMap::new();

        for line in re.captures_iter(&data) {
            let mut inputs = Vec::new();
            inputs.push(0b0);
            let output = u32::from_str_radix(&line[2], 2).unwrap(); //Should be guarenteed by regex

            if !(output == 0) { //empty lines are skipped over
                for ch in line[1].chars() {
                    if ch == '-' {
                        let mut input0 = Vec::new();
                        let mut input1 = Vec::new();
                        for ref input in &inputs {
                            input0.push(*input << 1);
                            input1.push((*input << 1) + 1);
                        }
                        inputs = Vec::new();
                        inputs.append(&mut input1);
                        inputs.append(&mut input0);
                    }
                    else if ch == '1' {
                        for input in &mut inputs {
                            *input = (&*input << 1) + 1;
                        }
                    }
                    else {
                        for input in &mut inputs {
                            *input = &*input << 1;
                        }
                    }
                }

                for input in inputs {
                    if !pla_table.contains_key(&input) {
                        pla_table.insert(input, output);
                    }
                    else {
                        return Err("Same input maps to multiple outputs");
                    }
                }
            }
        }
        Ok(pla_table)
    }

    pub fn load_system(version: u32, rom_file : String, ipla_file : String, opla_file : String) -> Result<Self, String> {

        let iPLA = match Self::read_PLA(ipla_file) {
            Ok(v) => v,
            Err(v) => {
                let e : String = "Instruction PLA error: ".to_owned() + v;
                return Err(e)},
        };

        let oPLA = match Self::read_PLA(opla_file) {
            Ok(v) => v,
            Err(v) => {
                let e : String = "Output PLA error: ".to_owned() + v;
                return Err(e)},
        };

        let mut rFile = match fs::File::open(rom_file) {
            Ok(v) => v,
            Err(_) => return Err("ROM error: Problem opening ROM file".to_owned()),
        };

        let mut rom_array = vec![];
        let _ = rFile.read_to_end(&mut rom_array);


        let mut sys = SYSTEM {
            VERSION: version,
            STATE: SYSTEM_STATE {
                STEP : 0,
                INSTRUCTION : 127, //should function as a no-op until incremented
                INSTRUCTION_DECODED : 0b0011111111001000,
                PROGRAM_COUNTER: 0,
                PC_INDEX: 0,
                SUBROUTINE_RETURN : 0,
                PAGE_ADDRESS: 15,
                PAGE_BUFFER: 15,
                CHAPTER_ADDRESS: 0, //On non TMS1100/1300 systems these will never be changed
                CHAPTER_BUFFER: 0,
                CHAPTER_SUBROUTINE_LATCH: 0,
                CALL_LATCH: 0,
                R_OUTPUT: (match version {
                    1200 | 1270 => vec![0; 13],
                    1300 => vec![0; 16],
                    _ => vec![0; 11],
                    }),
                O_OUTPUT: 0,
                STATUS: 1,
                STATUS_LIFETIME: 0,
                ADDER_INC: 0,
                K_INPUT: 0,
                RAM_ARRAY: [[255; 16]; 8], //this and all below are set to an invalid value, must be properly initialized by code
                X_REGISTER: 255,
                Y_REGISTER: 255,
                STATUS_LATCH: 255,
                ACCUMULATOR: 255,
                P_MUX_LOGIC: 0, //There are theoretical very niche circumstances when a valid value for these would be necessary and desirable - but it's undefined and bad coding
                N_MUX_LOGIC: 0,
            },
            ROM_ARRAY: rom_array,
            INSTRUCTION_PLA: iPLA,
            OUTPUT_PLA: oPLA,
        };

        return Ok(sys);
    }
}




