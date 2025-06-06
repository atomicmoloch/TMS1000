#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused_parens)]
#![allow(non_upper_case_globals)]
//using all caps to denote actual system variables
//and camelcase to denote handler elements

use std::fs;
use std::io::Read;
use std::collections::HashMap;
use regex::Regex;


use crate::decompiler;

//HELPER FUNCTIONS
//imitates smaller than u8


fn u2_usize(value : usize) -> usize {
    return value % 4;
}


fn u4(value : u8) -> u8 {
    return value % 16;
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

    ACCUMULATOR: u8, //U4 A, storage register
    ADDER_INC: u8, //u1 - whether to increment the adder - set by C8 microinstruction and should be reset to 0 every cycle
    P_MUX: u8,
    N_MUX: u8,

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
    logging: bool, //If expanded, should be a general 'systems settings' object
}

impl SYSTEM {

//System components

    fn CKI(&mut self) -> u8 {
        //selects either constant field, k input to enter cki data bus, or bit mask
        match self.STATE.INSTRUCTION {
            0x00..=0x07 => {self.log_append(format!("CKI: Returning constant operand {}", reversebits_u4(self.STATE.INSTRUCTION)));
                            return reversebits_u4(self.STATE.INSTRUCTION);}, //constant
            0x08..=0x0F => {self.log_append(format!("CKI: Returning K inputs {}", self.STATE.K_INPUT));
                            return self.STATE.K_INPUT;},
            0x30..=0x3A => {self.log_append(format!("CKI: Returning bitmask {}", (15 - reversebits_u2(self.STATE.INSTRUCTION))));
                            return 15 - reversebits_u2(self.STATE.INSTRUCTION);}, //bit mask
            0x40..=0x7F => {self.log_append(format!("CKI: Returning constant operand {}", reversebits_u4(self.STATE.INSTRUCTION)));
                            return reversebits_u4(self.STATE.INSTRUCTION);},
            _ => {self.log_append("CKI: Returning 0".to_string());
                            return 0;},
        }
    }

    fn ADDER(&mut self) -> (u8, u8) {
        if (self.STATE.P_MUX == 255) || (self.STATE.N_MUX == 255) {
            self.log_append("Adder: ALERT! Inputs set to uninitialized values".into());
        }
        let value: u32 = self.STATE.P_MUX as u32 + self.STATE.N_MUX as u32 + self.STATE.ADDER_INC as u32;
        let carry = if value > 15 { 1 } else { 0 };
        let return_value = (carry, u4(value as u8));
        self.log_append(format!("Adder: Returning {} + {} + {} = ({}, {})", self.STATE.P_MUX, self.STATE.N_MUX, self.STATE.ADDER_INC, return_value.0, return_value.1));
        return return_value;
    }

    fn INCREMENT_PC(&mut self) {
        self.STATE.PC_INDEX = u6_usize(self.STATE.PC_INDEX + 1);
        self.STATE.PROGRAM_COUNTER = PC_SEQ[self.STATE.PC_INDEX];
        self.log_append(format!("Program Counter: incremented to {} ({})", self.STATE.PROGRAM_COUNTER, self.STATE.PC_INDEX));
        if self.STATE.CALL_LATCH == 0 {
            self.STATE.SUBROUTINE_RETURN = PC_SEQ[self.STATE.PC_INDEX];
        }
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
        self.log_append(format!("Program Counter: set to {} ({})", self.STATE.PROGRAM_COUNTER, self.STATE.PC_INDEX));
        if self.STATE.CALL_LATCH == 0 {
            self.STATE.SUBROUTINE_RETURN = PC_SEQ[self.STATE.PC_INDEX];
        }
    }

//Microinstructions

    //Branch on status = one
    fn BR (&mut self) {
        //On status: changes PC to br value and if call latch not active, moved PB to PA
        //If not status: increments PC and changes status to 1
        if (self.STATE.STATUS == 1) {
            self.log_append("BR: Status = 1".to_string());
            if (self.STATE.CALL_LATCH == 0) {
                self.STATE.PAGE_ADDRESS = self.STATE.PAGE_BUFFER;
                self.log_append(format!("BR: CL = 0. PA set to PB value {}", self.STATE.PAGE_BUFFER));
            }
            else {
                self.log_append("BR: CL = 1".to_string());
            }
            self.STATE.CHAPTER_ADDRESS = self.STATE.CHAPTER_BUFFER;
            self.log_append(format!("BR: CA set to CB value {}", self.STATE.CHAPTER_BUFFER));

            self.SET_PC(u6(self.STATE.INSTRUCTION) as usize);
    //          self.DECREMENT_PC(); //Since Step() will increment it again
        }
        else {
            self.log_append(("BR: Status = 0. Branch not taken.").to_string());
        }
    }

