#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
//using all caps to denote actual system variables
//and camelcase to denote handler elements


//imitates smaller than u8
fn u1(value : u8) -> u8 {
    return value % 1;
}

fn u2(value : u8) -> u8 {
    return value % 4;
}

fn u4(value : u8) -> u8 {
    return value % 16;
}

fn u5(value : u8) -> u8 {
    return value % 32;
}

fn u6(value : u8) -> u8 {
    return value % 64;
}

fn reversebits_u4(value : u8) -> u8 {
    return value.reverse_bits() >> 4;
}

fn reversebits_u2(value : u8) -> u8 {
    return value.reverse_bits() >> 6;
}



mod TMS1000 {
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

    fn u4(value : u8) -> u8 {
        return value % 16;
    }

    fn u5(value : u8) -> u8 {
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

    fn reversebits_u2(value : u8) -> u8 {
        return value.reverse_bits() >> 6;
    }

    static PC_SEQ: [u8; 64] = [0x00, 0x01, 0x03, 0x07, 0x0F, 0x1F, 0x3F, 0x3E, 0x3D, 0x3B, 0x37, 0x2F, 0x1E, 0x3C, 0x39, 0x33, 0x27, 0x0E, 0x1D, 0x3A, 0x35, 0x2B, 0x16, 0x2C, 0x18, 0x30, 0x21, 0x02, 0x05, 0x0B, 0x17, 0x2E, 0x1C, 0x38, 0x31, 0x23, 0x06, 0x0D, 0x1B, 0x36, 0x2D, 0x1A, 0x34, 0x29, 0x12, 0x24, 0x08, 0x11, 0x22, 0x04, 0x09, 0x13, 0x26, 0x0C, 0x19, 0x32, 0x25, 0x0A, 0x15, 0x2A, 0x14, 0x28, 0x10, 0x20];

//Main body

    struct SYSTEM_STATE {
        X_REGISTER: usize, //U2 X, storage register; ram page address
        Y_REGISTER: usize, //U4 Y, storage register; ram word address and R output address

        PROGRAM_COUNTER: u8, //u6 PC, shift register
        PC_INDEX: usize, //u6, index for pseudo-random program counter
        SUBROUTINE_RETURN: u8, //u6 SR, storage register

        PAGE_ADDRESS: u8, //u4 PA, storage register; contains 4-bit page address of rom instructions
        PAGE_BUFFER: u8, //U4 PB storage register, used to set up page changes. also contains 4-bit return page address during call state
        CALL_LATCH: u8, //u1, CL, latch, stores call state

        //RAM array
        //four files with 16 * U4 each
        RAM_ARRAY: [[u8; 16]; 4],

        CKI_VALUE: u8, //u4; Value outputted by CKI bus. Varies (is set) based on opcode, independently of instruction executed
        P_MUX_LOGIC: u8, //u4, P-MUX: Data multiplexxer. Selects input to adder from Y register, CKI logic, or RAM array (0, 1, or 2)
        N_MUX_LOGIC: u8,//u5, N-MUX: Data multiplexxer. Selects N input to adder (0) RAM, (1) CKI, (2) accumulator, (3) not-accumulator or (4) F16

        ACCUMULATOR: u8, //U4 A, storage register
        ADDER_INC: u8, //u1 - whether to increment the adder - set by C8 microinstruction and should be reset to 0 every cycle
        STATUS: u8, //1-bit S, gates. conditional branch control. Normal state - 1. Branches are taken if S = 1. Selectively outputs a zero when carry is false or when logical compare is true. A zero lasts for one instruction cycle only.
        STATUS_LATCH: u8, //1-bit SL, latch, selectively stores status output. Transfers to O register w/ acc bits when TDO is executed

        //Outputs:
        R_OUTPUT: [u8; 11], //R output register - single bit RAM cells, latches for output to R buffers. Used to control external devices, display scans, input encoding, status logic outputs. Can be strobed to scan a key matrix. Using u8 instead of bool here costs a little memory but maintains consistency with the rest of the conventions. May change later.
        O_OUTPUT: u8, //U5, O output register. Used to transmit data

        K_INPUT: [u8; 4], //K input registers, K1, K2, K4, and K8
    }

    impl SYSTEM_STATE {
        fn ToString(&self) -> String {
            "{self.STATE.X_REGISTER}\n{self.STATE.Y_REGISTER}\n{self.STATE.X_REGISTER}\n{self.STATE.PROGRAM_COUNTER}\n{self.STATE.PC_INDEX}\n{self.STATE.SUBROUTINE_RETURN}\n{self.STATE.PAGE_ADDRESS}\n{self.STATE.PAGE_BUFFER}\n{self.STATE.CALL_LATCH}\n{self.STATE.RAM_ARRAY}\n{self.STATE.CKI_VALUE}\n{self.STATE.P_MUX_LOGIC}\n{self.STATE.N_MUX_LOGIC}\n{self.STATE.ACCUMULATOR}\n{self.STATE.ADDER_INC}\n{self.STATE.STATUS}\n{self.STATE.STATUS_LATCH}\n{self.STATE.R_OUTPUT}\n{self.STATE.O_OUTPUT}\n{self.STATE.K_INPUT}".to_string()
        }

    }


    pub struct SYSTEM {
        VERSION: u32,
        STATE: SYSTEM_STATE,
        ROM_ARRAY: Vec<u8>,
        INSTRUCTION_PLA: HashMap<u32, u32>,
        OUTPUT_PLA: HashMap<u32, u32>,
     //   PC_SEQ: [U6; 64], not sure if changable
    }

    impl SYSTEM {

    //Microinstructions
        fn INCREMENT_PC(&mut self) {
            self.STATE.PC_INDEX = u6_usize(self.STATE.PC_INDEX + 1);
            self.STATE.PROGRAM_COUNTER = PC_SEQ[self.STATE.PC_INDEX];
        }

        fn DECREMENT_PC(&mut self) {
            self.STATE.PC_INDEX = u6_usize(self.STATE.PC_INDEX - 1);
            self.STATE.PROGRAM_COUNTER = PC_SEQ[self.STATE.PC_INDEX];
        }

        fn SET_PC(&mut self, value : u8) {
            self.STATE.PROGRAM_COUNTER = value;
            self.STATE.PC_INDEX = PC_SEQ.iter().position(|&i| i == value).unwrap(); //this should be guarenteed; thus the use of unwrap()
        }

        fn BR (&mut self, instruction : u8) {
            //Branch instruction
            //On status: changes PC to br value and if call latch not active, moved PB to PA
            //If not status: increments PC and changes status to 1
            if (self.STATE.STATUS == 1) {
                if (self.STATE.CALL_LATCH == 0) {
                    self.STATE.PAGE_ADDRESS = self.STATE.PAGE_BUFFER;
                }
                self.SET_PC(u6(instruction));
                self.DECREMENT_PC(); //Since Step() will increment it again
            }
        }

        fn CALL (&mut self, instruction : u8) {
            //CALL SUBROUTINE instruction
            if (self.STATE.STATUS == 1) {
                if (self.STATE.CALL_LATCH == 0) {
                    self.STATE.SUBROUTINE_RETURN = PC_SEQ[self.STATE.PC_INDEX]; //removed the +1 for now, expecting Step to increment after RETN calls
                    (self.STATE.PAGE_ADDRESS, self.STATE.PAGE_BUFFER) = (self.STATE.PAGE_BUFFER, self.STATE.PAGE_ADDRESS);
                }
                else {
                    self.STATE.PAGE_BUFFER = self.STATE.PAGE_ADDRESS;
                }
                self.SET_PC(u6(instruction));
                self.DECREMENT_PC();
            }
        }

        fn RETN(&mut self, _ : u8) {
            self.STATE.PAGE_ADDRESS = self.STATE.PAGE_BUFFER;
            if (self.STATE.CALL_LATCH == 1) {
                self.SET_PC(self.STATE.SUBROUTINE_RETURN);
                self.STATE.CALL_LATCH = 0;
            }
            else {
                self.INCREMENT_PC();
            }
        }

        fn LDP (&mut self, instruction : u8) {
            self.STATE.PAGE_BUFFER = reversebits_u4(instruction); //MSB on right
        }

        fn LDX(&mut self, instruction : u8) {
            self.STATE.X_REGISTER = reversebits_u2(instruction) as usize;
        }

        fn COMX (&mut self, instruction : u8) {
            //Should flip bits of X register (1s compliment)
            self.STATE.X_REGISTER = u2_usize(self.STATE.X_REGISTER - 3);
        }

        fn TDO (&mut self, _ : u8) {
            //Acc and SL transferred to O-output register
            self.STATE.O_OUTPUT = u5(self.STATE.ACCUMULATOR + (self.STATE.STATUS_LATCH * 16));
        }

        fn CLO (&mut self, _ : u8) {
            //zeroes O-register
            self.STATE.O_OUTPUT = 0;
        }

        fn SETR (&mut self, _ : u8) {
            //sets R(Y) to 1; if Y out of range, no-op
            if (self.STATE.Y_REGISTER <= 10) {
                self.STATE.R_OUTPUT[self.STATE.Y_REGISTER] = 1;
            }
        }

        fn RSTR (&mut self, _ : u8) {
            //sets R(Y) to 0; if Y out of range, no-op
            if (self.STATE.Y_REGISTER <= 10) {
                self.STATE.R_OUTPUT[self.STATE.Y_REGISTER] = 0;
            }
        }

        fn SBIT (&mut self, instruction : u8) {
            //sets BIT of RAM(X,Y) to 1
            let BIT_U8 = reversebits_u2(instruction);
            let IS_SET = self.STATE.RAM_ARRAY[self.STATE.X_REGISTER][self.STATE.Y_REGISTER] & (1_u8 << BIT_U8) != 0;
            if !(IS_SET) {
                self.STATE.RAM_ARRAY[self.STATE.X_REGISTER][self.STATE.Y_REGISTER] = u4(self.STATE.RAM_ARRAY[self.STATE.X_REGISTER][self.STATE.Y_REGISTER] + (1_u8 << BIT_U8));
            }
        }

        fn RBIT (&mut self, instruction : u8) {
            //sets BIT of RAM(X,Y) to 0
            let BIT_U8 = reversebits_u2(instruction);
            let IS_SET = self.STATE.RAM_ARRAY[self.STATE.X_REGISTER][self.STATE.Y_REGISTER] & (1_u8 << BIT_U8) != 0;
            if (IS_SET) {
                self.STATE.RAM_ARRAY[self.STATE.X_REGISTER][self.STATE.Y_REGISTER] = u4(self.STATE.RAM_ARRAY[self.STATE.X_REGISTER][self.STATE.Y_REGISTER] - (1_u8 << BIT_U8));
            }
        }
    //Instruction PLA and decoding

    //Hardware meta-instructions
        //Replicates INIT pin behavior
        fn INITIALIZE(&mut self) {
            self.STATE.PAGE_ADDRESS = 15;
            self.STATE.PAGE_BUFFER = 15;
            self.STATE.PROGRAM_COUNTER = 0;
            self.STATE.PC_INDEX = 0;
            self.STATE.R_OUTPUT = [0; 11];
            self.STATE.O_OUTPUT = 0;
            self.STATE.CALL_LATCH = 0;
        }

        //Reads PLA into a HashMap
        fn read_PLA(filename : &'static str) -> Result<HashMap<u32, u32>, Box<dyn std::error::Error>> {
            let data: String = fs::read_to_string(filename)?;
            let re = Regex::new(r"([\-0-1]+) ([\-0-1]+)").unwrap();
            let mut pla_table = HashMap::new();

            for line in re.captures_iter(&data) {
                let mut inputs = Vec::new();
                inputs.push(0b0);
                let output = u32::from_str_radix(&line[2], 2).unwrap();

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
                    pla_table.insert(input, output);
                }
            }
            Ok(pla_table)
        }

        pub fn LoadSystem(version: u32, rom_file : &'static str, ipla_file : &'static str, opla_file : &'static str) -> Result<Self, &'static str> {

            let iPLA = match Self::read_PLA(ipla_file) {
                            Ok(v) => v,
                            Err(_) => return Err("Problem Loading PLA"),
            };

            let oPLA = match Self::read_PLA(opla_file) {
                            Ok(v) => v,
                            Err(_) => return Err("Problem Loading PLA"),
            };

            let mut rFile = match fs::File::open(rom_file) {
                Ok(v) => v,
                Err(_) => return Err("Problem Loading ROM"),
            };

            let mut rom_array = vec![];
            let _ = rFile.read_to_end(&mut rom_array);


            let mut sys = SYSTEM {
                VERSION: version,
                STATE: SYSTEM_STATE {
                    PROGRAM_COUNTER: 0,
                    PC_INDEX: 0,
                    SUBROUTINE_RETURN : 0,
                    PAGE_ADDRESS: 15,
                    PAGE_BUFFER: 15,
                    CALL_LATCH: 0,
                    R_OUTPUT: [0; 11],
                    O_OUTPUT: 0,
                    STATUS: 1,
                    ADDER_INC: 0,
                    K_INPUT: [0; 4],
                    RAM_ARRAY: [[255; 16]; 4], //this and all below are set to an invalid value, must be properly initialized by code
                    X_REGISTER: 255,
                    Y_REGISTER: 255,
                    STATUS_LATCH: 255,
                    ACCUMULATOR: 255,
                    CKI_VALUE: 255,
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

}


fn main() {
     let _ = match TMS1000::SYSTEM::LoadSystem(1000, "/home/moloch/Documents/thesis/tms/src/mp3300.bin", "/home/moloch/Documents/thesis/tms/src/tms1100_merlin_mpla.pla", "/home/moloch/Documents/thesis/tms/src/tms1100_merlin_opla.pla") {
        Ok(v) => println!("Success"),
        Err(e) => println!("{}", e),
    };
}
