use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::log::init_log;

mod assembler;
pub mod emulator;
pub mod linker;
pub mod log;
mod tests;

#[tokio::main]
async fn main() -> Result<()> {
    init_log();

    let project_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut code_dir_path = project_dir.clone();
    code_dir_path.push("code");
    {
        let mut boot_code_path = code_dir_path.clone();
        boot_code_path.push("boot");
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
