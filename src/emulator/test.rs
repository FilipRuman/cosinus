#[cfg(test)]
pub mod tests {
    use anyhow::{Context, Result};
    use log::{debug, info};

    use crate::{
        assembler::{
            self,
            instruction::{Immediate, Instruction, Macro},
        },
        emulator::{self, interrupts::InterruptType, memory::MEMORY, thread::Thread},
        log::init_log,
    };

    // This needs to be done sequentially because of the common memory architecture
    #[test]
    pub fn run_emulator_tests() -> Result<()> {
        init_log();
        instant_halt()?;
        set()?;
        arithmetic()?;
        memory()?;
        control_flow()?;
        compare()?;
        branching()?;
        syscalls()?;
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
                imm: Immediate::Direct(TEST_VAL as i32),
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
add r5 r0 25
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
        let instructions = assembler::assemble_from_string(
            "
add r5 r0 25
store r0 r5  0xFFF

add r5 r0 30
store r0 r5 0x1FFF

set32 r5 0xCB2AFF2F 
storeb r0 r5 0x2FFF # only 0x2F should be written
storeh r0 r5 0x3FFF # only 0xFF2F should be written
store r0 r5 0x4FFF # only 0xCB2AFF2F  should be written

load r5 r0 0x4FFF # only 0xCB2AFF2F  should be read
loadb r6 r0 0x4FFF# only 0x2F  should be read
loadh r7 r0 0x4FFF# only 0xFF2F should be read

set32 r8 0xF0000001 # nothing should be written to this addr because the privilege mode is user
add r9 r0 70
store r8 r9 0

halt
",
        )
        .context("assembling the test instructions")?;
        let thread = emulator::run_test(instructions.clone());
        assert_eq!(unsafe { MEMORY.read(0xFFF) }, 25);
        assert_eq!(unsafe { MEMORY.read(0x1FFF) }, 30);
        assert_eq!(unsafe { MEMORY.read(0x2FFF) }, 0x2F);
        assert_eq!(unsafe { MEMORY.read(0x3FFF) }, 0xFF2F);
        assert_eq!(unsafe { MEMORY.read(0x4FFF) }, 0xCB2AFF2Fu32 as i32);
        // load
        assert_eq!(thread.gpr[5], 0xCB2AFF2Fu32 as i32);
        assert_eq!(thread.gpr[6], 0x2F);
        assert_eq!(thread.gpr[7], 0xFF2Fu16 as i16 as i32);
        // privilege test
        assert_eq!(unsafe { MEMORY.read(0xF0000001) }, 0 as i32);

        Ok(())
    }

    fn control_flow() -> Result<()> {
        // |  U   |  JMP   | 0x1E | imm26     | PC += imm       |
        // |  U   |  CALL  | 0x1F | imm26     | ra = PC+4; jump |
        // |  U   |  RET   | 0x20 | —         | PC = ra         |
        // |  U   |  JMPR  | 0x21 | rs, imm16 | PC = rs + imm   |
        // |  U   |  APC   | 0x22 | rd, imm16 | rd = PC + imm   |
        let instructions = assembler::assemble_from_string(
            "
call one
or r6 r0 2
jmp three
halt # r10 = here
or r8 r0 4
halt
halt
halt
halt
halt
halt
halt
halt
:one
or r5 r0 1
ret
:three
or r7 r0 3
apc r10 -52 # byte space so 13 * 4
jmpr r10 0
halt
",
        )
        .context("assembling the test instructions")?;
        let thread = emulator::run_test(instructions.clone());

        assert_eq!(thread.gpr[5], 1);
        assert_eq!(thread.gpr[6], 2);
        assert_eq!(thread.gpr[7], 3);
        assert_eq!(thread.gpr[8], 4);
        Ok(())
    }

    fn branching() -> Result<()> {
        // |  U   |  BEQ   | 0x23 | rs1, rs2, imm16 | if rs1 equal rs2     |
        // |  U   |  BNE   | 0x24 | rs1, rs2, imm16 | if rs1 not equal rs2 |
        // |  U   |  BLT   | 0x25 | rs1, rs2, imm16 | if rs1 less rs2      |
        // |  U   |  BGT   | 0x26 | rs1, rs2, imm16 | if rs1 greater rs2   |
        // |  U   |  BLE   | 0x27 | rs1, rs2, imm16 | if rs1 <= rs2        |
        // |  U   |  BGE   | 0x28 | rs1, rs2, imm16 | if rs1 >= rs2        |
        let instructions = assembler::assemble_from_string(
            "
add r5 r0 25
add r6 r0 -25
beq r5 r6 end
bne r5 r6 skip_r20
add r20 r0 1 # r20 should = 0
:skip_r20
add r7 r0 0
:loop_start
add r7 r7 1
BLT r7 r5 loop_start # r7 should = 25
BGT r6 r5 end
add r10 r0 1 # r10 should = 1
:end
halt
",
        )
        .context("assembling the test instructions")?;
        let thread = emulator::run_test(instructions.clone());

        assert_eq!(thread.gpr[20], 0);
        assert_eq!(thread.gpr[7], 25);
        assert_eq!(thread.gpr[10], 1);
        Ok(())
    }

