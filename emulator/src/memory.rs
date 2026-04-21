pub const MEMORY_SIZE: usize = 1 << 32;
pub struct Memory {
    data: Vec<u8>,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            data: vec![0u8; MEMORY_SIZE],
        }
    }

    #[inline]
    pub unsafe fn read(&self, addr: usize) -> u32 {
        unsafe {
            let ptr = self.data.as_ptr().add(addr) as *const u32;
            ptr.read_unaligned()
        }
    }
    #[inline]
    pub unsafe fn write(&self, addr: usize, value: u32) {
        unsafe {
            let ptr = self.data.as_ptr().add(addr) as *mut u32;
            ptr.write_unaligned(value);
        }
    }
}
