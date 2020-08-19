use crate::bus::*;
use crate::trap::*;
use crate::uart::*;
use crate::virtio::*;
use crate::plic::*;
use crate::memory::*;

// Defalt memory size(128MB)
pub const MEMORY_SIZE: u64 = 1024 * 1024 * 128;
pub const REGISTER_NUMBER: usize = 32;
// CSRs 2^12
pub const CSRS_NUMBER: usize = 4096;

// CRSs
// Machine level CSRs
// Machine information register
// Hardware thread ID
pub const MHARTID: usize = 0xf14;
// Machine trap setup
// Machine status register
pub const MSTATUS: usize = 0x300;
// Machine exception delefation register
pub const MEDELEG: usize = 0x302;
// Machine interrupt-enable register
pub const MIE: usize = 0x304;
// Machine trap-handler base address.
pub const MTVEC: usize = 0x305;
// Machine conuter enable 
pub const MCOUNTEREN: usize = 0x306;
// Machine trap handling
// Scratch register for machine trap handlers
pub const MSCRATCH: usize = 0x340;
// Machine exception program counter
pub const MEPC: usize = 0x341;
// Machine trap cause
pub const MCAUSE: usize = 0x342;
// Machine bad address or instruction
pub const MTVAL: usize = 0x343;

// MIP fields.
pub const MIP_SSIP: u64 = 1 << 1;
pub const MIP_MSIP: u64 = 1 << 3;
pub const MIP_STIP: u64 = 1 << 5;
pub const MIP_MTIP: u64 = 1 << 7;
pub const MIP_SEIP: u64 = 1 << 9;
pub const MIP_MEIP: u64 = 1 << 11;

// Surpervisor-level CSRs
// Surpervisor status register
// Surpervisor trap setup
pub const SSTATUS: usize = 0x100;
// Surpervisor interrupt-enable register
pub const SIE:usize = 0x104;
// Surpervisor trap hander base address
pub const STVEC: usize = 0x105;
// Surpervisor trap handling
// Scratch register for surpervisor trap hander
pub const SSCRACH: usize = 0x140;
// Surpervisor exception program counter
pub const SEPC: usize = 0x141;
// Surpervisor trap cause
pub const SCAUSE: usize = 0x142;
// Surpervisor bad address or instruction
pub const STVAL: usize = 0x143;
// Surpervisor interrupt pending
pub const SIP: usize = 0x144;
// Surpervisor address translation and protection
pub const SATP: usize = 0x180;

// The CPU mode
#[derive(Debug, PartialEq, PartialOrd, Eq, Copy, Clone)]
pub enum Mode {
    // user mode (application)
    User = 0b00,
    // surpervisor mode (kernel,OS)
    Surpervisor = 0b01,
    // hypervisor mode (not now)
    // Hypervisor = 0b10,
    // Everything ?
    Machine = 0b11,
}

// CPU
// it connent via system bus
pub struct Cpu{
    //register 64bit & 32 registers
    pub regs:[u64; REGISTER_NUMBER], 
    // programm counter
    pub pc:u64, 
    // System Bus
    pub bus: Bus,
    // Control and status registers. RISC-V ISA sets 
    // aside a 12-bit encoding space (csr[11:0]) for
    // up to 4096 CSRs.
    pub csrs: [u64; CSRS_NUMBER],
    // Privilege mode
    pub mode : Mode,
}

impl Cpu{
    pub fn new(binary: Vec<u8>) -> Self {
        let mut regs = [0; REGISTER_NUMBER];
        // regs[2](x2) is a stack pointer
        regs[2] = MEMORY_BASE + MEMORY_SIZE;

        Self {
            regs,
            pc: MEMORY_BASE,
            bus: Bus::new(binary),
            csrs: [0; CSRS_NUMBER],
            mode:Mode::Machine,
        }
    }

    // print values in all registers (x0-x31)
    pub fn dump_registers(&self){
        let mut output = String::from("");
        let abi = [
            "zero", " ra ", " sp ", " gp ", " tp ", " t0 ", " t1 ", " t2 ",
            " s0 ", " s1 ", " a0 ", " a1 ", " a2 ", " a3 ", " a4 ", " a5 ", " a6 ",
            " a7 ", " s2 ", " s3 ", " s4 ", " s5 ", " s6 ", " s7 ", " s8 ", " s9 ",
            "s10 ", "s11 ", " t3 ", " t4 ", " t5 ", " t6 "
        ];
        for i in (0..32).step_by(4){
            output = format!(
                "{}\n{}",
                output,
                format!(
                "x{:02}({})={:>#18x} x{:02}({})={:>#18x} x{:02}({})={:>#18x} x{:02}({})={:>#18x}",
                i,
                abi[i],
                self.regs[i],
                i + 1,
                abi[i + 1],
                self.regs[i + 1],
                i + 2,
                abi[i + 2],
                self.regs[i + 2],
                i + 3,
                abi[i + 3],
                self.regs[i + 3],
                )
            );
        }
        println!("{}",output);
    }
    
