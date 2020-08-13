// Defalt memory size(128MB)
pub const MEMORY_SIZE: u64 = 1024 * 1024 * 128;
pub const REGISTER_NUMBER: usize = 32;

// CPU
// CPU does not have memory inside CPU.
// it connent via system bus. but now, to simplify, it's OK
pub struct Cpu{
    pub regs:[u64; REGISTER_NUMBER], //register 64bit & 32 registers
    pub pc:u64, // programm counter
    pub memory:Vec<u8>,
}

impl Cpu{
    pub fn new(binary: Vec<u8>) -> Self {
        let mut regs = [0; REGISTER_NUMBER];
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
    pub fn fetch(&self) -> u32{
        let index = self.pc as usize;
        return (self.memory[index] as u32)
            | ((self.memory[index + 1] as u32) << 8)
            | ((self.memory[index + 2] as u32) << 16)
            | ((self.memory[index + 3] as u32) << 24);
    }

    pub fn execute(&mut self, inst:u32){
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
    pub fn dump_registers(&self){
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