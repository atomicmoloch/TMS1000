Kavi Wilson
Advisor: Erica Blum

project.rs:
The code emulates a Texas Instruments TMS1000 microcontroller, first introduced in 1974.

I attempted to adapt something of a functional programming paradigm: Everything that changes in the course of running a program is implemented in the SYTEMSTATE object. The loaded ROM and PLAs are in the SYTEM object, while everything that is fixed in the microcontroller are implemented on the SYSTEM object, modifying the SYTEMSTATE.
Does not presently compile; have been rapidly prototyping architechture.

decompiler.rs:
The code decompiles binary ROM dumps from TMS1000 microcontrollers into assembly source code.

This is a somewhat simplified version of how the instruction decoder in the TMS1000 works - since in the full microcontroller, all instructions except for BR (branch), CALL (call), RETN (return), LDP (load page buffer), LDX (load x register), COMX (complement X), TDO (transfer to O outputs), CLO (clear o-outputs), SETR (set r outputs), RSTR (reset r outputs), SBIT (set memory bit), and RBIT (reset memory bit) are composed of a series of microinstructions as decided by a programmable logic array, and can be reprogrammed to execute any subset of microinstructions. The o-outputs too are outputted through a programmable logic array, so a complete system configuration must include both of these PLAs. However, many (possibly most) applications did utilize the "standard instruction decode" which is the basis of the opcode definitions found in the Programmer's Reference Manual - the PLA for this instruction decoder is represented as figure 2-17.2 in the manual. Die shots have confirmed (https://seanriddle.com/simon.html) that Simon units used the standard instruction decoding PLA.

The TMS1000 has 8192 bits of RAM; this is divided into 16 'pages' of 64 8-bit words. The word that is loaded on the current page is selected by a 6-bit program counter register; the current page is selected by a 4 bit page address register. Pages can be switched by loading a constant into the page buffer register using the LDP instruction, then by executing a BR (branch) or CALL instruction with the status flag equal to one, which moves the page buffer register into the page address (in the case of BR) or switches the page buffer and page address registers (in the case of CALL). Both instructions also load a constant into the program counter register.

The status latch 'defaults' to 1, and is only set to zero if a arithmetic instruction is called and does not carry, or if a comparison instructon fails. If an instruction does not explicitly modify the status latch, it always sets it to 1. Thus, the status result in effect only persists for one instruction cycle without special handling.

Program execution is nonlinear - in addition to the fact that CALL and BR work like arbitrary goto instructions, the program counter follows a pseudo-random pattern (the variable PC_SEQ in the decoder). Thus, for example, if the current instruction is the third word in the loaded ROM, the next executed instruction will be the seventh world in the ROM (barring a CALL or BR statement which can arbitrarily set the program counter). The decoder rearranges the instructions in execution order, rather than in the order that instructions appear in the ROM. The output format is [Execution order] - [Page number] : [Actual ROM location] [Instruction] [Argument (if applicable)]. The execution order number is indicated in parenthesis after the actual ROM address in the arguments of BR and CALL statements.

Upon initialization, the page buffer and address registers are set to 15, and the program counter is set to 0. Thus, execution of the stored program begins on the first instruction of the last page of ROM.

I have further added the TMS1000-series Programmer's Reference Manual in pdf form, the outputted source code of Simon for the TMS1000, and the beginnings of a detailed explication of the source code, to the github repository