    //Call subroutine on status = one
    fn CALL (&mut self) {
        if (self.STATE.STATUS == 1) {
            self.log_append("CALL: Status = 1".to_string());
            if (self.STATE.CALL_LATCH == 0) {
                self.log_append("CALL: CL = 0".to_string());

                self.STATE.SUBROUTINE_RETURN = PC_SEQ[self.STATE.PC_INDEX];
                self.log_append(format!("CALL: SR disconnected. Current value {}", self.STATE.SUBROUTINE_RETURN));

                (self.STATE.PAGE_ADDRESS, self.STATE.PAGE_BUFFER) = (self.STATE.PAGE_BUFFER, self.STATE.PAGE_ADDRESS);
                self.log_append(format!("CALL: PA (now: {}) and PB (now: {}) swapped", self.STATE.PAGE_ADDRESS, self.STATE.PAGE_BUFFER));

                self.STATE.CHAPTER_SUBROUTINE_LATCH = self.STATE.CHAPTER_ADDRESS;
                self.log_append(format!("CALL: CSL set to CA value {}", self.STATE.CHAPTER_SUBROUTINE_LATCH));

                self.STATE.CHAPTER_ADDRESS = self.STATE.CHAPTER_BUFFER;
                self.log_append(format!("CALL: CA set to CB value {}", self.STATE.CHAPTER_ADDRESS));

                self.STATE.CALL_LATCH = 1;
                self.log_append("CALL: CL set to 1".to_string());
            }
            else {
                self.log_append("CALL: CL = 1".to_string());
                 self.log_append("CALL: ALERT! Call attempted inside of another call".to_string());
                self.STATE.CHAPTER_ADDRESS = self.STATE.CHAPTER_BUFFER;
                self.log_append(format!("CALL: CA set to CB value {}", self.STATE.CHAPTER_BUFFER));

                self.STATE.PAGE_BUFFER = self.STATE.PAGE_ADDRESS;
                self.log_append(format!("CALL: PB set to PA value {}", self.STATE.PAGE_BUFFER));
            }
            self.SET_PC(u6(self.STATE.INSTRUCTION) as usize);
    //         self.DECREMENT_PC();
        }
        else {
            self.STATE.STATUS = 1;
            self.log_append(("CALL: Status = 0. Call not executed.").to_string());
        }
    }

    //Return from subroutine
    fn RETN(&mut self) {
        self.STATE.PAGE_ADDRESS = self.STATE.PAGE_BUFFER;
        self.log_append(format!("RETN: PA set to PB value {}", self.STATE.PAGE_BUFFER));

        if (self.STATE.CALL_LATCH == 1) {
            self.log_append("RETN: CL = 1".to_string());

            self.SET_PC(self.STATE.SUBROUTINE_RETURN);

            self.STATE.CHAPTER_ADDRESS = self.STATE.CHAPTER_SUBROUTINE_LATCH;
            self.log_append(format!("RETN: CA set to CSL value {}", self.STATE.CHAPTER_ADDRESS));

            self.STATE.CALL_LATCH = 0;
            self.log_append("RETN: CL set to 0".to_string());
        }
        else {
            self.log_append("RETN: CL = 0".to_string());
        }
        //Step will increment PC
    }

    //Load page buffer with constant
    fn LDP (&mut self) {
        self.STATE.PAGE_BUFFER = reversebits_u4(self.STATE.INSTRUCTION); //MSB on right
        self.log_append(format!("LDP: PB set to {}", self.STATE.PAGE_BUFFER));
    }

    //Load X register with constant
    fn LDX_TMS1000(&mut self) {
        self.STATE.X_REGISTER = reversebits_u2(self.STATE.INSTRUCTION) as usize;
        self.log_append(format!("LDX: X register set to {}", self.STATE.X_REGISTER));
    }

