use std::sync::LazyLock;

use crate::emulator::{core::Core, interrupts::ExceptionType, psr::PsrBitMask};
use log::{debug, error, trace, warn};
pub static MEMORY: LazyLock<Memory> = LazyLock::new(|| Memory::new());

// This implementation uses unsafe operations because I don't want any atomic operations hide logic
// bugs in code running on the emulator.
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

    #[inline]
    /// WARN: CAN'T BE USED WITH VEC-s: will write value of the vec datastructure not it's contents.
    /// For use vec versions of read && write functions.
    pub unsafe fn write_vec<T>(&self, addr: u32, value: Vec<T>) {
        let addr = addr as u32 as usize;
        if addr > MEMORY_SIZE {
            panic!(
                "Address was outside of the allocated memory: addr:'{addr:#x}' size:'{MEMORY_SIZE:#x}' "
            )
        }
        unsafe {
            let ptr = self.ptr.clone().add(addr) as *mut T;
            (value.as_ptr()).copy_to_nonoverlapping(ptr, value.len());
        }
    }

    #[inline]
    pub unsafe fn read_vec<T>(&self, addr: u32, amount: usize) -> Vec<T> {
        let addr = addr as u32 as usize;
        if addr > MEMORY_SIZE {
            panic!(
                "Address was outside of the allocated memory: addr:'{addr:#x}' size:'{MEMORY_SIZE:#x}' "
            )
        }

        unsafe {
            let mut vec = Vec::<T>::with_capacity(amount);

            std::ptr::copy_nonoverlapping(
                self.ptr.clone().add(addr) as *mut u8,
                vec.as_mut_ptr() as *mut u8,
                amount * std::mem::size_of::<T>(),
            );

            vec.set_len(amount);

            vec
        }
    }

    #[inline]
    pub unsafe fn write<T>(&self, addr: u32, value: T) {
        let addr = addr as u32 as usize;
        if addr > MEMORY_SIZE {
            panic!(
                "Address was outside of the allocated memory: addr:'{addr:#x}' size:'{MEMORY_SIZE:#x}' "
            )
        }
        unsafe {
            let ptr = self.ptr.clone().add(addr) as *mut T;
            ptr.write_unaligned(value);
        }
    }

    #[inline]
    pub unsafe fn read<T>(&self, addr: u32) -> T {
        let addr = addr as u32 as usize;
        if addr > MEMORY_SIZE {
            panic!(
                "Address was outside of the allocated memory: addr:'{addr:#x}' size:'{MEMORY_SIZE:#x}' "
            )
        }
        trace!("read: {addr}");
        unsafe {
            let ptr = self.ptr.clone().add(addr) as *const T;
            trace!("ptr: {ptr:?}");
            ptr.read_unaligned()
        }
    }
}

impl Core {
    pub fn handle_memory_load(&mut self, addr: i32) -> Option<i32> {
        let addr = addr as u32;
        let value = {
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
        };

        debug!("handle_memory_load:{addr:#x} value:{value:?}");
        value
    }
    pub fn handle_memory_store<F>(&mut self, addr: i32, value: i32, store_closure: F)
    where
        F: Fn(),
    {
        debug!("handle_memory_store:{addr:#x} value:{value:#x}");
        let addr = addr as u32;
        if addr < 0xD0000000u32 {
            store_closure();
        } else {
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
            MEMORY.write(addr as u32, value);
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
            MEMORY.write(addr as u32, value);
        });
    }

    pub fn loadpc(&mut self, rd: u8, imm: i16) {
        let addr = self.pc.wrapping_add(imm as i32);
        let value = self.handle_memory_load(addr).unwrap_or_else(|| 0);
        self.gpr[rd as usize] = value;
    }
}
