use crate::bus::*;
use crate::trap::*;

/// Default memory size (128MB).
pub const MEMORY_SIZE: u64 = 1024 * 1024 * 128;

pub struct Memory {
    pub memory : Vec<u8>,
}

impl Device for Memory {
    fn load(&self, addr:u64, size: u64) -> Result<u64, Exception>{
        match size {
            8 => Ok(self.load8(addr)),
            16 => Ok(self.load16(addr)),
            32 => Ok(self.load32(addr)),
            64 => Ok(self.load64(addr)),
            _ => Err(Exception::LoadAccessFault),
        }
    }

    fn store(&mut self,addr: u64, size:u64, value: u64) -> Result<(),Exception> {
        match size {
            8 => Ok(self.store8(addr,value)),
            16 => Ok(self.store16(addr,value)),
            32 => Ok(self.store32(addr,value)),
            64 => Ok(self.store64(addr,value)),
            _ => Err(Exception::StoreAMOAccessFault),
        }
    }
}

impl Memory {
    pub fn new(binary: Vec<u8>) -> Memory {
        let mut memory = vec![0; MEMORY_SIZE as usize];
        memory.splice(..binary.len(), binary.iter().cloned());

        Self { memory }
    }

    // load 1 bite from the littele-endian memory
    pub fn load8(&self, addr:u64) -> u64 {
        let index = (addr - MEMORY_BASE) as usize;
        self.memory[index] as u64
    }

    // load 2 bites from the littele-endian memory
    pub fn load16(&self, addr:u64) -> u64 {
        let index = (addr - MEMORY_BASE) as usize;
        return (self.memory[index] as u64) 
            | ((self.memory[index + 1] as u64) << 8);
    }

    //load 4 bites from the littele-endian memory
    pub fn load32(&self, addr:u64) -> u64 {
        let index = (addr - MEMORY_BASE) as usize;
        return (self.memory[index] as u64)
            | ((self.memory[index + 1] as u64) << 8)
            | ((self.memory[index + 2] as u64) << 16)
            | ((self.memory[index + 3] as u64) << 24);
    }

    // load 8 bites from the little-endian memory
    pub fn load64(&self, addr: u64) -> u64 {
        let index = (addr - MEMORY_BASE) as usize;
        return (self.memory[index] as u64) 
            | ((self.memory[index + 1] as u64) << 8)
            | ((self.memory[index + 2] as u64) << 16)
            | ((self.memory[index + 3] as u64) << 24)
            | ((self.memory[index + 4] as u64) << 32)
            | ((self.memory[index + 5] as u64) << 40)
            | ((self.memory[index + 6] as u64) << 48)
            | ((self.memory[index + 7] as u64) << 56);
    }

    // 🍫 val & 0xff ?
    // store 1 bite to the little-endian memory
    pub fn store8(&mut self, addr:u64, val:u64){
        let index = (addr - MEMORY_BASE) as usize;
        self.memory[index] = val as u8;
    }

    // store 2 bites to the little-endian memory
    pub fn store16(&mut self, addr:u64, val:u64){
        let index = (addr - MEMORY_BASE) as usize;
        self.memory[index] = (val & 0xff) as u8;
        self.memory[index + 1] = ((val >> 8) & 0xff) as u8;
    }

    // store 4 bites to the little-endian memory
    pub fn store32(&mut self, addr:u64, val:u64){
        let index = (addr - MEMORY_BASE) as usize;
        self.memory[index] = (val & 0xff) as u8;
        self.memory[index + 1] = ((val >> 8) & 0xff) as u8;
        self.memory[index + 2] = ((val >> 16) & 0xff) as u8;
        self.memory[index + 3] = ((val >> 24) & 0xff) as u8;
    }

    // store 8 bites to the little-endian memory
    pub fn store64(&mut self,addr:u64, val:u64){
        let index = (addr - MEMORY_BASE) as usize;
        self.memory[index] = (val & 0xff) as u8;
        self.memory[index + 1] = ((val >> 8) & 0xff) as u8;
        self.memory[index + 2] = ((val >> 16) & 0xff) as u8;
        self.memory[index + 3] = ((val >> 24) & 0xff) as u8;
        self.memory[index + 4] = ((val >> 32) & 0xff) as u8;
        self.memory[index + 5] = ((val >> 40) & 0xff) as u8;
        self.memory[index + 6] = ((val >> 48) & 0xff) as u8;
        self.memory[index + 7] = ((val >> 56) & 0xff) as u8;
    }
}