    fn LDX_TMS1100(&mut self) {
        self.STATE.X_REGISTER = reversebits_u3(self.STATE.INSTRUCTION) as usize;
        self.log_append(format!("LDX: X register set to {}", self.STATE.X_REGISTER));
    }

    //Complement X
    fn COMX (&mut self) {
        if (self.VERSION == 1100) || (self.VERSION == 1300) {
            //Changes MSB of X register
            self.STATE.X_REGISTER ^= 0b1 << 2;
            self.log_append(format!("COMX: X register set to {}", self.STATE.X_REGISTER));
        }
        else {
            //Should flip bits of X register (1s compliment)
            self.STATE.X_REGISTER = u2_usize(!self.STATE.X_REGISTER);
            self.log_append(format!("COMX: X register set to {}", self.STATE.X_REGISTER));
        }
    }

    //Transfer data from accumulator and status latch to O outputs
    fn TDO (&mut self) {
        //Acc and SL transferred to O-output register
        if (self.STATE.ACCUMULATOR == 255) || (self.STATE.STATUS_LATCH == 255) {
            self.log_append("TDO: ALERT! Uninitialized value being stored.".into());
        }
        self.STATE.O_OUTPUT = u5_u32((self.STATE.ACCUMULATOR + (self.STATE.STATUS_LATCH << 4)).into());
        self.log_append(format!("TDO: O output set to {:b}", self.STATE.O_OUTPUT));
    }

    //Clear O-output register
    fn CLO (&mut self) {
        //zeroes O-register
        self.STATE.O_OUTPUT = 0;
        self.log_append("CLO: O output cleared".to_string());
    }

    fn COMC (&mut self) {
        //Toggles chapter buffer
        self.STATE.CHAPTER_BUFFER = (self.STATE.CHAPTER_BUFFER + 1) % 2;
        self.log_append(format!("COMC: CB set to {}", self.STATE.CHAPTER_BUFFER));
    }

    //Set R output addressed by Y
    fn SETR (&mut self) {
        //sets R(Y) to 1; if Y out of range, no-op
        if self.STATE.Y_REGISTER == 255 {
            self.log_append("SETR: ALERT! Uninitialized Y register value being used.".into());
        }
        if (self.STATE.Y_REGISTER < self.STATE.R_OUTPUT.len()) && (self.STATE.X_REGISTER < 4) {
            self.STATE.R_OUTPUT[self.STATE.Y_REGISTER] = 1;
            self.log_append(format!("SETR: R output {} set to 1", self.STATE.Y_REGISTER));
        }
        else {
            self.log_append("SETR: Y register out of range".to_string());
        }
    }

    //Reset R output addressed by Y
    fn RSTR (&mut self) {
        //sets R(Y) to 0; if Y out of range, no-op
        if self.STATE.Y_REGISTER == 255 {
            self.log_append("SETR: ALERT! Uninitialized Y register value being used.".into());
        }
        if (self.STATE.Y_REGISTER < self.STATE.R_OUTPUT.len()) && (self.STATE.X_REGISTER < 4) {
            self.STATE.R_OUTPUT[self.STATE.Y_REGISTER] = 0;
            self.log_append(format!("RSETR: R output {} set to 0", self.STATE.Y_REGISTER));
        }
        else {
            self.log_append("RSETR: Y register out of range".to_string());
        }
    }

    //Set memory bit
    fn SBIT (&mut self) {
        //sets BIT of RAM(X,Y) to 1
        let BIT_U8 = reversebits_u2(self.STATE.INSTRUCTION);
        let IS_SET = self.STATE.RAM_ARRAY[self.STATE.X_REGISTER][self.STATE.Y_REGISTER] & (1_u8 << BIT_U8) != 0;
        if !(IS_SET) {
            self.STATE.RAM_ARRAY[self.STATE.X_REGISTER][self.STATE.Y_REGISTER] = u4(self.STATE.RAM_ARRAY[self.STATE.X_REGISTER][self.STATE.Y_REGISTER] + (1_u8 << BIT_U8));
            self.log_append(format!("SBIT: Set bit {} at RAM address {}, {} to 1", BIT_U8, self.STATE.X_REGISTER, self.STATE.Y_REGISTER));
        }
        else {
            self.log_append(format!("SBIT: Bit {} at RAM address {}, {} was already set to 1", BIT_U8, self.STATE.X_REGISTER, self.STATE.Y_REGISTER));
        }
    }

