//1 bit unsigned integer
//Implemented with unnecessary complexity as a struct
//in order to maintain consistency with u4, u6
#[derive(Clone)]
struct U1(u8);
impl U1 {
    fn new(x: u8) -> Self {
        U1(x % 1)
    fn get() -> u8 {
        self.0
    }
}
impl From<u8> for U1 {
    fn from(x: u8) -> Self {
        U1::new(x)
    }
}

//2 bit unsigned integer
#[derive(Clone)]
struct U2(u8);
impl U2 {
    fn new(x: u8) -> Self {
        U2(x % 4)
    }
    fn get() -> u8 {
        self.0
    }
}
impl From<u8> for U2 {
    fn from(x: u8) -> Self {
        U2::new(x)
    }
}

//4 bit unsigned integer
#[derive(Clone)]
struct U4(u8);
impl U4 {
    fn new(x: u8) -> Self {
        U4(x % 16)
    }
    fn get(self) -> u8 {
        self.0
    }
    //switches bit ordering
    //default: MSB on left
    fn reverse(self) {
        self.0 = self.0.reverse_bits() >> 4
    }
}
impl From<u8> for U4 {
    fn from(x: u8) -> Self {
        U4::new(x)
    }
}

//5 bit unsigned integer
#[derive(Clone)]
struct U5(u8);
impl U5 {
    fn new(x: u8) -> Self {
        U5(x % 32)
    }
    fn get() -> u8 {
        self.0
    }
}
impl From<u8> for U5 {
    fn from(x: u8) -> Self {
        U5::new(x)
    }
}

//6 bit unsigned integer
#[derive(Clone)]
struct U6(u8);
impl U6 {
    fn new(x: u8) -> Self {
        U6(x % 64)
    }
    fn get() -> u8 {
        self.0
    }
}
impl From<u8> for U6 {
    fn from(x: u8) -> Self {
        U6::new(x)
    }
}


//power up latch (not shown on diagram)

//Design thoughts:
//Logic handler contained within 'physical system handler'
//for interfacing with circuit simulation - e.g. physical system handler would call the 'init' function of the logic handler on receiving power to the init pin
//System objects: represent a system configuration
//systemstate objects represent a system state
//system objects behave as functions that take a system state and return a new one
//the oscillator can be represented as a main loop which iterates through systemstates
//things which are completely static (such as ALU behavior) and not dependent on system implementation or state can be implemented as static functions in TMS1000
//How to implement divergent behaviors of the TMS1100? will cross that bridge later
//Instructions and microinstructions: microinstructions implemented directly, instructions will call some number of microinstructions - or else pass microinstruction list up to be properly ordered
mod TMS1000 {
    let PC_SEQ: [U6; 64] = [0x00, 0x01, 0x03, 0x07, 0x0F, 0x1F, 0x3F, 0x3E, 0x3D, 0x3B, 0x37, 0x2F, 0x1E, 0x3C, 0x39, 0x33, 0x27, 0x0E, 0x1D, 0x3A, 0x35, 0x2B, 0x16, 0x2C, 0x18, 0x30, 0x21, 0x02, 0x05, 0x0B, 0x17, 0x2E, 0x1C, 0x38, 0x31, 0x23, 0x06, 0x0D, 0x1B, 0x36, 0x2D, 0x1A, 0x34, 0x29, 0x12, 0x24, 0x08, 0x11, 0x22, 0x04, 0x09, 0x13, 0x26, 0x0C, 0x19, 0x32, 0x25, 0x0A, 0x15, 0x2A, 0x14, 0x28, 0x10, 0x20].map(|v| {U6::new(v)});

    struct SYSTEMSTATE {
        PROGRAM_COUNTER: U6, //PC, shift register
        SUBROUTINE_RETURN: U6, //SR, storage register

        PAGE_ADDRESS: U4, //PA, storage register; contains 4-bit page address of rom instructions
        PAGE_BUFFER: U4, //PB storage register, used to set up page changes. also contains 4-bit return page address during call state

        CALL_LATCH: U1, //CL, latch, stores call state

        //RAM array
        //four files with 16 * U4 each
        RAM_ARRAY: [[U4, 4], 4],

        //R output register - single bit RAM cells, latches for output to R buffers. Used to control external devices, display scans, input encoding, status logic outputs. Can be strobed to scan a key matrix.

        X_REGISTER: U2, //X, storage register; ram page address
        Y_REGISTER: U4, //Y, storage register; ram word address and R output address

   //     WRITE_MUX_LOGIC: U2, //data selector; selects either constant and K inputs or acc for writing into ram, also performs bit setting & resetting
        // actually unnecessary

        CKI_LOGIC: U4, //CKI, data multiplexxer; selects either constant field, k input to enter cki data bus, or bit mask
        CKI_CONSTANT: U4,

        //au_select ; //data selector; selects destination of adder output to Y reg, acc, or neither

        STATUS: U1, //S, gates. conditional branch control. Normal state - 1. Branches are taken if S = 1. Selectively outputs a zero when carry is false or when logical compare is true. A zero lasts for one instruction cycle only.
        STATUS_LATCH: U1, //SL, latch, selectively stores status output. Transfers to O register w/ acc bits when TDO is executed
        ACCUMULATOR: U4, //A, storage register

        O_OUTPUT_REGISTER: U5, //O output register. Used to transmit data

        //external inputs: gates, input buffers. Performs page and PC override for initializing and hardware reset

