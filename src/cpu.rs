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
        let funct3 = (inst & 0x00007000) >> 12;
        let funct7 = (inst & 0xfe000000) >> 25;
        let rd = ((inst & 0x00000f80) >> 7 ) as usize;
        let rs1 = ((inst & 0x000f8000) >> 15 ) as usize;
        let rs2 = ((inst & 0x01f00000) >> 20 ) as usize;

        // regs[0](x0) is always 0 (hardwired)
        self.regs[0] = 0;

        // exec 
        // wrapping_add & wrapping_sub ignore overflow
        match opcode {
            0x03 => {
                // imm[11:0] = inst[31:20]
                let imm = ((inst as i32 as i64) >> 20) as u64;
                let addr = self.regs[rs1].wrapping_add(imm);
                match funct3 {
                    0x0 => {
                        // lb
                        // 🍫 what is sign-extends ?
                        // Loads a 8 bits value from memory 
                        // and sign-extends this to XLEN bits 
                        // before string it in register rd.
                        // finally store it register rd.
                        let val = self.read8(addr);
                        self.regs[rd] = val as i8 as i64 as u64;
                    }
                    0x1 => {
                        // lh
                        // // Loads a 16 bits value from memory 
                        // and sign-extends this to XLEN bits 
                        // before string it in register rd.
                        // finally store it register rd.
                        let val = self.read32(addr);
                        self.regs[rd] = val as i16 as i64 as u64;
                    }
                    0x02 => {
                        // lw
                        // Loads a 32 bits value from memory 
                        // and sign-extends this to XLEN bits 
                        // before string it in register rd.
                        // finally store it register rd.
                        let val = self.read16(addr);
                        self.regs[rd] = val as i32 as i64 as u64;
                    }
                    0x03 => {
                        // ld
                        // Loads a 64-bit value from memory 
                        // into register rd for RV64I.
                        let val = self.read64(addr);
                        self.regs[rd] = val;
                    }
                    0x04 => {
                        // lbu
                        // Loads a 8 bits value from memory 
                        // and zero-extends this to XLEN bits 
                        // before string it in register rd.
                        // finally store it register rd.
                        let val = self.read8(addr);
                        self.regs[rd] = val;
                    }
                    0x05 => {
                        // lhu
                        // Loads a 16 bits value from memory 
                        // and zero-extends this to XLEN bits 
                        // before string it in register rd.
                        // finally store it register rd.
                        let val = self.read16(addr);
                        self.regs[rd] = val;
                    }
                    0x06 => {
                        // lwu
                        // Loads a 32 bits value from memory 
                        // and zero-extends this to XLEN bits 
                        // before string it in register rd.
                        // finally store it register rd.
                        let val = self.read32(addr);
                        self.regs[rd] = val;
                    }
                    _ => {}
                }
            }
            // addi
            // rd = rs1 + imm
            0x13 => {
                let imm = ((inst & 0xfff00000) as i32 as i64 >> 20 ) as u64;
                self.regs[rd] = self.regs[rs1].wrapping_add(imm);
            }
            // auipc
            // 🍫 add upper immediate to pc
            0x17 => {
                let imm = (inst & 0xfffff000) as i32 as i64 as u64;
                self.regs[rd] = self.pc.wrapping_add(imm).wrapping_sub(4);
            }
            0x1b => {
                // imm[11:0] = inst[31:20]
                let imm = ((inst as i32 as i64) >> 20) as u64;
                let shamt = (imm & 0x1f) as u32;
                match funct3 {
                    0x0 => {
                        // addiw
                        // Add word immediate.
                        self.regs[rd] = self.regs[rs1].wrapping_add(imm) as i32 as i64 as u64;
                    }
                    0x1 => {
                        // slliw
                        // 🍫 Shift left logical word immdiate
                        self.regs[rd] = self.regs[rs1].wrapping_shl(shamt) as i32 as i64 as u64;
                    }
                    0x5 => {
                        match funct7 {
                            0x00 => {
                                // srliw
                                // 🍫 Shift right logical word immediate
                                self.regs[rd] = self.regs[rs1].wrapping_shr(shamt) as i32 as i64 as u64;
                            }
                            0x20 => {
                                // sraiw
                                // 🍫 Shift right arithmetic word immediate.
                                self.regs[rd] = (self.regs[rs1] as i32).wrapping_shr(shamt) as i64 as u64;
                            }
                            _=>{}
                        }
                    }
                    _ => {}
                }
            }
            0x23 => {
                    // imm[11:5|4:0] = inst[31:25|11:7]
                    let imm = (((inst & 0xfe000000) as i32 as i64 >> 20) as u64)
                        | (((inst >> 7) & 0x1f) as u64);
                    let addr = self.regs[rs1].wrapping_add(imm);
                    match funct3 {
                        // sb 
                        // Store 8-bit, values from the low bits of register rs2 to memory.
                        0x0 => self.write8(addr,self.regs[rs2]), 
                        // sh
                        // Store 16-bit, values from the low bits of register rs2 to memory.
                        0x1 => self.write16(addr,self.regs[rs2]),
                        // sw
                        // Store 32-bit, values from the low bits of register rs2 to memory.
                        0x2 => self.write32(addr,self.regs[rs2]),
                        // sd
                        // Store 64-bit, values from the low bits of register rs2 to memory.
                        0x3 => self.write64(addr,self.regs[rs2]),
                        _ => {}
                    }
            }
            // add 
            // rd = rs1 + rs2
            0x33 => {
                self.regs[rd] = self.regs[rs1].wrapping_add(self.regs[rs2]);
            }
            0x37 => {
                // lui
                // 🍫 Load upper immediate value.
                self.regs[rd] = (inst & 0xfffff000) as i32 as i64 as u64;
            }
            _=> {
                dbg!(format!("not implemented yet: opcade {:#x}",opcode));
            }
        }
    }


    // read 1 bite from the littele-endian memory
    fn read8(&self, addr:u64) -> u64 {
        self.memory[addr as usize] as u64
    }

    // read 2 bites from the littele-endian memory
    fn read16(&self, addr:u64) -> u64 {
        let index = addr as usize;
        return (self.memory[index] as u64) 
            | ((self.memory[index + 1] as u64) << 8);
    }

    //read 4 bites from the littele-endian memory
    fn read32(&self, addr:u64) -> u64 {
        let index = addr as usize;
        return (self.memory[index] as u64)
            | ((self.memory[index + 1] as u64) << 8)
            | ((self.memory[index + 2] as u64) << 16)
            | ((self.memory[index + 3] as u64) << 24);
    }

    // read 8 bites from the little-endian memory
    fn read64(&self, addr: u64) -> u64 {
        let index = addr as usize;
        return (self.memory[index] as u64) 
            | ((self.memory[index + 1] as u64) << 8)
            | ((self.memory[index + 2] as u64) << 16)
            | ((self.memory[index + 3] as u64) << 24)
            | ((self.memory[index + 4] as u64) << 32)
            | ((self.memory[index + 5] as u64) << 40)
            | ((self.memory[index + 6] as u64) << 48)
            | ((self.memory[index + 2] as u64) << 56);
    }

    // 🍫 val & 0xff ?
    // write 1 bite to the little-endian memory
    fn write8(&mut self, addr:u64, val:u64){
        let index = addr as usize;
        self.memory[index] = val as u8;
    }

    // write 2 bites to the little-endian memory
    fn write16(&mut self, addr:u64, val:u64){
        let index = addr as usize;
        self.memory[index] = (val & 0xff) as u8;
        self.memory[index + 1] = ((val >> 8) & 0xff) as u8;
    }

    // write 4 bites to the little-endian memory
    fn write32(&mut self, addr:u64, val:u64){
        let index = addr as usize;
        self.memory[index] = (val & 0xff) as u8;
        self.memory[index + 1] = ((val >> 8) & 0xff) as u8;
        self.memory[index + 2] = ((val >> 16) & 0xff) as u8;
        self.memory[index + 3] = ((val >> 24) & 0xff) as u8;
    }

    // write 8 bites to the little-endian memory
    fn write64(&mut self,addr:u64, val:u64){
        let index = addr as usize;
        self.memory[index] = (val & 0xff) as u8;
        self.memory[index + 1] = ((val >> 8) & 0xff) as u8;
        self.memory[index + 2] = ((val >> 16) & 0xff) as u8;
        self.memory[index + 3] = ((val >> 24) & 0xff) as u8;
        self.memory[index + 4] = ((val >> 32) & 0xff) as u8;
        self.memory[index + 5] = ((val >> 40) & 0xff) as u8;
        self.memory[index + 6] = ((val >> 48) & 0xff) as u8;
        self.memory[index + 7] = ((val >> 56) & 0xff) as u8;
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