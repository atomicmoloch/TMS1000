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
    use std::fs::File;
    use std::io::Read;
    struct SYSTEM_STATE {
        X_REGISTER: u8, //U2 X, storage register; ram page address
        Y_REGISTER: u8, //U4 Y, storage register; ram word address and R output address

        PROGRAM_COUNTER: u8, //u6 PC, shift register
        PC_INDEX: u8, //u6, index for pseudo-random program counter
        SUBROUTINE_RETURN: u8, //u6 SR, storage register

        PAGE_ADDRESS: u8, //u4 PA, storage register; contains 4-bit page address of rom instructions
        PAGE_BUFFER: u8, //U4 PB storage register, used to set up page changes. also contains 4-bit return page address during call state
        CALL_LATCH: u8, //u1, CL, latch, stores call state

        //RAM array
        //four files with 16 * U4 each
        RAM_ARRAY: [[u8; 16]; 4],

        CKI_VALUE: u8, //u4; Value outputted by CKI bus. Varies (is set) based on opcode, independently of instruction executed
        P_MUX_LOGIC: u8, //u4, P-MUX: Data multiplexxer. Selects input to adder from RAM, CKI, or Y (0, 1, or 2)
        N_MUX_LOGIC: u8,//u5, N-MUX: Data multiplexxer. Selects N input to adder (0) RAM, (1) CKI, (2) accumulator, (3) not-accumulator or (4) F16

        ACCUMULATOR: u8, //U4 A, storage register
        ADDER_INC: u8, //u1 - whether to increment the adder - set by C8 microinstruction and should be reset to 0 every cycle
        STATUS: u8, //1-bit S, gates. conditional branch control. Normal state - 1. Branches are taken if S = 1. Selectively outputs a zero when carry is false or when logical compare is true. A zero lasts for one instruction cycle only.
        STATUS_LATCH: u8, //1-bit SL, latch, selectively stores status output. Transfers to O register w/ acc bits when TDO is executed

        //Outputs:
        R_OUTPUT: [u8; 11], //R output register - single bit RAM cells, latches for output to R buffers. Used to control external devices, display scans, input encoding, status logic outputs. Can be strobed to scan a key matrix.
        O_OUTPUT: u8, //U5, O output register. Used to transmit data
    }

    impl SYSTEM_STATE {
        fn ToString(&self) -> String {
            "{Self.X_REGISTER}\n{Self.Y_REGISTER}\n{Self.X_REGISTER}\n{Self.PROGRAM_COUNTER}\n{Self.PC_INDEX}\n{Self.SUBROUTINE_RETURN}\n{Self.PAGE_ADDRESS}\n{Self.PAGE_BUFFER}\n{Self.CALL_LATCH}\n{Self.RAM_ARRAY}\n{Self.CKI_VALUE}\n{Self.P_MUX_LOGIC}\n{Self.N_MUX_LOGIC}\n{Self.ACCUMULATOR}\n{Self.ADDER_INC}\n{Self.STATUS}\n{Self.STATUS_LATCH}\n{Self.R_OUTPUT}\n{Self.O_OUTPUT}".to_string()
        }

    }


    struct SYSTEM {
        STATE: SYSTEM_STATE,
        ROM_ARRAY: Vec<u8>,
     //   PC_SEQ: [U6; 64], not sure if changable
        //Output PLA
        //Instruction decode PLA
    }

    impl SYSTEM {


        //Replicates INIT pin behavior
    //    fn INITIALIZE() -> SYSTEM_STATE {

    //    }

        fn LoadSystem(rom_file : &'static str) -> Self {
            let mut sys = SYSTEM {
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
                    RAM_ARRAY: [[255; 16]; 4], //this and all below are set to an invalid value, must be properly initialized by code
                    X_REGISTER: 255,
                    Y_REGISTER: 255,
                    STATUS_LATCH: 255,
                    ACCUMULATOR: 255,
                    CKI_VALUE: 255,
                    P_MUX_LOGIC: 255,
                    N_MUX_LOGIC: 255,
                },
                ROM_ARRAY: vec![],
            };

            let file = File::open(rom_file);
            let _ = file.expect("REASON").read_to_end(&mut sys.ROM_ARRAY);

            return sys;
        }
    }

}


fn main() {


}