    // Print values in some csrs
    pub fn dump_csrs(&self){
        let output = format!(
            "{}\n{}",
            format!(
                "mstatus={:>#18x} mtvec={:>#18x} mepc={:>#18x} mcause={:>#18x}",
                self.csrs[MSTATUS], self.csrs[MTVEC], self.csrs[MEPC], self.csrs[MCAUSE],
            ),
            format!(
                "sstatus={:>#18x} stvec={:>#18x} sepc={:>#18x} scause={:>#18x}",
                self.csrs[SSTATUS], self.csrs[STVEC], self.csrs[SEPC], self.csrs[SCAUSE],
            ),
        );
        println!("{}", output);
    }

    pub fn check_pending_interrupt(&mut self) -> Option<Interrupt> {
        // 3.1.6.1 Privilege and Global Interrupt-Enable Stack in mstatus register
        // "When a hart is executing in privilege mode x, interrupts are globally enabled when x
        // IE=1 and globally disabled when x IE=0."
        match self.mode {
            Mode::Machine => {
                // Check if the MIE bit is enabled.
                if (self.csrs[MSTATUS] >> 3) & 1 == 0 {
                    return None;
                }
            }
            Mode::Supervisor => {
                // Check if the SIE bit is enabled.
                if (self.csrs[SSTATUS] >> 1) & 1 == 0 {
                    return None;
                }
            }
            _ => {}
        }

        // Check external interrupt for uart and virtio.
        let irq;
        if self.bus.uart.is_interrupting() {
            irq = UART_IRQ;
        } else {
            irq = 0;
        }

        if irq != 0 {
            self.bus
                .store(PLIC_SCLAIM, 32, irq)
                .expect("failed to write an IRQ to the PLIC_SCLAIM");
            self.csrs[MIP] = self.csrs[MIP] | MIP_SEIP;
        }

        // "An interrupt i will be taken if bit i is set in both mip and mie, and if interrupts are globally enabled.
        // By default, M-mode interrupts are globally enabled if the hart’s current privilege mode is less than
        // M, or if the current privilege mode is M and the MIE bit in the mstatus register is set. If bit i
        // in mideleg is set, however, interrupts are considered to be globally enabled if the hart’s current
        // privilege mode equals the delegated privilege mode (S or U) and that mode’s interrupt enable bit
        // (SIE or UIE in mstatus) is set, or if the current privilege mode is less than the delegated privilege
        // mode."

        let pending = self.csrs[MIE] & self.csrs[MIP];

        if (pending & MIP_MEIP) != 0 {
            self.csrs[MIP] = self.csrs[MIP] & !MIP_MEIP;
            return Some(Interrupt::MachineExternalInterrupt);
        }
        if (pending & MIP_MSIP) != 0 {
            self.csrs[MIP] = self.csrs[MIP] & !MIP_MSIP;
            return Some(Interrupt::MachineSoftwareInterrupt);
        }
        if (pending & MIP_MTIP) != 0 {
            self.csrs[MIP] = self.csrs[MIP] & !MIP_MTIP;
            return Some(Interrupt::MachineTimerInterrupt);
        }
        if (pending & MIP_SEIP) != 0 {
            self.csrs[MIP] = self.csrs[MIP] & !MIP_SEIP;
            return Some(Interrupt::SupervisorExternalInterrupt);
        }
        if (pending & MIP_SSIP) != 0 {
            self.csrs[MIP] = self.csrs[MIP] & !MIP_SSIP;
            return Some(Interrupt::SupervisorSoftwareInterrupt);
        }
        if (pending & MIP_STIP) != 0 {
            self.csrs[MIP] = self.csrs[MIP] & !MIP_STIP;
            return Some(Interrupt::SupervisorTimerInterrupt);
        }
        None
    }

