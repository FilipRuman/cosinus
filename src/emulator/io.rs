use crate::emulator::{core::Core, disk::io_device, serial::handle_serial_write};
use anyhow::{Result, bail};
use log::{debug, info};
const DEVICE_ID_MASK: u32 = 0xFF00000;
impl Core {
    pub fn handle_io_write(&mut self, relative_addr: u32, value: u32) -> Result<()> {
        debug!("handle_io_write relative_addr{relative_addr:#x}");
        let id = (relative_addr & DEVICE_ID_MASK) >> 20;
        let relative_addr = relative_addr & 0xFFFFF;
        match id {
            0 => self.handle_disk_write(relative_addr, value),
            1 => self.handle_timer_write(relative_addr, value),
            2 => self.handle_audio_write(relative_addr, value),
            3 => handle_serial_write(relative_addr, value),
            _ => bail!("device with id:{id} is not yet implemented"),
        }
    }
    fn handle_timer_write(&self, relative_addr: u32, value: u32) -> Result<()> {
        todo!("timer is not yet implemented");
    }
    fn handle_audio_write(&self, relative_addr: u32, value: u32) -> Result<()> {
        todo!("audio is not yet implemented");
    }
    fn handle_disk_write(&self, relative_addr: u32, value: u32) -> Result<()> {
        if let Some(disk_handle) = &self.disk_handle {
            disk_handle.write(relative_addr, value)
        } else {
            bail!(
                "There was no disk handle assigned to this core, id:'{}'",
                self.id
            );
        }
    }
}
