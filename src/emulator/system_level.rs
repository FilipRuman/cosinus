use crate::emulator::{
    interrupts::{ExceptionType, InterruptType},
    psr::PsrBitMask,
    thread::Thread,
};

impl Thread {
    pub fn sysw(&mut self, rd: u8, imm: i16) {
        if !self.read_psr_bit(PsrBitMask::KernelPrivelage) {
            self.trigger_exception(ExceptionType::InsufficientPrivelages);
            return;
        }
        match imm {
            0 => self.psr = self.reg(rd),

            1 => self.ivt = self.reg(rd),

            2 => self.imr = self.reg(rd),

            3 => self.epc = self.reg(rd),

            4 => self.tid = self.reg(rd),

            _ => {
                self.trigger_exception(ExceptionType::InvalidSystemRegister);
            }
        }
    }
    pub fn scall(&mut self) {
        self.trigger_interrupt(InterruptType::Syscall);
    }
    pub fn sret(&mut self) {
        self.write_psr_bit(PsrBitMask::KernelPrivelage, false);
        self.pc = self.epc;
    }

    pub fn sysr(&mut self, rd: u8, imm: i16) {
        match imm {
            0 => self.gpr[rd as usize] = self.psr,

            1 => self.gpr[rd as usize] = self.ivt,

            2 => self.gpr[rd as usize] = self.imr,

            3 => self.gpr[rd as usize] = self.epc,

            4 => self.gpr[rd as usize] = self.tid,

            5 => self.gpr[rd as usize] = self.etr,
            _ => {
                self.trigger_exception(ExceptionType::InvalidSystemRegister);
            }
        }
    }
}
