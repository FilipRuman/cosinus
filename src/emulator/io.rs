use crate::emulator::thread::Thread;
use anyhow::{Context, Result, bail};
const DEVICE_ID_MASK: u32 = 0xFF00000;
impl Thread {
    pub fn handle_io_write(&self, relative_addr: u32, value: u32) -> Result<()> {
        let id = (relative_addr & DEVICE_ID_MASK) << 20;
        match id {
            0 => bail!("disk is not yet implemented"),
            1 => bail!("timer is not yet implemented"),
            2 => bail!("audio is not yet implemented"),
            3 => bail!("serial is not yet implemented"),
            _ => bail!("device with id:{id} is not yet implemented"),
        };
    }
}
