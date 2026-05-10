use std::env;

use ::log::info;
use anyhow::{Context, Result};

use crate::{
    dir_handling::project_dir,
    emulator::disk::{self, DISK, Extent, SuperBlock, helpers::DiskWriteStream},
    log::init_log,
};

mod assembler;
pub mod dir_handling;
pub mod emulator;
pub mod linker;
pub mod log;
mod tests;

#[tokio::main]
async fn main() -> Result<()> {
    init_log();
    let mut args = env::args();
    args.next();
    if let Some(arg) = args.next() {
        match arg.as_str() {
            "emulate_clean" => {
                emulate().await?;
                init_fs()?;
                setup_code()?
            }
            "emulate" => emulate().await?,
            "init_fs" => init_fs()?,
            "setup_code" => setup_code()?,
            arg => {
                info!("argument was invalid: {arg:?}");
                print_help()
            }
        }
    } else {
        print_help();
    }
    Ok(())
}
fn init_fs() -> Result<()> {
    emulator::disk::init_file_system(SuperBlock {
        total_inodes: 122880,
        data_blocks_per_group: 8 * 3000,
        inodes_per_group: 60,
        group_count: 2048,
        inode_size: size_of::<crate::disk::inode::INode>() as u32,
        flags: 0,
        boot_code_block_count: 0,
        boot_code_block_base_index: 0,
    })
}

fn setup_boot_code() -> Result<()> {
    DISK.mark_extend_as_free_data_blocks(Extent {
        base_block_index: DISK.superblock.boot_code_block_base_index,
        block_count: DISK.superblock.boot_code_block_count,
    })?;
    let boot_code_path = project_dir("code/boot");
    const IS_BOOT_CODE: bool = true;
    let boot_code = linker::generate_elf_for_dir(boot_code_path, IS_BOOT_CODE)
        .context("generating elf for boot code")?;
    let mut boot_code_bytes: Vec<u8> = bytemuck::cast_slice(&boot_code).to_vec();
    let boot_code_blocks = (boot_code_bytes.len()).div_ceil(disk::BLOCK_SIZE_BYTES as usize);
    boot_code_bytes.resize(boot_code_blocks * disk::BLOCK_SIZE_BYTES as usize, 0);
    let alloc = DISK.allocate_blocks(boot_code_blocks as u32)?;
    DISK.write_extent(alloc, &boot_code_bytes)?;

    let mut superblock = DISK.superblock.clone();
    superblock.boot_code_block_base_index = alloc.base_block_index;
    superblock.boot_code_block_count = alloc.block_count;
    let mut super_block_bytes = bytemuck::bytes_of(&superblock).to_vec();
    super_block_bytes.resize(disk::BLOCK_SIZE_BYTES as usize, 0);
    DISK.write_extent(Extent::single(0), &super_block_bytes)?;

    // I can skip writing to the DISK's superblock value because it doesn't care about boot code
    // anyway.
    Ok(())
}
fn setup_code() -> Result<()> {
    setup_boot_code().context("setup boot code")
}
fn print_help() {
    todo!()
}
async fn emulate() -> Result<()> {
    {
        let bios_code_path = project_dir("code/bios");
        const IS_BOOT_CODE: bool = true;
        let bios_code = linker::generate_elf_for_dir(bios_code_path, IS_BOOT_CODE)
            .context("generating elf for boot code")?;
        unsafe {
            emulator::write_instructions_to_memory(0, bios_code);
        }
    }
    emulator::run().await?;

    loop {}
}
