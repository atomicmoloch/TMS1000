Running

cargo build --bins --release

will create four binary files:

decompile, which will take a TMS 1000-family version number and an input file of TMS 1000 machine code, and will decode it back into instruction mnemonics, and dump it to stdout. It will additionally reorganise the file into execution order, instead of the TMS 1000's pseudorandom ordering.

compile, which takes an input in the form of a TMS 1000 version number and a text file in the format of the decompile output, and 'compiles' it back into bytecode.

speedtest, which is a primative speedtest of the emulator core.

and finally, tms, which functions as a somewhat GDB like debugger utility, allowing TMS 1000 programs to be stepped through, and the system state observed.
