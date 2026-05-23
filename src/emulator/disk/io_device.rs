use anyhow::{Result, anyhow, bail};
use log::{debug, error, info};
use tokio::sync::mpsc;

use crate::emulator::{
    self,
    disk::{BLOCK_SIZE_BYTES, DISK, Extent},
    interrupts::InterruptType,
    memory::MEMORY,
};

#[derive(Clone, Copy, Debug)]
pub struct DiskOp {
    pub local_addr: u32,
    pub value: u32,
}

#[derive(Clone)]
pub struct DiskHandle {
    tx: mpsc::Sender<DiskOp>,
}

impl DiskHandle {
    pub fn write(&self, local_addr: u32, value: u32) -> Result<()> {
        let op = DiskOp { local_addr, value };

        self.tx
            .try_send(op)
            .map_err(|e| anyhow!("channel send failed: {e}"))?;

        Ok(())
    }
}

fn create_disk_channel() -> (DiskHandle, mpsc::Receiver<DiskOp>) {
    let (tx, rx) = mpsc::channel::<DiskOp>(10_000);

    (DiskHandle { tx }, rx)
}
pub static mut DISK_STATUS_REGISTER: u32 = 1u32;

pub async fn run_disk_loop(mut rx: mpsc::Receiver<DiskOp>) -> Result<()> {
    let mut disk_registers = DiskRegisters {
        base_block_index: 0,
        buffer_address: 0,
        block_count: 0,
        core_id_to_interrupt: 0,
        last_interrupt_acknowledged: true,
        threads_to_interrupt: vec![],
    };
    info!("run_disk_loop - init");

    while let Some(op) = rx.recv().await {
        apply_op(&mut disk_registers, op);
        debug!(
            "run_disk_loop -apply_op: op:'{op:?}', registers:'{:?}'",
            disk_registers
        );
        if disk_registers.last_interrupt_acknowledged
            && let Some(thread_interrupt_data) = disk_registers.threads_to_interrupt.pop()
        {
            debug!("run_disk_loop - interrupt");
            disk_registers.last_interrupt_acknowledged = false;
            unsafe {
                DISK_STATUS_REGISTER = thread_interrupt_data.status_register_data;
            }
            let core =
                unsafe { &mut emulator::core::CORES[thread_interrupt_data.core_id as usize] };
            core.trigger_interrupt(InterruptType::Drive);
        }
    }
    Ok(())
}
#[derive(Debug)]
struct CoreInterruptData {
    pub core_id: u32,
    pub status_register_data: u32,
}
#[derive(Debug)]
struct DiskRegisters {
    last_interrupt_acknowledged: bool,
    threads_to_interrupt: Vec<CoreInterruptData>,
    base_block_index: u32,
    buffer_address: u32,
    block_count: u32,
    core_id_to_interrupt: u32,
}

#[inline]
fn apply_op(disk_registers: &mut DiskRegisters, op: DiskOp) {
    match op.local_addr {
        0x00 => disk_registers.base_block_index = op.value,
        0x04 => disk_registers.buffer_address = op.value,
        0x08 => disk_registers.block_count = op.value,
        0x0C => disk_registers.core_id_to_interrupt = op.value,

        0x14 => {
            if let Err(err) = handle_control_register_write(disk_registers, op) {
                error!("encountered an error while handling write to the control register: {err}")
            }
        }
        other => error!("writing to disk register with local address of:'{other}' is invalid",),
    }
}
fn handle_control_register_write(disk_registers: &mut DiskRegisters, op: DiskOp) -> Result<()> {
    let operation_type = op.value & 0b11;
    match operation_type {
        0b00 => {
            // read
            let bytes = DISK.read_extent(Extent {
                base_block_index: disk_registers.base_block_index,
                block_count: disk_registers.block_count,
            })?;

            debug!(
                "handle_control_register_write- READ: bytes:{bytes:?}, buffer_address:{:#x}",
                disk_registers.buffer_address
            );
            unsafe {
                MEMORY.write_vec(disk_registers.buffer_address, bytes);
            }

            let command_id = op.value & !0b11 << 8;
            disk_registers.threads_to_interrupt.push(CoreInterruptData {
                core_id: disk_registers.core_id_to_interrupt,
                status_register_data: 0b1 | command_id,
            });
        }

        0b10 => {
            // write
            //
            let bytes: Vec<u8> = unsafe {
                MEMORY.read_vec(
                    disk_registers.buffer_address,
                    (disk_registers.block_count * BLOCK_SIZE_BYTES) as usize,
                )
            };
            debug!(
                "handle_control_register_write- WRITE: bytes:{bytes:?}, buffer_address:{:#x}",
                disk_registers.buffer_address
            );
            DISK.write_extent(
                Extent {
                    base_block_index: disk_registers.base_block_index,
                    block_count: disk_registers.block_count,
                },
                &bytes,
            )?;

            let command_id = op.value & !0b11 << 8;
            disk_registers.threads_to_interrupt.push(CoreInterruptData {
                core_id: disk_registers.core_id_to_interrupt,
                status_register_data: 0b1 | command_id,
            });
        }
        0b01 => {
            // acknowledge
            disk_registers.last_interrupt_acknowledged = true;
            debug!(
                "handle_control_register_write- last_interrupt_acknowledged:{}",
                disk_registers.last_interrupt_acknowledged
            );
        }
        other => bail!("Operation type is invalid:'{other}'"),
    };

    Ok(())
}
pub fn init() -> Result<(DiskHandle, mpsc::Receiver<DiskOp>)> {
    let (disk_handle, rx) = create_disk_channel();

    Ok((disk_handle, rx))
}
