use crate::emulator::thread::Thread;
use anyhow::{Result, bail};
use log::{debug, info};
const DEVICE_ID_MASK: u32 = 0xFF00000;
impl Thread {
    pub fn handle_io_write(&mut self, relative_addr: u32, value: u32) -> Result<()> {
        debug!("handle_io_write relative_addr{relative_addr:#x}");
        let id = (relative_addr & DEVICE_ID_MASK) >> 20;
        let relative_addr = relative_addr & 0xFFFFF;
        match id {
            0 => self.handle_disk_write(relative_addr, value),
            1 => self.handle_timer_write(relative_addr, value),
            2 => self.handle_audio_write(relative_addr, value),
            3 => self.handle_serial_write(relative_addr, value),
            _ => bail!("device with id:{id} is not yet implemented"),
        }
    }
    fn handle_timer_write(&self, relative_addr: u32, value: u32) -> Result<()> {
        todo!("timer is not yet implemented");
    }
    fn handle_audio_write(&self, relative_addr: u32, value: u32) -> Result<()> {
        todo!("audio is not yet implemented");
    }
    fn handle_serial_write(&mut self, relative_addr: u32, value: u32) -> Result<()> {
        debug!("handle_serial_write buffer:'{}'", self.serial_buffer);
        match relative_addr {
            0x00 => {
                let char = value as u8 as char;
                if char == '\n' {
                    info!("|>'{:#}'", self.serial_buffer);
                    self.serial_buffer.clear();
                } else {
                    self.serial_buffer.push(char);
                }
            }
            _ => bail!(
                "writing to '{relative_addr}' address for the serial device wasn't implemented yet!"
            ),
        };
        Ok(())
    }
    fn handle_disk_write(&self, relative_addr: u32, value: u32) -> Result<()> {
        todo!("disk is not yet implemented");
    }
}
