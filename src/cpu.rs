use crate::bus::*;

// Defalt memory size(128MB)
pub const MEMORY_SIZE: u64 = 1024 * 1024 * 128;
pub const REGISTER_NUMBER: usize = 32;

// CPU
// CPU does not have memory inside CPU.
// it connent via system bus. but now, to simplify, it's OK
pub struct Cpu{
    //register 64bit & 32 registers
    pub regs:[u64; REGISTER_NUMBER], 
    // programm counter
    pub pc:u64, 
    // System Bus
    pub bus: Bus,
    // The size of binary
    pub codesize: u64,
}

impl Cpu{
    pub fn new(binary: Vec<u8>) -> Self {
        let mut regs = [0; REGISTER_NUMBER];
        // regs[2](x2) is a stack pointer
        regs[2] = MEMORY_BASE + MEMORY_SIZE;

        let codesize = binary.len() as u64;

        Self {
            regs,
            pc: MEMORY_BASE,
            bus: Bus::new(binary),
            codesize,
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
    
    // Get an instruction from memory
    // | means OR in Rust
    // this is a little-endian system
    pub fn fetch(&self) -> Result<u64,()>{
        return self.bus.load(self.pc,32);
    }

    //  Return true if an error happens, otherwise false.
    pub fn execute(&mut self, inst:u64) -> Result<(),()>{
        // decode 
        // get opcode,rd,rs1,rs2
        //let inst = inst as u64;
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
        println!("{}:{}",opcode,funct3);
        //println!("{}",inst);
        match opcode {
            0x03 => {
                // imm[11:0] = inst[31:20]
                let imm = ((inst as i32 as i64) >> 20) as u64;
                let addr = self.regs[rs1].wrapping_add(imm);
                println!("{}:{}:{}:{}",opcode,funct3,addr,imm);
                match funct3 {
                    0x0 => {
                        // lb
                        // 🍫 what is sign-extends ?
                        // Loads a 8 bits value from memory 
                        // and sign-extends this to XLEN bits 
                        // before string it in register rd.
                        // finally store it register rd.
                        let val = self.bus.load(addr,8)?;
                        self.regs[rd] = val as i8 as i64 as u64;
                    }
                    0x1 => {
                        // lh
                        // // Loads a 16 bits value from memory 
                        // and sign-extends this to XLEN bits 
                        // before string it in register rd.
                        // finally store it register rd.
                        let val = self.bus.load(addr,16)?;
                        self.regs[rd] = val as i16 as i64 as u64;
                    }
                    0x2 => {
                        // lw
                        // Loads a 32 bits value from memory 
                        // and sign-extends this to XLEN bits 
                        // before string it in register rd.
                        // finally store it register rd.
                        let val = self.bus.load(addr,32)?;
                        self.regs[rd] = val as i32 as i64 as u64;
                    }
                    0x3 => {
                        // ld
                        // Loads a 64-bit value from memory 
                        // into register rd for RV64I.
                        let val = self.bus.load(addr,64)?;
                        self.regs[rd] = val;
                    }
                    0x4 => {
                        // lbu
                        // Loads a 8 bits value from memory 
                        // and zero-extends this to XLEN bits 
                        // before string it in register rd.
                        // finally store it register rd.
                        let val = self.bus.load(addr,8)?;
                        self.regs[rd] = val;
                    }
                    0x5 => {
                        // lhu
                        // Loads a 16 bits value from memory 
                        // and zero-extends this to XLEN bits 
                        // before string it in register rd.
                        // finally store it register rd.
                        let val = self.bus.load(addr,16)?;
                        self.regs[rd] = val;
                    }
                    0x6 => {
                        // lwu
                        // Loads a 32 bits value from memory 
                        // and zero-extends this to XLEN bits 
                        // before string it in register rd.
                        // finally store it register rd.
                        let val = self.bus.load(addr,32)?;
                        self.regs[rd] = val;
                    }
                    _ => {}
                }
            }
            0x13 => {
                // imm[11:0] = inst[31:20]
                let imm = ((inst & 0xfff00000) as i32 as i64 >> 20 ) as u64;
                // The shift amount is encoded in the lower 6 bits of the I-immediate field for RV64I.
                let shamt = (imm & 0x3f) as u32;
                match funct3 {
                    0x0 => {
                        // addi
                        // rd = rs1 + imm
                        self.regs[rd] = self.regs[rs1].wrapping_add(imm);
                    }
                    0x1 => {
                        // slli
                        // Shift left logical immediate.
                        self.regs[rd] = self.regs[rs1] << shamt;
                    }
                    0x2 => {
                        // slti
                        // Set if less than.
                        self.regs[rd] = if(self.regs[rs1] as i64) < (imm as i64){
                            1
                        }else{
                            0
                        };
                    }
                    0x3 => {
                        // sltiu
                        // Set if less than, unsigned.
                        self.regs[rd] = if self.regs[rs1] < imm {
                            1
                        }else{
                            0
                        };
                    }
                    0x4 => {
                        // xori
                        // Exclusive OR immediate.
                        self.regs[rd] = self.regs[rs1] ^ imm;
                    }
                    0x5 => {
                        match funct7 >> 1 {
                            // srli
                            // Shift right logical immediate.
                            0x00 => {
                                self.regs[rd] = self.regs[rs1].wrapping_shr(shamt);
                            }
                            // srai
                            // Shift right arithmetic immediate.
                            0x10 => {
                                self.regs[rd] = (self.regs[rs1] as i64).wrapping_shr(shamt) as u64;
                            }
                            _ => {}
                        }
                    }
                    // ori
                    // or immediate
                    0x6 => {
                        self.regs[rd] = self.regs[rs1] | imm;
                    }
                    // andi
                    // and immediate
                    0x7 => {
                        self.regs[rd] = self.regs[rs1] & imm;
                    }
                    _ => {}
                }
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
                        | (((inst >> 7) & 0x1f));
                    let addr = self.regs[rs1].wrapping_add(imm);
                    match funct3 {
                        // sb 
                        // Store 8-bit, values from the low bits of register rs2 to memory.
                        0x0 => self.bus.store(addr,8,self.regs[rs2])?, 
                        // sh
                        // Store 16-bit, values from the low bits of register rs2 to memory.
                        0x1 => self.bus.store(addr,16,self.regs[rs2])?,
                        // sw
                        // Store 32-bit, values from the low bits of register rs2 to memory.
                        0x2 => self.bus.store(addr,32,self.regs[rs2])?,
                        // sd
                        // Store 64-bit, values from the low bits of register rs2 to memory.
                        0x3 => self.bus.store(addr,64,self.regs[rs2])?,
                        _ => {}
                    }
            }
            0x33 => {
                let shamt = ((self.regs[rs2] & 0x3f) as u64) as u32;
                match (funct3, funct7) {
                    (0x0,0x00) => {
                        // add 
                        // rd = rs1 + rs2
                        self.regs[rd] = self.regs[rs1].wrapping_add(self.regs[rs2]);
                    }
                    (0x0,0x20) => {
                        // sub
                        // rd = rs1 - rs2
                        self.regs[rd] = self.regs[rs1].wrapping_sub(self.regs[rs2]);
                    }
                    (0x1,0x00) => {
                        // sll
                        // shift left logical
                        // rd = rs1 << rs2 ,pc++
                        self.regs[rd] = self.regs[rs1].wrapping_shl(shamt);
                    }
                    (0x2,0x00) => {
                        // slt
                        // rd = (rs1 < rs2) ? 1:0 ,pc++
                        self.regs[rd] = if(self.regs[rs1] as i64) < (self.regs[rs2] as i64) {
                            1
                        }else{
                            0
                        };
                    }
                    (0x3,0x00) => {
                        // sltu
                        // Set if less than, unsigned.
                        self.regs[rd] = if self.regs[rs1] < self.regs[rs2]{
                            1
                        }else{
                            0
                        };
                    }
                    (0x4, 0x00) => {
                        // xor
                        self.regs[rd] = self.regs[rs1] ^ self.regs[rs2];
                    }
                    (0x5,0x00) => {
                        // srl
                        // Shift right logical.
                        self.regs[rd] = self.regs[rs1].wrapping_shr(shamt);
                    }
                    (0x5,0x20) => {
                        // sra
                        // Shift right arithmetic.
                        self.regs[rd] = (self.regs[rs1] as i64).wrapping_shr(shamt) as u64;
                    }
                    (0x06,0x00) => {
                        // or
                        self.regs[rd] = self.regs[rs1] | self.regs[rs2];
                    }
                    (0x7,0x00) => {
                        // and
                        self.regs[rd] = self.regs[rs1] & self.regs[rs2];
                    }
                    _ => {}
                }
            }
            0x37 => {
                // lui
                // 🍫 Load upper immediate value.
                self.regs[rd] = (inst & 0xfffff000) as i32 as i64 as u64;
            }
            0x3b => {
                // The shift amount is given by rs2[4:0]
                let shamt = (self.regs[rs2] & 0x1f) as u32;
                match (funct3,funct7) {
                    (0x0, 0x00) => {
                        // addw
                        // Adds the 32-bit of registers rs1 and 32-bit of register rs2 and stores the result in rd.
                        self.regs[rd] = self.regs[rs1].wrapping_add(self.regs[rs2]) as i32 as i64 as u64;
                    }
                    (0x0, 0x20) => {
                        // subw
                        // Subtract the 32-bit of registers rs1 and 32-bit of register rs2 and stores the result in rd.
                        self.regs[rd] = ((self.regs[rs1].wrapping_sub(self.regs[rs2])) as i32) as u64;
                    }
                    (0x1, 0x00) => {
                        // sllw
                        // Shift left logical word.
                        self.regs[rd] = (self.regs[rs1] as u32).wrapping_shl(shamt) as i32 as u64;
                    }
                    (0x5, 0x00) => {
                        // srlw
                        // Shift right logical word.
                        self.regs[rd] = (self.regs[rs1] as u32).wrapping_shr(shamt) as i32 as u64;
                    }
                    (0x5, 0x20) => {
                        // sraw
                        // Shift right arithmetic word.
                        // 🍫 what is arithmetic ?
                        self.regs[rd] = ((self.regs[rs1] as i32) >> (shamt as i32)) as u64;
                    }
                    _ => {}
                }
            }
            0x63 => {
                // imm[12|10:5|4:1|11] = inst[31|30:25|11:8|7]
                let imm = (((inst & 0x80000000) as i32 as i64 >> 19) as u64)
                    | ((inst & 0x80) << 4) as u64 // imm[11]
                    | ((inst >> 20) & 0x7e0) as u64 // imm[10:5]
                    | ((inst >> 7) & 0x1e) as u64; // imm[4:1]

                match funct3 {
                    0x0 => {
                        // beq
                        // Take the branch if registers rs1 and rs2 are equal.
                        if self.regs[rs1] == self.regs[rs2]{
                            self.pc = self.pc.wrapping_add(imm).wrapping_sub(4);
                        }
                    }
                    0x1 => {
                        // bne
                        // Take the branch if registers rs1 and rs2 are not equal.
                        if self.regs[rs1] != self.regs[rs2]{
                            self.pc = self.pc.wrapping_add(imm).wrapping_sub(4);
                        }
                    }
                    0x4 => {
                        // blt
                        // Take the branch if registers rs1 is less than rs2, 
                        // using signed comparison.
                        if (self.regs[rs1] as i64) < (self.regs[rs2] as i64) {
                            self.pc = self.pc.wrapping_add(imm).wrapping_sub(4);
                        }
                    }
                    0x5 => {
                        // bge
                        // Take the branch if registers rs1 is greater than rs2, 
                        // using signed comparison.
                        if (self.regs[rs1] as i64) >= (self.regs[rs2] as i64) {
                            self.pc = self.pc.wrapping_add(imm).wrapping_sub(4);
                        }
                    }
                    0x6 => {
                        // bltu
                        // Take the branch if registers rs1 is less than rs2, 
                        // using unsigned comparison.
                        if self.regs[rs1] < self.regs[rs2]{
                            self.pc = self.pc.wrapping_add(imm).wrapping_sub(4);
                        }
                    }
                    0x7 => {
                        // bgeu
                        // Take the branch if registers rs1 is greater than rs2,
                        // using unsigned comparison.
                        if self.regs[rs1] >= self.regs[rs2]{
                            self.pc = self.pc.wrapping_add(imm).wrapping_sub(4);
                        }
                    }
                    _ => {}
                }
            }
            // jalr
            // jump and link register
            0x67 => {
                let t = self.pc;

                let imm = (((inst & 0xfff00000) as i64) >> 20) as u64;
                self.pc = (self.regs[rs1].wrapping_add(imm)) & !1;

                // 🍫
                self.regs[rd] = t;
            }
            // jal
            // jump and link
            0x6f => {
                self.regs[rd] = self.pc;

                // imm[20|10:1|11|19:12] = inst[31|30:21|20|19:12]
                let imm = (((inst & 0x80000000) as i32 as i64 >> 11) as u64) // imm[20]
                    | (inst & 0xff000) as u64 // imm[19:12]
                    | ((inst >> 9) & 0x800) as u64 // imm[11]
                    | ((inst >> 20) & 0x7fe) as u64; // imm[10:1]

                self.pc = self.pc.wrapping_add(imm).wrapping_sub(4);
            }
            _ => {
                dbg!(format!("not implemented yet: opcade {:#x}",opcode));
                return Err(());
            }
        }
        return Ok(());
    }
}