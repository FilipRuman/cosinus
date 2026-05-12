use log::{info, trace};

use crate::emulator::{core::Core, memory::MEMORY, psr::PsrBitMask};
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
}

impl Core {
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

        trace!(
            "should_trigger_an_interrupt- masked_ipr:{:#b} EnableInterrupts:{} psr:{:#b}",
            masked_ipr,
            self.read_psr_bit(PsrBitMask::EnableInterrupts),
            self.psr
        );
        masked_ipr != 0 && self.read_psr_bit(PsrBitMask::EnableInterrupts)
    }
    pub fn handle_interrupt(&mut self) {
        let masked_ipr = self.ipr & !self.imr;
        let interrupt_index = masked_ipr.trailing_zeros() as i32;
        let ivt_addr = self.ivt as u32 + interrupt_index as u32 * 4;
        self.set_pending_interrupt(interrupt_index, false);
        let adress = unsafe { MEMORY.read::<u32>(ivt_addr) } + 4;

        info!(
            "handle_interrupt! idx:'{interrupt_index}' ivt:{:#x} ivt_addr:{:#x}, new_pc:{adress}, pc:{}",
            self.ivt, ivt_addr, self.pc
        );

        self.epc = self.pc;
        self.pc = adress as i32;
        self.write_psr_bit(PsrBitMask::KernelPrivelage, true);
    }
}
