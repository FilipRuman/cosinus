use anyhow::Result;
mod assembler;
pub mod compiler;
pub mod emulator;
pub mod log;
mod tests;

#[tokio::main]
async fn main() -> Result<()> {
    colog::init();
    unsafe {
        let assembled = assembler::assemble_from_string("halt")?;
        emulator::write_instructions_to_memory(0, assembled);
    }
    emulator::run().await?;

    loop {}
}
