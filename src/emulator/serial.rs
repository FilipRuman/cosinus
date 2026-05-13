use anyhow::{Context, Result, bail};
use log::info;

use crate::emulator::memory::MEMORY;

pub static mut SERIAL_REGISTERS: SerialRegisters = SerialRegisters {
    output_buffer_address: 0,
    output_bytes_count: 0,
};
pub struct SerialRegisters {
    output_buffer_address: u32,
    output_bytes_count: u32,
}

pub fn handle_serial_write(relative_addr: u32, value: u32) -> Result<()> {
    match relative_addr {
        0x00 => unsafe { SERIAL_REGISTERS.output_buffer_address = value },
        0x04 => unsafe { SERIAL_REGISTERS.output_bytes_count = value },
        0x0C => {
            let bytes = unsafe {
                MEMORY.read_vec::<u8>(
                    SERIAL_REGISTERS.output_buffer_address,
                    SERIAL_REGISTERS.output_bytes_count as usize,
                )
            };
            let str = String::from_utf8(bytes.clone()).with_context(|| {
                format!(
                    "Converting bytes to utf-8 for serial printing did not succeed, bytes:'{bytes:?}'"
                )
            })?;
            info!("|>{str}");
        }
        _ => bail!(
            "writing to '{relative_addr}' address for the serial device wasn't implemented yet!"
        ),
    };
    Ok(())
}
