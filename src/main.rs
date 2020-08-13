use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;

// Defalt memory size(128MB)
pub const MEMORY_SIZE: u64= 1024 * 1024 * 128;

// CPU
// CPU does not have memory inside CPU.
// it connent via system bus. but now, to simplify, it's OK
struct Cpu{
    regs:[u64;32], //register 64bit & 32 registers
    pc:u64, // programm counter
    memory:Vec<u8>,
}

impl Cpu{
    fn new(binary: Vec<u8>) -> Self {
        let mut regs = [0; 32];
        // regs[2](x2) is a stack pointer
        regs[2] = MEMORY_SIZE;

        Self {
            regs,
            pc: 0,
            memory:binary,
        }
    }

    // Get an instruction from memory
    // | means OR in Rust
    // this is a little-endian system
    fn fetch(&self) -> u32{
        let index = self.pc as usize;
        return (self.memory[index] as u32)
            | ((self.memory[index + 1] as u32) << 8)
            | ((self.memory[index + 2] as u32) << 16)
            | ((self.memory[index + 3] as u32) << 24);
    }

    fn execute(&mut self, inst:u32){
        // decode 
        // get opcode,rd,rs1,rs2
        let opcode = inst & 0x0000007f;
        let rd = ((inst & 0x00000f80) >> 7 ) as usize;
        let rs1 = ((inst & 0x000f8000) >> 15 ) as usize;
        let rs2 = ((inst & 0x01f00000) >> 20 ) as usize;

        // regs[0](x0) is always 0 (hardwired)
        self.regs[0] = 0;

        // exec 
        // wrapping_add & wrapping_sub ignore overflow
        match opcode {
            // addi
            // rd = rs1 + imm
            0x13 => {
                let imm = ((inst & 0xfff00000) as i32 as i64 >> 20 ) as u64;
                self.regs[rd] = self.regs[rs1].wrapping_add(imm);
            }
            // add 
            // rd = rs1 + rs2
            0x33 => {
                self.regs[rd] = self.regs[rs1].wrapping_add(self.regs[rs2]);
            }
            _=> {
                dbg!(format!("not implemented yet: opcade {:#x}",opcode));
            }
        }
    }

    // print values in all registers (x0-x31)
    fn dump_registers(&self){
        let mut output = String::from("");
        for i in (0..32).step_by(4){
            output = format!(
                "{}\n{}",
                output,
                format!("x{:02}={:>#18x} x{:02}={:>#18x} x{:02}={:>#18x} x{:02}={:>#18x}",
                i,
                self.regs[i],
                i + 1,
                self.regs[i + 1],
                i + 2,
                self.regs[i + 2],
                i + 3,
                self.regs[i + 3],
                )
            );
        }
        println!("{}",output);
    }
}



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
        cpu.execute(inst);
    }

    // print reg (x0 to x31)
    cpu.dump_registers();

    Ok(())
}
