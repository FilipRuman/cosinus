use log::info;
use thread::Thread;

use crate::emulator::memory::MEMORY;

pub mod arithmetics;
pub mod atomic;
pub mod branching;
pub mod compare;
pub mod fb;
pub mod flow_controll;
pub mod instruction_parsing;
pub mod interrupts;
pub mod io;
pub(crate) mod memory;
pub mod psr;
pub mod system_level;
pub mod test;
pub mod thread;

pub async fn run() {
    info!("Hello from emulator!");
    let thread_0 = Thread::new(0);
    tokio::spawn(thread_0.run_loop());
}
pub unsafe fn write_instructions_to_memory(base_addr: u32, data: Vec<i32>) {
    unsafe {
        for (i, value) in data.iter().enumerate() {
            MEMORY.write(base_addr as usize + i * 4, *value);
        }
    }
}
/// Quits on HALT
pub fn run_test(data: Vec<i32>) -> Thread {
    info!("RUN TEST!");
    unsafe {
        write_instructions_to_memory(0, data);
    }
    let mut thread_0 = Thread::new(0);
    thread_0.run_test_loop();
    thread_0
}