        P_MUX_LOGIC: U4, //P-MUX: Data multiplexxer. Selects input to adder from RAM, CKI, or Y (0, 1, or 2)
        N_MUX_LOGIC: U5,//N-MUX: Data multiplexxer. Selects N input to adder (0) RAM, (1) CKI, (2) accumulator, (3) not-accumulator or (4) F16

        ADDER_INC: U1, //whether to increment the adder - set by C8 microinstruction and should be reset to 0 every cycle

    }

    struct SYSTEM {
        STATE: SYSTEMSTATE,
        //ROM array
        //Output PLA
        //Instruction decode PLA

    }

    //ROM PC decode
    //Page decode

    //RAM Y decode
    //RAM X decode

    //Adder/Comparator: Adds P input and N input with a possible carry. Logically compares P and N inputs too

    //Fixed instruction decoder

    impl SYSTEM {

        fn PAGE_RAM(&mut self) -> U4 {
            return self.STATE.RAM_ARRAY[self.STATE.X_REGISTER][self.STATE.Y_REGISTER];
        }

        fn WRITE_RAM(&mut self, U4 VALUE) {
            self.STATE.RAM_ARRAY[self.STATE.X_REGISTER][self.STATE.Y_REGISTER] = VALUE;
        }

        fn K_INPUT(&mut self) -> U4 {
            //todo
            //might need to page an external function to get current input values?
            //alternatively, there could be a systemstate value that's updated by an external handler
            //and K_input reads this state variable and then zeroes it
            //this way prevent race condition problems
        }

        fn CKI(&mut self) -> U4 {
            //selects either constant field, k input to enter cki data bus, or bit mask
            match self.STATE.CKI_LOGIC {
                0 => return self.STATE.CKI_CONSTANT,
                _ => return self.K_INPUT(),
               // _ => bitmask

            }

        }

        fn P_MUX(&mut self) -> U4 {
        //(0) RAM, (1) CKI, (2) Y register
            match state.N_MUX_LOGIC {
                0 => return self.PAGE_RAM(),
                1 => return self.CKI(),
                _ => return self.STATE.Y_REGISTER,
            }
        }

        fn N_MUX(&mut self) -> U4 {
        //(0) RAM, (1) CKI, (2) accumulator, (3) not-accumulator or (4) F16
            match state.P_MUX_LOGIC {
                0 => return self.PAGE_RAM(),
                1 => return self.CKI(),
                2 => return self.STATE.ACCUMULATOR,
                3 => return U4::new(1 + !(self.STATE.ACCUMULATOR.get())),
                _ => return U4::new(15),
            }
        }

        fn ADDER(&mut self) -> (U4, U1) {
            const VALUE: u8 = (self.P_MUX().get() + self.N_MUX().get() + self.STATE.ADDER_INC.get());
            return (U1::new(VALUE >> 4 & 1), U4::new(VALUE));
        }

        //Microinstructions

        //Fixed microinstructions

        fn BR (&mut self) {

        }

        fn CALL (&mut self) {

        }

        fn LDP (&mut self, U4 VALUE) {
            self.STATE.PAGE_BUFFER = VALUE.reverse(); //MSB on right

        }

        fn LDX(&mut self, U2 VALUE) {
            self.STATE.X_REGISTER = VALUE;
        }

        fn COMX (&mut self) {

        }

        fn TDO (&mut self) {

        }

        fn CLO (&mut self) {

        }

        fn SETR (&mut self) {

        }

        fn RSTR (&mut self) {

        }

        fn SBIT (&mut self) {

        }

        fn RBIT (&mut self) {

        }

        //Programmable microinstructions
        //P-MUX instructions

        fn CKP(&mut self, U4 VALUE) {
            self.STATE.P_MUX_LOGIC = U4::new(1);
        }

        fn YTP(&mut self) {
            self.STATE.P_MUX_LOGIC = U4::new(2);
        }

        fn MTP(&mut self) {
            self.STATE.P_MUX_LOGIC = U4::new(0);
        }

        //N-MUX instructions

        fn ATN(&mut self) {
            self.STATE.N_MUX_LOGIC = U5::new(2);
        }

        fn NATN(&mut self) {
            self.STATE.N_MUX_LOGIC = U5::new(3);
        }

        fn MTN(&mut self) {
            self.STATE.N_MUX_LOGIC = U5::new(0);
        }

        fn 15TN(&mut self) {
            self.STATE.N_MUX_LOGIC = U5::new(4);
        }

        fn CKN(&mut self) {
            self.STATE.N_MUX_LOGIC = U5::new(1);
        }

        //Adder/status instructions

        fn CIN(&mut self) {
            self.STATE.ADDER_INC = U1::new(1);
        }

        fn NE(&mut self) {
            if (self.N_MUX() == self.P_MUX()) {
                self.STATE.STATUS = U1::new(0);
            }
            else {
                self.STATE.STATUS = U1::new(1);
            }
        }

        fn C8(&mut self) {
            self.STATE.STATUS, _ = self.ADDER();
        }

        //Write MUX instructions

        fn STO(&mut self) {
            self.WRITE_RAM(self.STATE.ACCUMULATOR);
        }

        fn CKM(&mut self) {
            self.WRITE_RAM(self.CKI());
        }

        //AU Select/Status latch instructions

        fn AUTA(&myt self) {
            _, self.STATE.ACCUMULATOR = self.ADDER();
        }

        fn AUTY(&mut self) {
            _, self.STATE.Y_REGISTER = self.ADDER();
        }

        fn STSL(&mut self) {
            self.STATE.STATUS_LATCH = self.STATE.STATUS;
        }

    }

}


fn main() {


}





//Powerup:
//set PC to location zero
//set PA to 15
//
