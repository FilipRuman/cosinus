use anyhow::{Context, Result};

use crate::{dir_handling::project_dir, log::init_log};

mod assembler;
pub mod dir_handling;
pub mod emulator;
pub mod linker;
pub mod log;
mod tests;

#[tokio::main]
async fn main() -> Result<()> {
    init_log();

    {
        let boot_code_path = project_dir("code/boot");
        const IS_BOOT_CODE: bool = true;
        let boot_code = linker::generate_elf_for_dir(boot_code_path, IS_BOOT_CODE)
            .context("generating elf for boot code")?;
        unsafe {
            emulator::write_instructions_to_memory(0, boot_code);
        }
    }
    emulator::run().await?;

    loop {}
}
