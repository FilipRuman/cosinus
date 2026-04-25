use std::sync::LazyLock;

use crate::emulator::{interrupts::ExceptionType, psr::PsrBitMask, thread::Thread};
pub static MEMORY: LazyLock<Memory> = LazyLock::new(|| Memory::new());
pub const MEMORY_SIZE: usize = 1 << 32;
pub struct Memory {
    data: Vec<i8>,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            data: vec![0i8; MEMORY_SIZE],
        }
    }
    pub fn clear(&self) {
        unsafe {
            std::ptr::write_bytes(self.data.as_ptr() as *mut i8, 0, self.data.len());
        }
    }

    #[inline]
    pub unsafe fn read16(&self, addr: usize) -> i16 {
        unsafe {
            let ptr = self.data.as_ptr().add(addr) as *const i16;
            ptr.read_unaligned()
        }
    }
    #[inline]
    pub unsafe fn write16(&self, addr: usize, value: i16) {
        unsafe {
            let ptr = self.data.as_ptr().add(addr) as *mut i16;
            ptr.write_unaligned(value);
        }
    }

    #[inline]
    pub unsafe fn read8(&self, addr: usize) -> i8 {
        unsafe {
            let ptr = self.data.as_ptr().add(addr) as *const i8;
            ptr.read_unaligned()
        }
    }
    #[inline]
    pub unsafe fn write8(&self, addr: usize, value: i8) {
        unsafe {
            let ptr = self.data.as_ptr().add(addr) as *mut i8;
            ptr.write_unaligned(value);
        }
    }

    #[inline]
    pub unsafe fn read(&self, addr: usize) -> i32 {
        unsafe {
            let ptr = self.data.as_ptr().add(addr) as *const i32;
            ptr.read_unaligned()
        }
    }
    #[inline]
    pub unsafe fn write(&self, addr: usize, value: i32) {
        unsafe {
            let ptr = self.data.as_ptr().add(addr) as *mut i32;
            ptr.write_unaligned(value);
        }
    }
}

impl Thread {
    pub fn load(&mut self, rd: u8, rs1: u8, imm: i16) {
        let addr = self.gpr[rs1 as usize].wrapping_add(imm as i32);
        if addr < 0xD0000000u32 as i32 {
            unsafe {
                self.gpr[rd as usize] = MEMORY.read(addr as usize);
            }
        } else {
            if addr < 0xE0000000u32 as i32 {
                //Framebuffer
                todo!("Framebuffer support is not yet implemented");
            } else if addr < 0xF0000000u32 as i32 {
                todo!("I/O support is not yet implemented");
            } else {
                // Kernel
                if self.read_psr_bit(PsrBitMask::KernelPrivelage) {
                    unsafe {
                        self.gpr[rd as usize] = MEMORY.read(addr as usize);
                    };
                } else {
                    self.trigger_exception(ExceptionType::InsufficientPrivelages);
                }
            }
        }
    }

    pub fn store(&self, rs1: u8, rs2: u8, imm: i16) {
        let addr = self.gpr[rs1 as usize].wrapping_add(imm as i32);
        let value = self.gpr[rs2 as usize];
        unsafe {
            MEMORY.write(addr as usize, value);
        }
    }
    pub fn loadb(&mut self, rd: u8, rs1: u8, imm: i16) {
        let addr = self.gpr[rs1 as usize].wrapping_add(imm as i32);
        let val = unsafe { MEMORY.read8(addr as usize) };
        // sign-extend
        self.gpr[rd as usize] = (val as i8) as i32;
    }

    pub fn storeb(&self, rs1: u8, rs2: u8, imm: i16) {
        let addr = self.gpr[rs1 as usize].wrapping_add(imm as i32);
        let val = self.gpr[rs2 as usize] as i8;
        unsafe {
            MEMORY.write8(addr as usize, val);
        }
    }

    pub fn loadh(&mut self, rd: u8, rs1: u8, imm: i16) {
        let addr = self.gpr[rs1 as usize].wrapping_add(imm as i32);
        let val = unsafe { MEMORY.read16(addr as usize) };
        // sign-extend
        self.gpr[rd as usize] = (val as i16) as i32;
    }

    pub fn storeh(&self, rs1: u8, rs2: u8, imm: i16) {
        let addr = self.gpr[rs1 as usize].wrapping_add(imm as i32);
        let val = self.gpr[rs2 as usize] as i16;
        unsafe {
            MEMORY.write16(addr as usize, val);
        }
    }

    pub fn loadpc(&mut self, rd: u8, imm: i16) {
        let addr = self.pc.wrapping_add(imm as i32);
        let val = unsafe { MEMORY.read(addr as usize) };
        self.gpr[rd as usize] = val;
    }
}
