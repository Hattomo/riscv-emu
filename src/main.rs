pub mod cpu;

use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use cpu::*;

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

    while cpu.pc < cpu.memory.len() as u64 {
        // fetch
        let inst = cpu.fetch();
        // add 4 to the programm counter
        cpu.pc += 4;
        
        // decode & execute
        match cpu.execute(inst){
            true => break,
            false => {}
        };

        if cpu.pc == 0 {
            break;
        }
    }

    // print reg (x0 to x31)
    cpu.dump_registers();

    Ok(())
}
