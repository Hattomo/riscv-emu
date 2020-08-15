use crate::memory::*;

// The address which memory status.
pub const MEMORY_BASE: u64 = 0x8000_0000;

// trait is like imterface
pub trait Device {
    fn load(&self, addr:u64,size: u64) -> Result<u64, ()>;
    fn store(&mut self,addr: u64,size:u64,value:u64) -> Result<(),()>;
}

// The system bus
pub struct Bus {
    pub memory: Memory,
}

impl Bus {
    pub fn new(binary:Vec<u8>) -> Bus {
        Self {
            memory: Memory::new(binary),
        }
    }

    pub fn load(&self, addr:u64,size:u64) -> Result<u64,()> {
        if MEMORY_BASE <= addr {
            return self.memory.load(addr,size);
        }
        Err(())
    }

    pub fn store(&mut self,addr:u64,size:u64,value:u64) -> Result<(),()> {
        if MEMORY_BASE <= addr {
            return self.memory.store(addr,size,value);
        }
        Err(())
    }
}