    fn syscalls() -> Result<()> {
        info!("----------");
        const SYSCALL_FUNC_ADDR: i32 = 0xF0000001u32 as i32;
        const IVT_ADDR: i32 = 0xF0100000u32 as i32;
        unsafe { MEMORY.read(IVT_ADDR as usize) }; // read test
        const IVT_SYSCALL_ADDR: usize =
            IVT_ADDR as u32 as usize + InterruptType::Syscall as usize * 4;
        unsafe { MEMORY.write(IVT_SYSCALL_ADDR, SYSCALL_FUNC_ADDR) };
        unsafe { MEMORY.write(IVT_SYSCALL_ADDR, SYSCALL_FUNC_ADDR) };
        let syscall_instructions = assembler::assemble_from_string(
            "
add r5 r0 10
SYSR r6 0 # psr
sret
",
        )
        .context("assembling the test syscall instructions")?;
        unsafe {
            emulator::write_instructions_to_memory(SYSCALL_FUNC_ADDR as u32, syscall_instructions)
        };

        let instructions = assembler::assemble_from_string(
            "
scall
halt
",
        )
        .context("assembling the test instructions")?;
        unsafe {
            emulator::write_instructions_to_memory(0, instructions);
        }
        let mut thread = Thread::new(0, None);
        thread.psr = 0b11;
        thread.ivt = IVT_ADDR;
        thread.run_test_loop();
        assert_eq!(thread.gpr[5], 10);
        assert_eq!(thread.gpr[6], 0b11); // Privilege Mode = T Interrupt Enable = T HALT = F  

        Ok(())
    }

    // D] handle_instruction optcode:0x29;00000000000000000000000000101001 r0:0x0 r1:0x0 r2:0x0 r3:0x0 r2_imm16:0x0 r1_imm16:0x0 imm26:0x0
    // [T] pc: 4
    // [D] handle_instruction optcode:0xb;00000000000000000000000000001011 r0:0x5 r1:0x0 r2:0x0 r3:0x0 r2_imm16:0x1 r1_imm16:0x0 imm26:0xa00001
    // [T] pc: 8
    // [D] handle_instruction optcode:0x30;00000000000000000000000000110000 r0:0x0 r1:0x0 r2:0x0 r3:0x0 r2_imm16:0x0 r1_imm16:0x0 imm26:0x0
    // [T] pc: 12
    // [*] RUN TEST LOOP!
    // [D] handle_instruction optcode:0x29;00000000000000000000000000101001 r0:0x0 r1:0x0 r2:0x0 r3:0x0 r2_imm16:0x0 r1_imm16:0x0 imm26:0x0
    // [T] pc: 4
    // [D] handle_instruction optcode:0x30;00000000000000000000000000110000 r0:0x0 r1:0x0 r2:0x0 r3:0x0 r2_imm16:0x0 r1_imm16:0x0 imm26:0x0
    // [T] pc: 8
    // test emulator::test::tests::run_emulator_tests ... FAILED
    //
    fn interrupts() -> Result<()> {
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
        let instructions = assembler::assemble_from_string(
            "
add r20 r0 20
add r21 r0 -20
add r4 r0 10
LTR r5  r4 r20 # T
add r4 r0 -20
EQR r7  r4 r20 # F
LT r8 r4 20 # T 
EQ r9 r4 -20 # T

add r20 r0 2525 # = True
add r21 r0 0    # = False
sel r12 r20 r21 r20# r12 = r20 = 2525  

set32 r21 9876 # 0000 0000 0000 0000 0010 0110 1001 0100
clz r13 r21 # 2  
ctz r14 r21 # 18  

set32 r20 1234 # 0000 0000 0000 0000  0000 0100 1101 0010
clz r15 r20 # 1  
ctz r16 r20 # 21 
halt
",
        )
        .context("assembling the test instructions")?;
        let thread = emulator::run_test(instructions.clone());
        assert_eq!(thread.gpr[5], 1);
        assert_eq!(thread.gpr[7], 0);
        assert_eq!(thread.gpr[8], 1);
        assert_eq!(thread.gpr[9], 1);
        assert_eq!(thread.gpr[12], 2525);
        assert_eq!(thread.gpr[13], 18);
        assert_eq!(thread.gpr[14], 2);
        assert_eq!(thread.gpr[15], 21);
        assert_eq!(thread.gpr[16], 1);
        Ok(())
    }
}
