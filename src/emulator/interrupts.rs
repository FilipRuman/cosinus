use crate::emulator::{memory::MEMORY, psr::PsrBitMask, thread::Thread};
#[repr(i32)]
pub enum InterruptType {
    Exception,
    Syscall,
    Keyboard,
    Timer,
    Drive,
}

#[repr(i32)]
pub enum ExceptionType {
    InsufficientPrivelages,
    InvalidSystemRegister,
    UnknownInstructionOptcode,
    InterruptLogicError,
    InvalidInterruptFucntionAdress,
}

impl Thread {
    fn set_pending_interrupt(&mut self, interrupt_bit: i32, val: bool) {
        let mask: i32 = 1i32 << interrupt_bit as i32;

        if val {
            self.ipr |= mask;
        } else {
            self.ipr &= !mask;
        }
    }
    pub fn trigger_interrupt(&mut self, interrupt_type: InterruptType) {
        self.set_pending_interrupt(interrupt_type as i32, true);
    }

    pub fn trigger_exception(&mut self, exception_type: ExceptionType) {
        self.trigger_interrupt(InterruptType::Exception);
        self.etr = exception_type as i32;
    }
    pub fn should_trigger_an_interrupt(&self) -> bool {
        let masked_ipr = self.ipr & !self.imr;
        masked_ipr != 0 && self.read_psr_bit(PsrBitMask::EnableInterrupts)
    }
    pub fn handle_interrupt(&mut self) {
        let masked_ipr = self.ipr & !self.imr;
        let interrupt_index = masked_ipr.leading_zeros() as i32;
        self.set_pending_interrupt(interrupt_index, false);
        let adress = unsafe { MEMORY.read((self.ivt + interrupt_index * 4) as usize) };

        if adress < 0xF0000000u32 as i32 {
            self.trigger_exception(ExceptionType::InvalidInterruptFucntionAdress);
            return;
        }
        self.epc = self.pc;
        self.pc = adress;
        self.write_psr_bit(PsrBitMask::KernelPrivelage, true);
    }
}
