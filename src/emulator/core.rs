use log::info;
use std::{cell::UnsafeCell, sync::LazyLock, time::Duration};
use tokio::time::sleep;

use crate::emulator::{fb::FramebufferHandle, memory::MEMORY, psr::PsrBitMask};

pub const CORE_COUNT: usize = 1;
pub static CORES: LazyLock<Cores> = LazyLock::new(|| todo!());
unsafe impl Sync for Cores {}
struct Cores {
    threads: UnsafeCell<[Core; CORE_COUNT]>,
}
impl Cores {
    pub fn new() -> Self {
        let threads = UnsafeCell::new([Core::new(0, None); CORE_COUNT]);
        Self { threads }
    }
    pub fn get(&self, thread_id: u8) -> &'static mut Core {
        unsafe { &mut (*self.threads.get())[thread_id as usize] }
    }
}
pub struct Core {
    pub id: u8,
    /// general purpose registers
    pub gpr: Vec<i32>,
    /// programable counter
    pub pc: i32,
    /// processor status register
    pub psr: i32,
    /// interrupt table base address
    pub ivt: i32,
    /// interrupt mask register
    pub imr: i32,
    /// interrupt pending register
    pub ipr: i32,
    /// stores PC when interrupt occurs
    pub epc: i32,
    /// thread id register
    pub tid: i32,
    /// exception type register
    pub etr: i32,
    pub frame_buffer_handle: Option<FramebufferHandle>,
    pub serial_buffer: String,
}

const GPR_COUNT: usize = 32;
impl Core {
    pub fn new(id: u8, frame_buffer_handle: Option<FramebufferHandle>) -> Self {
        Self {
            id,
            gpr: vec![0i32; GPR_COUNT],
            pc: 0,
            psr: 0,
            ivt: 0,
            imr: 0,
            ipr: 0,
            epc: 0,
            tid: id as i32,
            etr: 0,
            frame_buffer_handle: frame_buffer_handle,
            serial_buffer: String::new(),
        }
    }

    /// Stack Pointer Register
    pub const SP: usize = 1;
    /// Return Address Register
    pub const RA: usize = 2;
    /// Frame Pointer Register
    pub const FP: usize = 3;

    #[inline(always)]
    pub fn reg(&self, r: u8) -> i32 {
        self.gpr[r as usize]
    }

    pub async fn run_loop(&'static mut self) {
        println!("RUN");
        self.gpr[0] = 0;
        loop {
            self.run_current_instruction();
            // If there are multiple interrupts stacked at the same time, they should be executed
            // one after another without running the 'normal:non-interrupt' code in the middle. This
            // is because the last instruction of an interrupt function(sret) will enable interrupts
            // and another interrupt may be instantly executed.
            if self.should_trigger_an_interrupt() {
                self.write_psr_bit(PsrBitMask::EnableInterrupts, false);
                self.write_psr_bit(PsrBitMask::HALT, false);
                // WARN: This will not execute all instructions of an interrupt function.
                // It will only jump to the right address and set all CPU registers in the right way.
                // Interrupt code will run this loop in the normal way.
                self.handle_interrupt();
            }
            while self.read_psr_bit(PsrBitMask::HALT) && !self.should_trigger_an_interrupt() {
                sleep(Duration::from_micros(20)).await;
            }

            if unsafe { MEMORY.read::<u32>(self.pc as u32) } == 0 {
                // TEMP:
                panic!("Hit an zero instruction in a test code!");
            }
        }
    }
    /// Quits on HALT
    pub fn run_test_loop(&mut self) {
        self.gpr[0] = 0;
        info!("RUN TEST LOOP!{} {}", self.psr, self.ivt);
        loop {
            let addr = self.pc as u32;
            let instruction: u32 = unsafe { MEMORY.read(addr) };
            if self.should_trigger_an_interrupt() {
                self.handle_interrupt();
            } else if self.read_psr_bit(PsrBitMask::HALT) {
                break;
            } else if instruction == 0 {
                panic!("Hit an zero instruction in a test code!");
            } else {
                self.run_current_instruction();
            }
        }
    }
}
