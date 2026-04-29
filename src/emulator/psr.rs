use crate::emulator::thread::Thread;

#[repr(i32)]
pub enum PsrBitMask {
    KernelPrivelage = 1,
    EnableInterrupts = 1 << 1,
    HALT = 1 << 2,
}
impl Thread {
    pub fn write_psr_bit(&mut self, bit: PsrBitMask, val: bool) {
        if val {
            self.psr |= bit as i32;
        } else {
            self.psr &= !(bit as i32);
        }
    }
    pub fn read_psr_bit(&self, bit: PsrBitMask) -> bool {
        (self.psr & bit as i32) != 0
    }
    pub fn halt(&mut self) {
        self.write_psr_bit(PsrBitMask::HALT, true);
    }
}
