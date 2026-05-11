use anyhow::Result;
use log::info;
use thread::Core;

use crate::emulator::memory::MEMORY;

pub mod arithmetics;
pub mod atomic;
pub mod branching;
pub mod compare;
pub mod core;
pub mod disk;
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

pub async fn run() -> Result<()> {
    info!("Hello from emulator!");
    let (frame_buffer_handle, frame_buffer_rx) = fb::init()?;
    let thread_0 = thread::THREADS.get(0);
    thread_0.frame_buffer_handle = Some(frame_buffer_handle);
    thread_0.id = 0;
    tokio::spawn(thread_0.run_loop());
    // fb::run_framebuffer_loop(frame_buffer_rx).await?;
    Ok(())
}
pub unsafe fn write_instructions_to_memory(base_addr: u32, data: Vec<i32>) {
    unsafe {
        for (i, value) in data.iter().enumerate() {
            MEMORY.write_bytes(base_addr + i as u32 * 4, *value);
        }
    }
}
/// Quits on HALT
pub fn run_test(data: Vec<i32>) -> Core {
    unsafe {
        write_instructions_to_memory(0, data);
    }
    let mut thread_0 = Core::new(0, None);
    thread_0.run_test_loop();
    thread_0
}
