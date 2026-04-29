use std::sync::LazyLock;

use crate::emulator::{interrupts::ExceptionType, psr::PsrBitMask, thread::Thread};
use libc::{MAP_ANONYMOUS, MAP_PRIVATE, PROT_READ, PROT_WRITE, mmap};
use log::{debug, error, warn};
pub static MEMORY: LazyLock<Memory> = LazyLock::new(|| Memory::new());
pub const MEMORY_SIZE: usize = u32::MAX as usize;
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
    pub fn handle_memory_load(&mut self, addr: i32) -> Option<i32> {
        if (addr as u32) < 0xD0000000u32 {
            unsafe { Some(MEMORY.read(addr as usize)) }
        } else {
            if (addr as u32) < 0xE0000000u32 {
                //Framebuffer
                error!("Framebuffer memory loading is not yet implemented");
                None
            } else if (addr as u32) < 0xF0000000u32 {
                error!("I/O memory loading is not yet implemented");
                None
            } else {
                // Kernel
                if self.read_psr_bit(PsrBitMask::KernelPrivelage) {
                    unsafe { Some(MEMORY.read(addr as usize)) }
                } else {
                    self.trigger_exception(ExceptionType::InsufficientPrivelages);
                    None
                }
            }
        }
    }
    pub fn handle_memory_store<F>(&mut self, addr: i32, value: i32, store_closure: F)
    where
        F: Fn(),
    {
        if (addr as u32) < 0xD0000000u32 {
            debug!("1.handle_memory_store");
            store_closure();
        } else {
            debug!("2.handle_memory_store");
            if (addr as u32) < 0xE0000000u32 {
                //Framebuffer
                if let Some(handle) = &mut self.frame_buffer_handle {
                    if let Err(err) = handle.write((addr as u32) - 0xD0000000, value as u32) {
                        error!("Writing to frame buffer did not succeed: {err}");
                    }
                } else {
                    error!("Frame buffer was not yet initialized(at least for this thread)!");
                }
            } else if (addr as u32) < 0xF0000000u32 {
                if let Err(err) = self.handle_io_write((addr as u32) - 0xD0000000, value as u32) {
                    error!("Writing to io did not succeed: {err}")
                }
            } else {
                // Kernel
                if self.read_psr_bit(PsrBitMask::KernelPrivelage) {
                    store_closure();
                } else {
                    self.trigger_exception(ExceptionType::InsufficientPrivelages);
                }
            }
        }
    }

    pub fn load(&mut self, rd: u8, rs1: u8, imm: i16) {
        let addr = self.gpr[rs1 as usize].wrapping_add(imm as i32);
        let value = self.handle_memory_load(addr).unwrap_or_else(|| 0);
        self.gpr[rd as usize] = value;
    }

    pub fn store(&mut self, rs1: u8, rs2: u8, imm: i16) {
        let addr = self.gpr[rs1 as usize].wrapping_add(imm as i32);
        let value = self.gpr[rs2 as usize];
        warn!("store! a:{addr} v:{value} v_u:{:#x}", value as u32);
        self.handle_memory_store(addr, value, || unsafe {
            warn!("Closure hit!");
            MEMORY.write(addr as usize, value);
        });
    }
    pub fn loadb(&mut self, rd: u8, rs1: u8, imm: i16) {
        let addr = self.gpr[rs1 as usize].wrapping_add(imm as i32);
        let value = self.handle_memory_load(addr).unwrap_or_else(|| 0);

        self.gpr[rd as usize] = (value as i8) as i32;
    }

    pub fn storeb(&mut self, rs1: u8, rs2: u8, imm: i16) {
        let addr = self.gpr[rs1 as usize].wrapping_add(imm as i32);
        let value = self.gpr[rs2 as usize] as i8;

        self.handle_memory_store(addr, value as i32, || unsafe {
            MEMORY.write8(addr as usize, value);
        });
    }

    pub fn loadh(&mut self, rd: u8, rs1: u8, imm: i16) {
        let addr = self.gpr[rs1 as usize].wrapping_add(imm as i32);
        let value = self.handle_memory_load(addr).unwrap_or_else(|| 0);

        self.gpr[rd as usize] = (value as i16) as i32;
    }

    pub fn storeh(&mut self, rs1: u8, rs2: u8, imm: i16) {
        let addr = self.gpr[rs1 as usize].wrapping_add(imm as i32);
        let value = self.gpr[rs2 as usize] as i16;

        self.handle_memory_store(addr, value as i32, || unsafe {
            MEMORY.write16(addr as usize, value);
        });
    }

    pub fn loadpc(&mut self, rd: u8, imm: i16) {
        let addr = self.pc.wrapping_add(imm as i32);
        let value = self.handle_memory_load(addr).unwrap_or_else(|| 0);
        self.gpr[rd as usize] = value;
    }
}
