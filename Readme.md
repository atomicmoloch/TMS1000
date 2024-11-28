Kavi Wilson
Advisor: Erica Blum

project.rs:
The code emulates a Texas Instruments TMS1000 microcontroller, first introduced in 1974.

I attempted to adapt something of a functional programming paradigm: Everything that changes in the course of running a program is implemented in the SYTEMSTATE object. The loaded ROM and PLAs are in the SYTEM object, while everything that is fixed in the microcontroller are implemented on the SYSTEM object, modifying the SYTEMSTATE.
Does not presently compile; have been rapidly prototyping architechture.

decompiler.rs:
The code decompiles binary ROM dumps from TMS1000 microcontrollers into assembly source code
If you are reading this, I am about to resubmit the thesis check-in assignment.

