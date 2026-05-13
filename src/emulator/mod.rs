use anyhow::Result;
use log::info;

use crate::emulator::{
    core::{CORES, Core},
    memory::MEMORY,
};

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
pub mod serial;
pub mod system_level;
pub mod test;

pub async fn run() -> Result<()> {
    info!("Hello from emulator!");
    let (frame_buffer_handle, frame_buffer_rx) = fb::init()?;
    let (disk_buffer_handle, disk_buffer_rx) = disk::io_device::init()?;
    tokio::spawn(disk::io_device::run_disk_loop(disk_buffer_rx));

    let thread_0 = unsafe { &mut CORES[0] };

    thread_0.frame_buffer_handle = Some(frame_buffer_handle);
    thread_0.disk_handle = Some(disk_buffer_handle);
    thread_0.id = 0;
    thread_0.write_psr_bit(psr::PsrBitMask::KernelPrivelage, true);
    tokio::spawn(thread_0.run_loop());
    // fb::run_framebuffer_loop(frame_buffer_rx).await?;
    Ok(())
}
pub unsafe fn write_instructions_to_memory(base_addr: u32, data: Vec<i32>) {
    unsafe {
        for (i, value) in data.iter().enumerate() {
            MEMORY.write(base_addr + i as u32 * 4, *value);
        }
    }
}
/// Quits on HALT
pub fn run_test(data: Vec<i32>) -> Core {
    unsafe {
        write_instructions_to_memory(0, data);
    }
    let mut thread_0 = Core::new(0, None, None);
    thread_0.run_test_loop();
    thread_0
}
