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

static PC_SEQ: [usize; 64] = [0x00, 0x01, 0x03, 0x07, 0x0F, 0x1F, 0x3F, 0x3E, 0x3D, 0x3B, 0x37, 0x2F, 0x1E, 0x3C, 0x39, 0x33, 0x27, 0x0E, 0x1D, 0x3A, 0x35, 0x2B, 0x16, 0x2C, 0x18, 0x30, 0x21, 0x02, 0x05, 0x0B, 0x17, 0x2E, 0x1C, 0x38, 0x31, 0x23, 0x06, 0x0D, 0x1B, 0x36, 0x2D, 0x1A, 0x34, 0x29, 0x12, 0x24, 0x08, 0x11, 0x22, 0x04, 0x09, 0x13, 0x26, 0x0C, 0x19, 0x32, 0x25, 0x0A, 0x15, 0x2A, 0x14, 0x28, 0x10, 0x20];

//Main body

#[derive(Clone)]
struct SYSTEM_STATE {
    LOG: Vec<String>,

    INSTRUCTION: u8, //u8 current instruction
    INSTRUCTION_DECODED: u32,
    STEP: usize,

    X_REGISTER: usize, //U2 X, storage register; ram page address
    Y_REGISTER: usize, //U4 Y, storage register; ram word address and R output address

