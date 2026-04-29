use std::sync::LazyLock;

use crate::emulator::{interrupts::ExceptionType, psr::PsrBitMask, thread::Thread};
use log::{debug, error, info, trace, warn};
pub static MEMORY: LazyLock<Memory> = LazyLock::new(|| Memory::new());
pub struct Memory {
    ptr: *mut i8,
    vec: Vec<i8>,
}
unsafe impl Sync for Memory {}
unsafe impl Send for Memory {}

pub const MEMORY_SIZE: usize = 1 << 32;

impl Memory {
    pub fn new() -> Self {
        let mut vec = vec![0i8; MEMORY_SIZE];
        let ptr = vec.as_mut_ptr();
        // some tests
        unsafe {
            let base = ptr as *mut u8;

            // low address
            base.add(0).write_volatile(0);

            // mid
            base.add(0x10000000).write_volatile(0);

            // high
            base.add(0xF0100000).write_volatile(0);
        }

        Self {
            ptr: ptr as *mut i8,
            vec,
        }
    }
    pub fn clear(&self) {
        todo!()
        // unsafe {
        //     std::ptr::write_bytes(self.ptr as *mut i8, 0, self.data.len());
        // }
    }

    #[inline]
    pub unsafe fn read16(&self, addr: u32) -> i16 {
        let addr = addr as u32 as usize;
        if addr > MEMORY_SIZE {
            panic!(
                "Address was outside of the allocated memory: addr:'{addr:#x}' size:'{MEMORY_SIZE:#x}' "
            )
        }
        unsafe {
            let ptr = self.ptr.clone().add(addr) as *const i16;
            ptr.read_unaligned()
        }
    }
    #[inline]
    pub unsafe fn write16(&self, addr: u32, value: i16) {
        let addr = addr as u32 as usize;
        if addr > MEMORY_SIZE {
            panic!(
                "Address was outside of the allocated memory: addr:'{addr:#x}' size:'{MEMORY_SIZE:#x}' "
            )
        }
        unsafe {
            let ptr = self.ptr.clone().add(addr) as *mut i16;
            ptr.write_unaligned(value);
        }
    }

    #[inline]
    pub unsafe fn read8(&self, addr: u32) -> i8 {
        let addr = addr as u32 as usize;
        if addr > MEMORY_SIZE {
            panic!(
                "Address was outside of the allocated memory: addr:'{addr:#x}' size:'{MEMORY_SIZE:#x}' "
            )
        }
        unsafe {
            let ptr = self.ptr.clone().add(addr) as *const i8;
            ptr.read_unaligned()
        }
    }
    #[inline]
    pub unsafe fn write8(&self, addr: u32, value: i8) {
        let addr = addr as u32 as usize;
        if addr > MEMORY_SIZE {
            panic!(
                "Address was outside of the allocated memory: addr:'{addr:#x}' size:'{MEMORY_SIZE:#x}' "
            )
        }
        unsafe {
            let ptr = self.ptr.clone().add(addr) as *mut i8;
            ptr.write_unaligned(value);
        }
    }

    #[inline]
    pub unsafe fn read(&self, addr: u32) -> i32 {
        let addr = addr as u32 as usize;
        if addr > MEMORY_SIZE {
            panic!(
                "Address was outside of the allocated memory: addr:'{addr:#x}' size:'{MEMORY_SIZE:#x}' "
            )
        }
        trace!("read: {addr}");
        unsafe {
            let ptr = self.ptr.clone().add(addr) as *const i32;
            trace!("ptr: {ptr:?}");
            ptr.read_unaligned()
        }
    }
    #[inline]
    pub unsafe fn write(&self, addr: u32, value: i32) {
        let addr = addr as u32 as usize;
        if addr > MEMORY_SIZE {
            panic!(
                "Address was outside of the allocated memory: addr:'{addr:#x}' size:'{MEMORY_SIZE:#x}' "
            )
        }
        unsafe {
            let ptr = self.ptr.clone().add(addr) as *mut i32;
            ptr.write_unaligned(value);
        }
    }
}

impl Thread {
    pub fn handle_memory_load(&mut self, addr: i32) -> Option<i32> {
        let addr = addr as u32;
        if addr < 0xD0000000u32 {
            unsafe { Some(MEMORY.read(addr)) }
        } else {
            if addr < 0xE0000000u32 {
                //Framebuffer
                error!("Framebuffer memory loading is not yet implemented");
                None
            } else if addr < 0xF0000000u32 {
                error!("I/O memory loading is not yet implemented");
                None
            } else {
                // Kernel
                if self.read_psr_bit(PsrBitMask::KernelPrivelage) {
                    unsafe { Some(MEMORY.read(addr)) }
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
        let addr = addr as u32;
        if addr < 0xD0000000u32 {
            debug!("1.handle_memory_store");
            store_closure();
        } else {
            debug!("2.handle_memory_store");
            if addr < 0xE0000000u32 {
                //Framebuffer
                if let Some(handle) = &mut self.frame_buffer_handle {
                    if let Err(err) = handle.write(addr - 0xD0000000, value as u32) {
                        error!("Writing to frame buffer did not succeed: {err}");
                    }
                } else {
                    error!("Frame buffer was not yet initialized(at least for this thread)!");
                }
            } else if addr < 0xF0000000u32 {
                if let Err(err) = self.handle_io_write(addr - 0xD0000000, value as u32) {
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
            MEMORY.write(addr as u32, value);
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
            MEMORY.write8(addr as u32, value);
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
            MEMORY.write16(addr as u32, value);
        });
    }

    pub fn loadpc(&mut self, rd: u8, imm: i16) {
        let addr = self.pc.wrapping_add(imm as i32);
        let value = self.handle_memory_load(addr).unwrap_or_else(|| 0);
        self.gpr[rd as usize] = value;
    }
}