    // Get an instruction from memory
    // | means OR in Rust
    // this is a little-endian system
    pub fn fetch(&mut self) -> Result<u64,Exception>{
        match self.bus.load(self.pc,32){
            Ok(inst) => Ok(inst),
            Err(_e) => Err(Exception::InstructionAccessFault)
        }
    }

    //  Return true if an error happens, otherwise false.
    pub fn execute(&mut self, inst:u64) -> Result<(),Exception>{
        // decode 
        // get opcode,rd,rs1,rs2
        //let inst = inst as u64;
        let opcode = inst & 0x0000007f;
        let funct3 = (inst & 0x00007000) >> 12;
        let funct7 = (inst & 0xfe000000) >> 25;
        let rd = ((inst & 0x00000f80) >> 7 ) as usize;
        let rs1 = ((inst & 0x000f8000) >> 15 ) as usize;
        let rs2 = ((inst & 0x01f00000) >> 20 ) as usize;

        println!("{}",opcode);

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
            0x0f => {
                // A fence instruction does nothing because this emulator executes an
                // instruction sequentially on a single thread.
                match funct3 {
                    0x0 => {} // fence
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
            // RV64A: "A" standard extension for atmic instructions
            // atmic instruction guarantee not interfare with other orders
            0x2f => {
                let funct5 = (funct7 & 0b1111100) >> 2;
                // acquire access
                let _aq = (funct7 & 0b0000010) >> 1;
                // release access
                let _rl = funct7 & 0b0000001;
                match (funct3,funct5){
                    (0x2,0x00) => {
                        // amoadd.w
                        // loads and store 32 bits data
                        let t = self.bus.load(self.regs[rs1],32)?;
                        self.bus.store(self.regs[rs1], 32, t.wrapping_add(self.regs[rs2]))?;
                        self.regs[rd] = t;
                    }
                    (0x3,0x00) => {
                        // amoadd.d
                        // loads and store 64 bits data
                        let t = self.bus.load(self.regs[rs1],64)?;
                        self.bus.store(self.regs[rs1], 64, t.wrapping_add(self.regs[rs2]))?;
                        self.regs[rd] = t;
                    }
                    (0x2,0x01) => {
                        // amoswap.w
                        // swap 32 bits data
                        let t = self.bus.load(self.regs[rs1],32)?;
                        self.bus.store(self.regs[rs1], 32, self.regs[rs2])?;
                        self.regs[rd] = t ;
                    }
                    (0x3, 0x01) => {
                        // amowap.d
                        // swap and store 64 bits data
                        let t = self.bus.load(self.regs[rs1],64)?;
                        self.bus.store(self.regs[rs1], 64,self.regs[rs2])?;
                        self.regs[rd] = t;
                    }
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
            0x73 => {
                let csr_addr = ((inst & 0xfff00000) >> 20) as usize;
                match funct3 {
                    0x0 => {
                        match (rs2, funct7){
                            (0x0, 0x0) => {
                                // ecall 
                                // Makes a request of the execution enxironment 
                                // by raising an environment call exception
                                match self.mode {
                                    Mode::User => {
                                        return Err(Exception::EnvironmentCallFromUMode);
                                    }
                                    Mode::Surpervisor => {
                                        return Err(Exception::EnvironmentCallFromSMode);
                                    }
                                    Mode::Machine => {
                                        return Err(Exception::EnvironmentCallFromMMode)
                                    }
                                } 
                            }
                            (0x1, 0x0) => {
                                // ebreak
                                // Makes a request of the debugger 
                                // by raising a Breakpoint exception
                                return Err(Exception::Breakpoint);
                            }
                            (0x2, 0x8) => {
                                // 🍫🍫 sret
                                // * trap ment both exception and interruption  
                                // surpervisor mode to other mode
                                // The SRET instruction returns from a supervisor-mode exception
                                // handler. It does the following operations:
                                // - Sets the pc to CSRs[sepc].
                                // - Sets the privilege mode to CSRs[sstatus].SPP.
                                // - Sets CSRs[sstatus].SIE to CSRs[sstatus].SPIE.
                                // - Sets CSRs[sstatus].SPIE to 1.
                                // - Sets CSRs[sstatus].SPP to 0.
                                self.pc = self.csrs[SEPC];
                                // When the SRET instruction is executed to return from the trap
                                // handler, the privilege level is set to user mode if the SPP
                                // bit is 0, or supervisor mode if the SPP bit is 1. The SPP bit
                                // is the 8th of the SSTATUS csr.
                                self.mode = match (self.csrs[SSTATUS] >> 8) & 1 {
                                    1 => Mode::Surpervisor,
                                    _ => Mode::User,
                                };
                                // The SPIE bit is the 5th and the SIE bit is the 1st of the
                                // SSTATUS csr.
                                self.csrs[SSTATUS] = if ((self.csrs[SSTATUS] >> 5) & 1) == 1 {
                                    self.csrs[SSTATUS] | (1 << 1)
                                }else{
                                    self.csrs[SSTATUS] & !(1 << 1)
                                };
                                self.csrs[SSTATUS] = self.csrs[SSTATUS] | (1 << 5);
                                self.csrs[SSTATUS] = self.csrs[SSTATUS] & !(1 << 8);
                            }
                            (0x2, 0x18) => {
                                // 🍫 mret
                                // machine mode to other mode
                                // The MRET instruction returns from a machine-mode exception
                                // handler. It does the following operations:
                                // - Sets the pc to CSRs[mepc].
                                // - Sets the privilege mode to CSRs[mstatus].MPP.
                                // - Sets CSRs[mstatus].MIE to CSRs[mstatus].MPIE.
                                // - Sets CSRs[mstatus].MPIE to 1.
                                // - Sets CSRs[mstatus].MPP to 0.
                                self.pc = self.csrs[MEPC];
                                // MPP is two bits wide at [11..12] of the MSTATUS csr.
                                self.mode = match (self.csrs[MSTATUS] >> 11) & 0b11 {
                                    2 => Mode::Machine,
                                    1 => Mode::Surpervisor,
                                    _ => Mode::User,
                                };
                                // The MPIE bit is the 7th and the MIE bit is the 3rd of the
                                // MSTATUS csr. 
                                self.csrs[MSTATUS] = if((self.csrs[MSTATUS] >> 7) & 1) == 1 {
                                    self.csrs[MSTATUS] | (1 << 3)
                                }else{
                                    self.csrs[MSTATUS] & !(1 << 3)
                                };
                                self.csrs[MSTATUS] = self.csrs[MSTATUS] | (1 << 7);
                                self.csrs[MSTATUS] = self.csrs[MSTATUS] & !(0b11 << 11);
                            }
                            (_,0x9) => {
                                // sfence.vma
                                // Do nothing
                            }
                            _ => {}
                        }
                    }
                    0x1 => {
                        // csrrw
                        // atomic read/write CSR.
                        // Atomically swaps values in the CSRs and integer registers.
                        let t = self.csrs[csr_addr];
                        self.csrs[csr_addr] = self.regs[rs1];
                        self.regs[rd] = t;
                    }
                    0x2 => {
                        // csrrs
                        // atomic read and set bits in CSR.
                        let t = self.csrs[csr_addr];
                        self.csrs[csr_addr] = t | self.regs[rs1];
                        self.regs[rd] = t;
                    }
                    0x3 => {
                        // csrrc
                        // atomic read and clear bits in CSR.
                        let t = self.csrs[csr_addr];
                        self.csrs[csr_addr] = t & (!self.regs[rs1]);
                        self.regs[rd] = t;
                    }
                    0x5 => {
                        // csrrwi	
                        // Update the CSR using an XLEN-bit value obtained by zero-extending 
                        // a 5-bit unsigned immediate (uimm[4:0]) field encoded in the rs1 field.
                        let zimm = rs1 as u64;
                        self.regs[rd] = self.csrs[csr_addr];
                        self.csrs[csr_addr] = zimm;
                    }
                    0x06 => {
                        // csrrsi
                        // Set CSR bit using an XLEN-bit value obtained by zero-extending
                        // a 5-bit unsigned immediate (uimm[4:0]) field encoded in the rs1 field.
                        let zimm =rs1 as u64;
                        let t = self.csrs[csr_addr];
                        self.csrs[csr_addr] = t | zimm;
                        self.regs[rd] = t;
                    }
                    0x07 => {
                        // csrrci
                        // Clear CSR bit using an XLEN-bit value obtained by zero-extending
                        // a 5-bit unsigned immediate (uimm[4:0]) field encoded in the rs1 field.
                        let zimm = rs1 as u64;
                        let t = self.csrs[csr_addr];
                        self.csrs[csr_addr] = t & (!zimm);
                        self.regs[rd] = t;
                    }
                    _ => {}
                }
            }
            _ => {
                dbg!(format!("not implemented yet: opcade {:#x}",opcode));
                return Err(Exception::IllegalInstruction);
            }
        }
        return Ok(());
    }
}