    PROGRAM_COUNTER: usize, //u6 PC, shift register
    PC_INDEX: usize, //u6, index for pseudo-random program counter
    SUBROUTINE_RETURN: usize, //u6 SR, storage register

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
            0x00..=0x07 => {self.STATE.LOG.push(format!("CKI: Returning constant operand {}", reversebits_u4(self.STATE.INSTRUCTION)));
                            return reversebits_u4(self.STATE.INSTRUCTION);}, //constant
            0x08..=0x0F => {self.STATE.LOG.push(format!("CKI: Returning K inputs {}", self.STATE.K_INPUT));
                            return self.STATE.K_INPUT;},
            0x20..=0x2F => {self.STATE.LOG.push("CKI: Returning 0".to_string());
                            return 0;},
            0x30..=0x3A => {self.STATE.LOG.push(format!("CKI: Returning bitmask {}", (15 - reversebits_u2(self.STATE.INSTRUCTION))));
                            return 15 - reversebits_u2(self.STATE.INSTRUCTION);}, //bit mask
            0x40..=0x7F => {self.STATE.LOG.push(format!("CKI: Returning constant operand {}", reversebits_u4(self.STATE.INSTRUCTION)));
                            return reversebits_u4(self.STATE.INSTRUCTION);},
            _ => {self.STATE.LOG.push("CKI: Invalid instruction".to_string());
                  return 255;}, //does nothing on LDP, LDX, BR and CALL instructions
        }
    }

    fn P_MUX(&mut self) -> u8 {
    //(0) Y register, (1) CKI, (2) RAM page
        match self.STATE.N_MUX_LOGIC {
            0 => {self.STATE.LOG.push(format!("P-MUX: Returning Y register value {}", self.STATE.Y_REGISTER));
                  return self.STATE.Y_REGISTER as u8;},
            1 => {self.STATE.LOG.push("P-MUX: Returning CKI value".to_string());
                  return self.CKI();},
            _ => {self.STATE.LOG.push(format!("P-MUX: Returning data in RAM at {}, {} : {}", self.STATE.X_REGISTER, self.STATE.Y_REGISTER, self.STATE.RAM_ARRAY[self.STATE.X_REGISTER][self.STATE.Y_REGISTER]));
                  return self.STATE.RAM_ARRAY[self.STATE.X_REGISTER][self.STATE.Y_REGISTER];},
        }
    }

    fn N_MUX(&mut self) -> u8 {
    //(0) RAM, (1) CKI, (2) accumulator, (3) not-accumulator or (4) F16
        match self.STATE.P_MUX_LOGIC {
            0 => {self.STATE.LOG.push(format!("N-MUX: Returning data in RAM at {}, {} : {}", self.STATE.X_REGISTER, self.STATE.Y_REGISTER, self.STATE.RAM_ARRAY[self.STATE.X_REGISTER][self.STATE.Y_REGISTER]));
                  return self.STATE.RAM_ARRAY[self.STATE.X_REGISTER][self.STATE.Y_REGISTER];}
            1 => {self.STATE.LOG.push("N-MUX: Returning CKI value".to_string());
                  return self.CKI();},
            2 => {self.STATE.LOG.push(format!("N-MUX: returning accumulator value {}", self.STATE.ACCUMULATOR));
                  return self.STATE.ACCUMULATOR;},
            3 => {self.STATE.LOG.push(format!("N-MUX: returning inverse of accumulator value {}", u4(1 + !(self.STATE.ACCUMULATOR))));
                  return u4(1 + !(self.STATE.ACCUMULATOR));},
            _ => {self.STATE.LOG.push("N-MUX: returning 15".to_string());
                  return 15;},
        }
    }

    fn ADDER(&mut self) -> (u8, u8) {
        let p_mux : u8 = self.P_MUX();
        let n_mux : u8 = self.N_MUX();
        let value : u8 = u5(p_mux.wrapping_add(n_mux.wrapping_add(self.STATE.ADDER_INC)));
        let return_value = (u1(value >> 4 & 1), u4(value));
        self.STATE.LOG.push(format!("Adder: Returning {} + {} + {} = ({}, {})", p_mux, n_mux, self.STATE.ADDER_INC, return_value.0, return_value.1));
        return return_value;
    }

    fn INCREMENT_PC(&mut self) {
        self.STATE.PC_INDEX = u6_usize(self.STATE.PC_INDEX + 1);
        self.STATE.PROGRAM_COUNTER = PC_SEQ[self.STATE.PC_INDEX];
        self.STATE.LOG.push(format!("Program Counter: incremented to {} ({})", self.STATE.PROGRAM_COUNTER, self.STATE.PC_INDEX));
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

    fn SET_PC(&mut self, value : usize) {
        self.STATE.PROGRAM_COUNTER = value;
        self.STATE.PC_INDEX = PC_SEQ.iter().position(|&i| i == value).unwrap(); //this should be guarenteed; thus the use of unwrap()
        self.STATE.LOG.push(format!("Program Counter: set to {} ({})", self.STATE.PROGRAM_COUNTER, self.STATE.PC_INDEX));
    }

//Microinstructions

    //Branch on status = one
    fn BR (&mut self) {
        //On status: changes PC to br value and if call latch not active, moved PB to PA
        //If not status: increments PC and changes status to 1
        if (self.STATE.STATUS == 1) {
            self.STATE.LOG.push("BR: Status = 1".to_string());
            if (self.STATE.CALL_LATCH == 0) {
                self.STATE.PAGE_ADDRESS = self.STATE.PAGE_BUFFER;
                self.STATE.LOG.push(format!("BR: PA set to PB value {}", self.STATE.PAGE_BUFFER));
            }
            else {
                self.STATE.LOG.push("BR: CL = 1".to_string());
            }
            self.STATE.CHAPTER_ADDRESS = self.STATE.CHAPTER_BUFFER;
            self.STATE.LOG.push(format!("BR: CA set to CB value {}", self.STATE.CHAPTER_BUFFER));

            self.SET_PC(u6(self.STATE.INSTRUCTION) as usize);
    //          self.DECREMENT_PC(); //Since Step() will increment it again
        }
        else {
            self.STATE.STATUS = 1;
            self.STATE.LOG.push(("BR: Status = 0. Status set to 1").to_string());
        }
    }

    //Call subroutine on status = one
    fn CALL (&mut self) {
        if (self.STATE.STATUS == 1) {
            self.STATE.LOG.push("CALL: Status = 1".to_string());
            if (self.STATE.CALL_LATCH == 0) {
                self.STATE.LOG.push("CALL: CL = 0".to_string());

                self.STATE.SUBROUTINE_RETURN = PC_SEQ[self.STATE.PC_INDEX];
                self.STATE.LOG.push(format!("CALL: SR set to {}", self.STATE.SUBROUTINE_RETURN));

                (self.STATE.PAGE_ADDRESS, self.STATE.PAGE_BUFFER) = (self.STATE.PAGE_BUFFER, self.STATE.PAGE_ADDRESS);
                self.STATE.LOG.push(format!("CALL: PA (now: {}) and PB (now: {}) swapped", self.STATE.PAGE_ADDRESS, self.STATE.PAGE_BUFFER));

                self.STATE.CHAPTER_SUBROUTINE_LATCH = self.STATE.CHAPTER_ADDRESS;
                self.STATE.LOG.push(format!("CALL: CSL set to CA value {}", self.STATE.CHAPTER_SUBROUTINE_LATCH));

                self.STATE.CHAPTER_ADDRESS = self.STATE.CHAPTER_BUFFER;
                self.STATE.LOG.push(format!("CALL: CA set to CB value {}", self.STATE.CHAPTER_ADDRESS));

                self.STATE.CALL_LATCH = 1;
                self.STATE.LOG.push("CALL: CL set to 1".to_string());
            }
            else {
                self.STATE.LOG.push("CALL: CL = 1".to_string());

                self.STATE.CHAPTER_ADDRESS = self.STATE.CHAPTER_BUFFER;
                self.STATE.LOG.push(format!("CALL: CA set to CB value {}", self.STATE.CHAPTER_BUFFER));

                self.STATE.PAGE_BUFFER = self.STATE.PAGE_ADDRESS;
                self.STATE.LOG.push(format!("CALL: PA set to PB value {}", self.STATE.PAGE_BUFFER));
            }
            self.SET_PC(u6(self.STATE.INSTRUCTION) as usize);
    //         self.DECREMENT_PC();
        }
        else {
            self.STATE.STATUS = 1;
            self.STATE.LOG.push(("CALL: Status = 0. Status set to 1").to_string());
        }
    }

    //Return from subroutine
    fn RETN(&mut self) {
        self.STATE.PAGE_ADDRESS = self.STATE.PAGE_BUFFER;
        self.STATE.LOG.push(format!("RETN: PA set to PB value {}", self.STATE.PAGE_BUFFER));

        if (self.STATE.CALL_LATCH == 1) {
            self.STATE.LOG.push("RETN: CL = 1".to_string());

            self.SET_PC(self.STATE.SUBROUTINE_RETURN);

            self.STATE.CHAPTER_ADDRESS = self.STATE.CHAPTER_SUBROUTINE_LATCH;
            self.STATE.LOG.push(format!("RETN: CA set to CSL value {}", self.STATE.CHAPTER_ADDRESS));

            self.STATE.CALL_LATCH = 0;
            self.STATE.LOG.push("RETN: CL set to 0".to_string());
        }
        else {
            self.STATE.LOG.push("RETN: CL = 0".to_string());
        }
        //Step will increment PC
    }

    //Load page buffer with constant
    fn LDP (&mut self) {
        self.STATE.PAGE_BUFFER = reversebits_u4(self.STATE.INSTRUCTION); //MSB on right
        self.STATE.LOG.push(format!("LDP: PB set to {}", self.STATE.PAGE_BUFFER));
    }

    //Load X register with constant
    fn LDX_TMS1000(&mut self) {
        self.STATE.X_REGISTER = reversebits_u2(self.STATE.INSTRUCTION) as usize;
        self.STATE.LOG.push(format!("LDX: X register set to {}", self.STATE.X_REGISTER));
    }

    fn LDX_TMS1100(&mut self) {
        self.STATE.X_REGISTER = reversebits_u3(self.STATE.INSTRUCTION) as usize;
        self.STATE.LOG.push(format!("LDX: X register set to {}", self.STATE.X_REGISTER));
    }

    //Complement X
    fn COMX (&mut self) {
        //Should flip bits of X register (1s compliment)
        self.STATE.X_REGISTER = u2_usize(!self.STATE.X_REGISTER);
        self.STATE.LOG.push(format!("COMX: X register set to {}", self.STATE.X_REGISTER));
    }

    fn COMX_TMS1000 (&mut self) {
        //Changes MSB of X register
        self.STATE.X_REGISTER ^= 0b1 << 2;
        self.STATE.LOG.push(format!("COMX: X register set to {}", self.STATE.X_REGISTER));
    }

    //Transfer data from accumulator and status latch to O outputs
    fn TDO (&mut self) {
        //Acc and SL transferred to O-output register
        self.STATE.O_OUTPUT = u5_u32((self.STATE.ACCUMULATOR + (self.STATE.STATUS_LATCH << 4)).into());
        self.STATE.LOG.push(format!("TDO: O output set to {:b}", self.STATE.O_OUTPUT));
    }

    //Clear O-output register
    fn CLO (&mut self) {
        //zeroes O-register
        self.STATE.O_OUTPUT = 0;
        self.STATE.LOG.push("CLO: O output cleared".to_string());
    }

    fn COMC (&mut self) {
        //Toggles chapter buffer
        self.STATE.CHAPTER_BUFFER = (self.STATE.CHAPTER_BUFFER + 1) % 1;
        self.STATE.LOG.push(format!("COMC: CB set to {}", self.STATE.CHAPTER_BUFFER));
    }

    //Set R output addressed by Y
    fn SETR (&mut self) {
        //sets R(Y) to 1; if Y out of range, no-op
        if (self.STATE.Y_REGISTER < self.STATE.R_OUTPUT.len()) && (self.STATE.X_REGISTER < 4) {
            self.STATE.R_OUTPUT[self.STATE.Y_REGISTER] = 1;
            self.STATE.LOG.push(format!("SETR: R output {} set to 1", self.STATE.Y_REGISTER));
        }
        else {
            self.STATE.LOG.push("SETR: Y register out of range".to_string());
        }
    }

    //Reset R output addressed by Y
    fn RSTR (&mut self) {
        //sets R(Y) to 0; if Y out of range, no-op
        if (self.STATE.Y_REGISTER < self.STATE.R_OUTPUT.len()) && (self.STATE.X_REGISTER < 4) {
            self.STATE.R_OUTPUT[self.STATE.Y_REGISTER] = 0;
            self.STATE.LOG.push(format!("RSETR: R output {} set to 0", self.STATE.Y_REGISTER));
        }
        else {
            self.STATE.LOG.push("RSETR: Y register out of range".to_string());
        }
    }

    //Set memory bit
    fn SBIT (&mut self) {
        //sets BIT of RAM(X,Y) to 1
        let BIT_U8 = reversebits_u2(self.STATE.INSTRUCTION);
        let IS_SET = self.STATE.RAM_ARRAY[self.STATE.X_REGISTER][self.STATE.Y_REGISTER] & (1_u8 << BIT_U8) != 0;
        if !(IS_SET) {
            self.STATE.RAM_ARRAY[self.STATE.X_REGISTER][self.STATE.Y_REGISTER] = u4(self.STATE.RAM_ARRAY[self.STATE.X_REGISTER][self.STATE.Y_REGISTER] + (1_u8 << BIT_U8));
            self.STATE.LOG.push(format!("SBIT: Set bit {} at RAM address {}, {} to 1", BIT_U8, self.STATE.X_REGISTER, self.STATE.Y_REGISTER));
        }
        else {
            self.STATE.LOG.push(format!("SBIT: Bit {} at RAM address {}, {} was already set to 1", BIT_U8, self.STATE.X_REGISTER, self.STATE.Y_REGISTER));
        }
    }

    //Reset memory bit
    fn RBIT (&mut self) {
        //sets BIT of RAM(X,Y) to 0
        let BIT_U8 = reversebits_u2(self.STATE.INSTRUCTION);
        let IS_SET = self.STATE.RAM_ARRAY[self.STATE.X_REGISTER][self.STATE.Y_REGISTER] & (1_u8 << BIT_U8) != 0;
        if (IS_SET) {
            self.STATE.RAM_ARRAY[self.STATE.X_REGISTER][self.STATE.Y_REGISTER] = u4(self.STATE.RAM_ARRAY[self.STATE.X_REGISTER][self.STATE.Y_REGISTER] - (1_u8 << BIT_U8));
            self.STATE.LOG.push(format!("RBIT: Set bit {} at RAM address {}, {} to 0", BIT_U8, self.STATE.X_REGISTER, self.STATE.Y_REGISTER));
        }
        else {
            self.STATE.LOG.push(format!("SBIT: Bit {} at RAM address {}, {} was already set to 0", BIT_U8, self.STATE.X_REGISTER, self.STATE.Y_REGISTER));
        }
    }

    //P-MUX instructions

    //CKI to P-adder input
    fn CKP(&mut self) {
        self.STATE.P_MUX_LOGIC = 1;
        self.STATE.LOG.push("CKP: P-MUX set to output CKI".to_string());
    }

    //Y-register to P-adder input
    fn YTP(&mut self) {
        self.STATE.P_MUX_LOGIC = 0;
        self.STATE.LOG.push("YTP: P-MUX set to output Y register".to_string());
    }

    //Memory (X, Y) to P-adder input
    fn MTP(&mut self) {
        self.STATE.P_MUX_LOGIC = 2;
        self.STATE.LOG.push("MTP: P-MUX set to output RAM".to_string());
    }

    //N-MUX instructions

    //Accumulator to N-adder input
    fn ATN(&mut self) {
        self.STATE.N_MUX_LOGIC = 2;
        self.STATE.LOG.push("ATN: N-MUX set to output accumulator".to_string());
    }

    //not-accumulator to N-adder input
    fn NATN(&mut self) {
        self.STATE.N_MUX_LOGIC = 3;
        self.STATE.LOG.push("ATN: N-MUX set to output inverted accumulator".to_string());
    }

    //Memory (X, Y) to N-adder input
    fn MTN(&mut self) {
        self.STATE.N_MUX_LOGIC = 0;
        self.STATE.LOG.push("ATN: N-MUX set to output RAM".to_string());
    }

    //F16 to N-adder input
    fn TN15(&mut self) {
        self.STATE.N_MUX_LOGIC = 4;
        self.STATE.LOG.push("ATN: N-MUX set to output 15".to_string());
    }

    //CKI to N-adder input
    fn CKN(&mut self) {
        self.STATE.N_MUX_LOGIC = 1;
        self.STATE.LOG.push("ATN: N-MUX set to output CKI".to_string());
    }


    //Adder/status instructions

    //One is added to the sum of P plus N inputs (P + N + 1)
    fn CIN(&mut self) {
        self.STATE.ADDER_INC = 1;
        self.STATE.LOG.push("CIN: 1 added to P and N adder inputs".to_string());
    }

    //Adder compares P and N inputs. If they are identical, status is set to zero
    fn NE(&mut self) {
        if (self.N_MUX() == self.P_MUX()) {
            self.STATE.STATUS = 0;
            self.STATE.LOG.push("NE: P and N adder inputs identical. Status set to 0".to_string());
        }
        else {
            self.STATE.LOG.push("NE: P and N adder inputs different. Status set to 1".to_string());
            self.STATE.STATUS = 1;
        }
        self.STATE.STATUS_LIFETIME = 1 - self.STATE.STATUS;
    }

    //Carry is sent to status (MSB only)
    fn C8(&mut self) {
        self.STATE.STATUS = self.ADDER().0;
        self.STATE.LOG.push(format!("C8: Status set to adder carry value {}", self.STATE.STATUS));
        self.STATE.STATUS_LIFETIME = 1 - self.STATE.STATUS;
    }

    //Write MUX instructions

    //Accumulator data to memory
    fn STO(&mut self) {
        self.STATE.RAM_ARRAY[self.STATE.X_REGISTER][self.STATE.Y_REGISTER] = self.STATE.ACCUMULATOR;
        self.STATE.LOG.push(format!("STO: RAM location {}, {} set to accumulator value {}", self.STATE.X_REGISTER, self.STATE.Y_REGISTER, self.STATE.RAM_ARRAY[self.STATE.X_REGISTER][self.STATE.Y_REGISTER]));
    }

    //CKI to memory
    fn CKM(&mut self) {
        self.STATE.RAM_ARRAY[self.STATE.X_REGISTER][self.STATE.Y_REGISTER] = self.CKI();
        self.STATE.LOG.push(format!("CKM: RAM location {}, {} set to CKI value {}", self.STATE.X_REGISTER, self.STATE.Y_REGISTER, self.STATE.RAM_ARRAY[self.STATE.X_REGISTER][self.STATE.Y_REGISTER]));
    }

    //AU Select/Status latch instructions

    //Adder result stored into accumulator
    fn AUTA(&mut self) {
        self.STATE.ACCUMULATOR = self.ADDER().1;
        self.STATE.LOG.push(format!("AUTA: Accumulator set to adder result {}", self.STATE.ACCUMULATOR));
    }

    //Adder result stored into Y-register
    fn AUTY(&mut self) {
        self.STATE.Y_REGISTER = self.ADDER().1 as usize;
        self.STATE.LOG.push(format!("AUTY: Y register set to adder result {}", self.STATE.Y_REGISTER));
    }

    //Status is stored into status latch
    fn STSL(&mut self) {
        self.STATE.STATUS_LATCH = self.STATE.STATUS;
        self.STATE.LOG.push(format!("STSL: Status latch set to status value {}", self.STATE.STATUS_LATCH));
    }

//Hardware meta-instructions
//Instruction PLA and decoding

    const TMS1000_instructions : [fn(&mut SYSTEM); 16] = [SYSTEM::STO, SYSTEM::CKM, SYSTEM::CKP, SYSTEM::YTP, SYSTEM::MTP, SYSTEM::ATN, SYSTEM::NATN, SYSTEM::MTN, SYSTEM::TN15, SYSTEM::CKN, SYSTEM::NE, SYSTEM::C8, SYSTEM::CIN, SYSTEM::AUTA, SYSTEM::AUTY, SYSTEM::STSL];
    const TMS1000_mask : u32 = 0b0011111111001000;

    //Rom Address
    //Read RAM
    //ALU input
    //K-input value
    fn step_1(&mut self) {
        self.STATE.ADDER_INC = 0; //used by CIN; a little clumsy

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
                self.STATE.LOG.push("Status set to 1".to_string());
            }
            else {
                self.STATE.STATUS_LIFETIME -= 1;
            }
        }

        self.STATE.INSTRUCTION = self.ROM_ARRAY[(1024 * self.STATE.CHAPTER_ADDRESS) + (64 * self.STATE.PAGE_ADDRESS as usize) + self.STATE.PROGRAM_COUNTER];
        self.STATE.LOG.push(format!("Instruction {:b} loaded from ROM address {} {} {}", self.STATE.INSTRUCTION, self.STATE.CHAPTER_ADDRESS, self.STATE.PAGE_ADDRESS, self.STATE.PROGRAM_COUNTER));

        self.STATE.INSTRUCTION_DECODED = (match self.INSTRUCTION_PLA.get(&(self.STATE.INSTRUCTION as u32)) {
            Some(output) => output ^ SYSTEM::TMS1000_mask, //why was this dereferenced? removed
            None => 0, //Should be effectively a No-Op
        });
        self.STATE.LOG.push(format!("Instruction {:b} decoded to {:b} (raw: {:b})", self.STATE.INSTRUCTION, self.STATE.INSTRUCTION_DECODED, self.STATE.INSTRUCTION_DECODED ^ SYSTEM::TMS1000_mask));
    }

    const steps : [fn(&mut SYSTEM); 6] = [SYSTEM::step_1, SYSTEM::step_2, SYSTEM::step_3, SYSTEM::step_4, SYSTEM::step_5, SYSTEM::step_6, ];


    pub fn STEP(&mut self, k_inp : u8) -> Self {
        self.STATE.K_INPUT = k_inp;
        self.STATE.LOG.push(format!("Executing step {}", self.STATE.STEP));
        SYSTEM::steps[self.STATE.STEP](self);
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
        self.STATE.LOG.push("Hardware reinitialized".to_string());
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

    pub fn get_o_outputs(&mut self) -> u32 {
        let rval = match self.INSTRUCTION_PLA.get(&self.STATE.O_OUTPUT) {
            Some(v) => *v,
            None => 0,
        };
        return rval;
    }

    pub fn get_r_outputs(&mut self) -> Vec<u8>  {
        return self.STATE.R_OUTPUT.clone();
    }

    pub fn get_log(&mut self) -> Vec<String> {
        let retval = self.STATE.LOG.clone();
        self.STATE.LOG = Vec::new();
        return retval;
    }

    //Reads PLA into a HashMap
    //Used in initialization (below)
    fn read_PLA(filename : String) -> Result<HashMap<u32, u32>, String> {
        let data: String =  match fs::read_to_string(filename) {
            Ok(v) => v,
            Err(_) => return Err("Problem opening or reading PLA file".to_string()),
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
                        println!("Input: {:b} Output: {:b}", input, output);
                    }
                    else {
                        let combined_output : u32 = output & pla_table.get(&input).unwrap();
                        pla_table.insert(input, combined_output);
                        println!("Input: {:b} New Output: {:b}", input, combined_output);
                      //  return Err(format!("Input {:b} maps to multiple outputs {:b} and {:b}", input, output, pla_table.get(&input).unwrap()));
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
                let e : String = "Instruction PLA error: ".to_owned() + &v;
                return Err(e)},
        };

        let oPLA = match Self::read_PLA(opla_file) {
            Ok(v) => v,
            Err(v) => {
                let e : String = "Output PLA error: ".to_owned() + &v;
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
                LOG : Vec::new(),
                STEP : 0,
                INSTRUCTION : 127, //should function as a no-op until incremented
                INSTRUCTION_DECODED : 0,
                PROGRAM_COUNTER: 0x20,
                PC_INDEX: 63, //Starts at last instruction so on first cycle increment will go to first instruction; as far as I can tell this is how the actual hardware did it too
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




