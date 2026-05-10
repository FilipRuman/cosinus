use std::{cell::UnsafeCell, sync::LazyLock};

// This implementation uses unsafe operations because I don't want any atomic operations hide logic
// bugs in code running on the emulator.
pub static DISK_IO_DEVICE: LazyLock<DiskIODevice> = LazyLock::new(|| DiskIODevice {
    reg: UnsafeCell::new(0),
    base_block_index: UnsafeCell::new(0),
    buffer_address: UnsafeCell::new(0),
    block_count: UnsafeCell::new(0),
    core_id_to_interrupt: UnsafeCell::new(0),
    status: UnsafeCell::new(0),
});
unsafe impl Sync for DiskIODevice {}
pub struct DiskIODevice {
    pub reg: UnsafeCell<u32>,
    pub base_block_index: UnsafeCell<u32>,
    pub buffer_address: UnsafeCell<u32>,
    pub block_count: UnsafeCell<u32>,
    pub core_id_to_interrupt: UnsafeCell<u32>,
    pub status: UnsafeCell<u32>,
    tx: mpsc::Sender<PixelOp>,
}
pub fn handle_write() {
    unsafe {
        *DISK_IO_DEVICE.reg.get() = 25;
    }
}