    //Reset memory bit
    fn RBIT (&mut self) {
        //sets BIT of RAM(X,Y) to 0
        let BIT_U8 = reversebits_u2(self.STATE.INSTRUCTION);
        let IS_SET = self.STATE.RAM_ARRAY[self.STATE.X_REGISTER][self.STATE.Y_REGISTER] & (1_u8 << BIT_U8) != 0;
        if (IS_SET) {
            self.STATE.RAM_ARRAY[self.STATE.X_REGISTER][self.STATE.Y_REGISTER] = u4(self.STATE.RAM_ARRAY[self.STATE.X_REGISTER][self.STATE.Y_REGISTER] - (1_u8 << BIT_U8));
            self.log_append(format!("RBIT: Set bit {} at RAM address {}, {} to 0", BIT_U8, self.STATE.X_REGISTER, self.STATE.Y_REGISTER));
        }
        else {
            self.log_append(format!("SBIT: Bit {} at RAM address {}, {} was already set to 0", BIT_U8, self.STATE.X_REGISTER, self.STATE.Y_REGISTER));
        }
    }

    //P-MUX instructions

    //CKI to P-adder input
    fn CKP(&mut self) {
        self.log_append("CKP: P-MUX set to output CKI".to_string());
        self.STATE.P_MUX = self.CKI();
    }

    //Y-register to P-adder input
    fn YTP(&mut self) {
        self.log_append(format!("YTP: P-MUX set to output Y register value {}", self.STATE.Y_REGISTER));
        self.STATE.P_MUX = self.STATE.Y_REGISTER as u8;
    }

    //Memory (X, Y) to P-adder input
    fn MTP(&mut self) {
        self.STATE.P_MUX = self.STATE.RAM_ARRAY[self.STATE.X_REGISTER][self.STATE.Y_REGISTER];
        self.log_append(format!("MTP: P-MUX set to output RAM value {}", self.STATE.RAM_ARRAY[self.STATE.X_REGISTER][self.STATE.Y_REGISTER]));
    }

    //N-MUX instructions

    //Accumulator to N-adder input
    fn ATN(&mut self) {
        self.log_append(format!("ATN: N-MUX set to accumulator value {}", self.STATE.ACCUMULATOR));
        self.STATE.N_MUX = self.STATE.ACCUMULATOR;
    }

    //not-accumulator to N-adder input
    fn NATN(&mut self) {
        self.log_append("NATN: N-MUX set to output inverted accumulator".to_string());
        self.STATE.N_MUX = u4(1 + !(self.STATE.ACCUMULATOR));
    }

    //Memory (X, Y) to N-adder input
    fn MTN(&mut self) {
        self.STATE.N_MUX = self.STATE.RAM_ARRAY[self.STATE.X_REGISTER][self.STATE.Y_REGISTER];
        self.log_append(format!("MTN: N-MUX set to output RAM value {}", self.STATE.RAM_ARRAY[self.STATE.X_REGISTER][self.STATE.Y_REGISTER]));
    }

    //F16 to N-adder input
    fn TN15(&mut self) {
        self.STATE.N_MUX = 15;
        self.log_append("15TN: N-MUX set to output 15".to_string());
    }

    //CKI to N-adder input
    fn CKN(&mut self) {
        self.STATE.N_MUX = self.CKI();
        self.log_append("CKN: N-MUX set to output CKI".to_string());
    }


    //Adder/status instructions

    //One is added to the sum of P plus N inputs (P + N + 1)
    fn CIN(&mut self) {
        self.STATE.ADDER_INC = 1;
        self.log_append("CIN: 1 added to P and N adder inputs".to_string());
    }

    //Adder compares P and N inputs. If they are identical, status is set to zero
    fn NE(&mut self) {
        if (self.STATE.N_MUX == self.STATE.P_MUX) {
            self.STATE.STATUS = 0;
            self.log_append("NE: P and N adder inputs identical. Status set to 0".to_string());
        }
        else {
            self.log_append("NE: P and N adder inputs different. Status set to 1".to_string());
            self.STATE.STATUS = 1;
        }
        self.STATE.STATUS_LIFETIME = 1 - self.STATE.STATUS;
    }

