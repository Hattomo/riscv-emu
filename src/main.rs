pub mod cpu;
pub mod bus;
pub mod memory;

use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use cpu::*;
use crate::bus::MEMORY_BASE;

fn main() -> io::Result<()> {
    println!("Hello,RISC-V Emulator!");
    // get data from command line & get cammand length 
    let args: Vec<String> = env::args().collect();

    // check command line length of argments
    // if not length is 2, cammnd is wrong
    if args.len() != 2 {
        panic!("Usage: riscvemu <file name>");
    }

    // read file
    let mut file = File::open(&args[1])?;
    let mut binary = Vec::new();
    file.read_to_end(&mut binary)?;
    // set up Cpu & set binary read from file to cpu memory
    let mut cpu = Cpu::new(binary);

    while cpu.pc - MEMORY_BASE < cpu.codesize {
        // fetch
        // break when error occur
        let inst = match cpu.fetch(){
            Ok(inst) => inst,
            Err(_) => break,
        };

        // add 4 to the programm counter
        cpu.pc += 4;

        // decode & execute
        // break when error occur
        match cpu.execute(inst){
            Ok(_) => {},
            Err(_) => break,
        };

        // not to infine loop
        if cpu.pc == 0 {
            break;
        }
    }

    // print reg (x0 to x31)
    cpu.dump_registers();

    Ok(())
}
