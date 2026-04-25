#[cfg(test)]
pub mod tests {
    use anyhow::{Context, Result};
    use log::{debug, info};

    use crate::{
        assembler::{
            self,
            instruction::{Immediate, Instruction, Macro},
        },
        emulator::{self, memory::MEMORY, thread::Thread},
        log::init_log,
    };

    // This needs to be done sequencially
    #[test]
    pub fn run_emulator_tests() -> Result<()> {
        init_log();
        // MEMORY.clear();
        instant_halt()?;
        set()?;
        arithmetic()?;
        Ok(())
    }

    fn instant_halt() -> Result<()> {
        let instructions = assembler::assemble_commands(vec![Instruction::HALT {}.into()])
            .context("assembling the test instructions")?;
        let _ = emulator::run_test(instructions.clone());
        let first_instruction = unsafe { emulator::memory::MEMORY.read(0) };

        debug!(
            "first_instruction: {:#x}, asembled_instruction:{:#x}",
            first_instruction as u32, instructions[0] as u32
        );
        assert_eq!(first_instruction, instructions[0]);
        Ok(())
    }

    fn set() -> Result<()> {
        const TEST_VAL: i32 = 25;
        let instructions = assembler::assemble_commands(vec![
            Macro::Set32 {
                rd: 5,
                imm: Immediate::Direct(TEST_VAL as u32),
            }
            .into(),
            Instruction::HALT {}.into(),
        ])
        .context("assembling the test instructions")?;
        let thread = emulator::run_test(instructions.clone());
        assert_eq!(thread.gpr[5], TEST_VAL);
        Ok(())
    }
    fn arithmetic() -> Result<()> {
        const TEST_VAL_5: i32 = ((25 * 5 / 25 - 321) * -21 + 3) << 5;
        let instructions = assembler::assemble_from_string(
            "
set r5 25
mul r5 r5 5
div r5 r5 25
sub r5 r5 321
mul r5 r5 -21
add r5 r5 3
shl r5 r5 5
halt
",
        )
        .context("assembling the test instructions")?;
        let thread = emulator::run_test(instructions.clone());
        assert_eq!(thread.gpr[5], TEST_VAL_5);
        Ok(())
    }
    fn memory() -> Result<()> {
        todo!()
    }

    fn control_flow() -> Result<()> {
        todo!()
    }

    fn branching() -> Result<()> {
        todo!()
    }

    fn syscalls() -> Result<()> {
        todo!()
    }

    fn system_regs() -> Result<()> {
        todo!()
    }

    fn devices() -> Result<()> {
        todo!()
    }
    fn frame_buffer() -> Result<()> {
        todo!()
    }

    fn privileges() -> Result<()> {
        todo!()
    }

    fn atomic() -> Result<()> {
        todo!()
    }

    fn compare() -> Result<()> {
        todo!()
    }
}