    //Carry is sent to status (MSB only)
    fn C8(&mut self) {
        self.STATE.STATUS = self.ADDER().0;
        self.log_append(format!("C8: Status set to adder carry value {}", self.STATE.STATUS));
        self.STATE.STATUS_LIFETIME = 1 - self.STATE.STATUS;
    }

    //Write MUX instructions

    //Accumulator data to memory
    fn STO(&mut self) {
        if self.STATE.ACCUMULATOR == 255 {
            self.log_append("STO: ALERT! Uninitialized value being stored".into());
        }
        self.STATE.RAM_ARRAY[self.STATE.X_REGISTER][self.STATE.Y_REGISTER] = self.STATE.ACCUMULATOR;
        self.log_append(format!("STO: RAM location {}, {} set to accumulator value {}", self.STATE.X_REGISTER, self.STATE.Y_REGISTER, self.STATE.RAM_ARRAY[self.STATE.X_REGISTER][self.STATE.Y_REGISTER]));
    }

    //CKI to memory
    fn CKM(&mut self) {
        self.STATE.RAM_ARRAY[self.STATE.X_REGISTER][self.STATE.Y_REGISTER] = self.CKI();
        self.log_append(format!("CKM: RAM location {}, {} set to CKI value {}", self.STATE.X_REGISTER, self.STATE.Y_REGISTER, self.STATE.RAM_ARRAY[self.STATE.X_REGISTER][self.STATE.Y_REGISTER]));
    }

    //AU Select/Status latch instructions

    //Adder result stored into accumulator
    fn AUTA(&mut self) {
        self.STATE.ACCUMULATOR = self.ADDER().1;
        self.log_append(format!("AUTA: Accumulator set to adder result {}", self.STATE.ACCUMULATOR));
    }

    //Adder result stored into Y-register
    fn AUTY(&mut self) {
        self.STATE.Y_REGISTER = self.ADDER().1 as usize;
        self.log_append(format!("AUTY: Y register set to adder result {}", self.STATE.Y_REGISTER));
    }

    //Status is stored into status latch
    fn STSL(&mut self) {
        self.STATE.STATUS_LATCH = self.STATE.STATUS;
        self.log_append(format!("STSL: Status latch set to status value {}", self.STATE.STATUS_LATCH));
    }

//Hardware meta-instructions
//Instruction PLA and decoding

    const TMS1000_instructions : [fn(&mut SYSTEM); 16] = [SYSTEM::STO, SYSTEM::CKM, SYSTEM::CKP, SYSTEM::YTP, SYSTEM::MTP, SYSTEM::ATN, SYSTEM::NATN, SYSTEM::MTN, SYSTEM::TN15, SYSTEM::CKN, SYSTEM::NE, SYSTEM::C8, SYSTEM::CIN, SYSTEM::AUTA, SYSTEM::AUTY, SYSTEM::STSL];
    const TMS1000_mask : u32 = 0b0001001111111100;

    //Rom Address
    //Read RAM
    //ALU input
    //K-input value
    fn step_1(&mut self) {
        self.STATE.ADDER_INC = 0; //used by CIN; a little clumsy
        self.STATE.P_MUX = 0;
        self.STATE.N_MUX = 0;

        for i in 2..=12 {
            if (self.STATE.INSTRUCTION_DECODED & (1 << i) != 0) && !(i == 10 || i == 11){
                SYSTEM::TMS1000_instructions[i](self);
            }
        }
        for i in 10..=11 { //Ensures that NE and C8 only function after all inputs are loaded in
            if (self.STATE.INSTRUCTION_DECODED & (1 << i)) != 0 {
                SYSTEM::TMS1000_instructions[i](self);
            }
        }

        match self.STATE.INSTRUCTION { //Based on timing table, RSTR appears to occur at the falling edge of this osc pulse
            0x0C => SYSTEM::RSTR(self),
            _ => ()
        }
    }

    //Write RAM
    fn step_3(&mut self) {
        match self.STATE.INSTRUCTION {
            0x34..=0x37 => SYSTEM::RBIT(self),
            0x30..=0x33 => SYSTEM::SBIT(self),
            _ => ()
        }
        for i in 0..=1 {
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
            0x28..=0x2F => (match self.VERSION {
                1100 | 1300 => SYSTEM::LDX_TMS1100(self),
                _ => ()
            }),
            0x3C..=0x3F => (match self.VERSION {
                1100 | 1300 => (),
                _ => SYSTEM::LDX_TMS1000(self),
            }),
            0x00 => (if !((1100 == self.VERSION) || (1300 == self.VERSION)) {
                SYSTEM::COMX(self)
            }),
            0x09 => (if ((1100 == self.VERSION) || (1300 == self.VERSION)) {
                SYSTEM::COMX(self)
            }),
            _ => ()
        }
        for i in 13..=15 {
            if (self.STATE.INSTRUCTION_DECODED & (1 << i) != 0) {
                SYSTEM::TMS1000_instructions[i](self);
            }
        }
        self.INCREMENT_PC();
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
                self.log_append("Status set to 1".to_string());
            }
            else {
                self.STATE.STATUS_LIFETIME -= 1;
            }
        }

        self.STATE.INSTRUCTION = self.ROM_ARRAY[(1024 * self.STATE.CHAPTER_ADDRESS) + (64 * self.STATE.PAGE_ADDRESS as usize) + self.STATE.PROGRAM_COUNTER];
        self.log_append(format!("Instruction {:0>8b} loaded from ROM address {} {} {}", self.STATE.INSTRUCTION, self.STATE.CHAPTER_ADDRESS, self.STATE.PAGE_ADDRESS, self.STATE.PROGRAM_COUNTER));

        self.STATE.INSTRUCTION_DECODED = (match self.INSTRUCTION_PLA.get(&(self.STATE.INSTRUCTION as u32)) {
            Some(output) => output ^ SYSTEM::TMS1000_mask, //why was this dereferenced? removed
            None => 0, //Should be effectively a No-Op
        });
        self.log_append(format!("Instruction {:0>8b} decoded to {:b} (raw: {:b})", self.STATE.INSTRUCTION, self.STATE.INSTRUCTION_DECODED, self.STATE.INSTRUCTION_DECODED ^ SYSTEM::TMS1000_mask));
        self.log_append(format!("(Standard instruction {})", decompiler::decodeinstruction(self.STATE.INSTRUCTION, self.VERSION)));
    }

    const steps : [fn(&mut SYSTEM); 4] = [SYSTEM::step_1, SYSTEM::step_3, SYSTEM::step_4, SYSTEM::step_6];


    pub fn STEP(&mut self, k_inp : u8) -> Self {
        self.STATE.K_INPUT = k_inp;
        self.log_append(format!("Executing step {}", self.STATE.STEP));
        SYSTEM::steps[self.STATE.STEP](self);
        self.STATE.STEP = (self.STATE.STEP + 1 ) % 4;
        return self.clone();
    }

    pub fn STEP_mut(&mut self, k_inp : u8) {
        self.STATE.K_INPUT = k_inp;
        self.log_append(format!("Executing step {}", self.STATE.STEP));
        SYSTEM::steps[self.STATE.STEP](self);
        self.STATE.STEP = (self.STATE.STEP + 1 ) % 4;
    }

    //completes one full instruction cycle
    pub fn instruction_cycle(&mut self, k_inp : u8) -> Self {
        while (self.STATE.STEP < 3) {
            self.STEP(k_inp);
        }
        return self.STEP(k_inp);
    }

    pub fn instruction_cycle_mut(&mut self, k_inp : u8) {
        while (self.STATE.STEP < 3) {
            self.STEP_mut(k_inp);
        }
        self.STEP_mut(k_inp);
    }

    pub fn log_append(&mut self, entry: String) {
        if self.logging {
            self.STATE.LOG.push(entry);
        }
    }

    //Note: do not reverse the bits here. Already done in the read PLA functions.
    pub fn get_o_outputs(&mut self) -> u32 {
        let rval = match self.OUTPUT_PLA.get(&self.STATE.O_OUTPUT.clone()) {
            Some(v) => *v,
            None => 0,
        };
        return rval;
    }

    pub fn get_r_outputs_vec(&mut self) -> Vec<u8>  {
        return self.STATE.R_OUTPUT.clone();
    }

    pub fn get_r_outputs_u32(&mut self) -> u32 {
        let mut retval: u32 = 0;
        for (i, val) in self.STATE.R_OUTPUT.iter().enumerate() {
            retval += (*val as u32) << i;
        }
        return retval;
    }

    pub fn get_rom_index(&mut self) -> usize {
        return (self.STATE.CHAPTER_ADDRESS * 1024) + (self.STATE.PAGE_ADDRESS as usize * 64) + self.STATE.PC_INDEX;
    }

    pub fn get_log(&mut self) -> Vec<String> {
        let retval = self.STATE.LOG.clone();
        self.STATE.LOG = Vec::new();
        return retval;
    }

    pub fn reset_log(&mut self) {
        self.STATE.LOG = Vec::new();
    }

    pub fn get_ram_array(&mut self) -> [[u8; 16]; 8] {
        return self.STATE.RAM_ARRAY.clone();
    }

    pub fn get_x_reg(&mut self) -> usize {
        return self.STATE.X_REGISTER.clone();
    }

    pub fn get_y_reg(&mut self) -> usize {
        return self.STATE.Y_REGISTER.clone();
    }

    pub fn get_pa_reg(&mut self) -> u8 {
        return self.STATE.PAGE_ADDRESS.clone();
    }

    pub fn get_pb_reg(&mut self) -> u8 {
        return self.STATE.PAGE_BUFFER.clone();
    }

    pub fn get_cl_reg(&mut self) -> u8 {
        return self.STATE.CALL_LATCH.clone();
    }

    pub fn get_acc_reg(&mut self) -> u8 {
        return self.STATE.ACCUMULATOR.clone();
    }

    pub fn get_status(&mut self) -> u8 {
        return self.STATE.STATUS.clone();
    }

    pub fn get_sl_reg(&mut self) -> u8 {
        return self.STATE.STATUS_LATCH.clone();
    }

    pub fn get_ca_reg(&mut self) -> usize {
        return self.STATE.CHAPTER_ADDRESS.clone();
    }

    pub fn get_cb_reg(&mut self) -> usize {
        return self.STATE.CHAPTER_BUFFER.clone();
    }

    pub fn get_csl_reg(&mut self) -> usize {
        return self.STATE.CHAPTER_SUBROUTINE_LATCH.clone();
    }

    pub fn get_pc_reg(&mut self) -> usize {
        return self.STATE.PROGRAM_COUNTER.clone();
    }

    pub fn get_sr_reg(&mut self) -> usize {
        return self.STATE.SUBROUTINE_RETURN.clone();
    }

    pub fn get_ipla(&mut self) -> HashMap<u32, u32> {
        return self.INSTRUCTION_PLA.clone();
    }

    pub fn get_opla(&mut self) -> HashMap<u32, u32> {
        return self.OUTPUT_PLA.clone();
    }

    pub fn set_logging(&mut self) {
        self.logging = !self.logging;
    }

    //Replicates INIT pin behavior
    pub fn INITIALIZE(&mut self) {
        self.log_append("Hardware reinitialized".to_string());
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
    //Used in initialization (below)
    fn read_PLA(filename : String) -> Result<HashMap<u32, u32>, String> {
        let data: String =  match fs::read_to_string(filename) {
            Ok(v) => v,
            Err(_) => return Err("Problem opening or reading PLA file".to_string()),
        };
        let re = Regex::new(r"([\-0-1]+) ([0-1]+)").unwrap(); //Unwrapping a static valid regex should be safe
        let mut pla_table = HashMap::new();

        for line in re.captures_iter(&data) {
            let mut inputs = Vec::new();
            inputs.push(0b0);
            let output = u32::from_str_radix(line[2].as_ref(), 2).unwrap(); //Should be guarenteed by regex


            if !(output == 0) { //empty lines are skipped over
                for ch in line[1].chars().rev() {
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
                        let combined_output : u32 = output | pla_table.get(&input).unwrap();
                        //Bitwise ORs overlapping PLA terms
                        //Since if one term activates (and brings the instruction line to 1), the output line will be 1
                        pla_table.insert(input, combined_output);
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


        let sys = SYSTEM {
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
                P_MUX: 0,
                N_MUX: 0,
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
            },
            ROM_ARRAY: rom_array,
            INSTRUCTION_PLA: iPLA,
            OUTPUT_PLA: oPLA,
            logging: true,
        };

        return Ok(sys);
    }
}




