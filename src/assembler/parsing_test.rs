#[cfg(test)]
mod test {
    use crate::assembler::{
        instruction::{Command, Immediate, Instruction, Macro},
        instruction_parsing,
    };
    use anyhow::{Result, anyhow};

    pub fn assert_instr_eq(actual: &[Command], expected: &[Command]) -> Result<()> {
        let mut ok = true;
        let mut report = String::new();

        let len = actual.len().max(expected.len());

        for i in 0..len {
            match (actual.get(i), expected.get(i)) {
                (Some(a), Some(e)) if a == e => {
                    // fine
                }
                (Some(a), Some(e)) => {
                    ok = false;
                    report.push_str(&format!(
                        "\n[{}] MISMATCH\n  actual  = {:?}\n  expected= {:?}\n",
                        i, a, e
                    ));
                }
                (Some(a), None) => {
                    ok = false;
                    report.push_str(&format!("\n[{}] EXTRA ACTUAL\n  {:?}\n", i, a));
                }
                (None, Some(e)) => {
                    ok = false;
                    report.push_str(&format!("\n[{}] MISSING ACTUAL\n  expected = {:?}\n", i, e));
                }
                (None, None) => unreachable!(),
            }
        }

        if ok {
            return Ok(());
        }

        Err(anyhow!(
            "instruction mismatch (len actual={}, expected={}){}",
            actual.len(),
            expected.len(),
            report
        ))
    }
    #[test]
    fn test_instruction_parsing() -> Result<()> {
        let instr = instruction_parsing::parse_program(
            "
jmp jump_label
or r5 r5 25
set32 r25 0x2222333
set32 r25 -2236979
beq r10 r5 253
scall
sel 30 20 10 3
:jump_label
halt
",
        )?;
        assert_instr_eq(
            &instr,
            &vec![
                Instruction::JMP {
                    imm: Immediate::Label("jump_label".to_string()),
                }
                .into(),
                Instruction::OR {
                    rd: 5,
                    rs1: 5,
                    imm: Immediate::Direct(25),
                }
                .into(),
                Macro::Set32 {
                    rd: 25,
                    imm: Immediate::Direct(0x2222333i32 as i32),
                }
                .into(),
                Macro::Set32 {
                    rd: 25,
                    imm: Immediate::Direct(-0x222233i32 as i32),
                }
                .into(),
                Instruction::BEQ {
                    rs1: 10,
                    rs2: 5,
                    imm: Immediate::Direct(253),
                }
                .into(),
                Instruction::SCALL.into(),
                Instruction::SEL {
                    rd: 30,
                    rs1: 20,
                    rs2: 10,
                    rs3: 3,
                }
                .into(),
                Command::Label("jump_label".to_string()),
                Instruction::HALT.into(),
            ],
        )?;
        Ok(())
    }
    #[test]
    fn test_arithmetic_and_logical_parsing() -> Result<()> {
        let instr = instruction_parsing::parse_program(
            "
addr r1 r2 r3
subr r4 r4 r5
and r6 r6 0xFF
or r7 r7 10
xor r8 r8 1
mul r9 r9 2
div r10 r10 3
rem r11 r11 4
shl r12 r12 1
shr r13 r13 2
sar r14 r14 3
not r15 r15
lui r16 0x1234
",
        )?;

        assert_instr_eq(
            &instr,
            &vec![
                Instruction::ADDR {
                    rd: 1,
                    rs1: 2,
                    rs2: 3,
                }
                .into(),
                Instruction::SUBR {
                    rd: 4,
                    rs1: 4,
                    rs2: 5,
                }
                .into(),
                Instruction::AND {
                    rd: 6,
                    rs1: 6,
                    imm: Immediate::Direct(0xFF),
                }
                .into(),
                Instruction::OR {
                    rd: 7,
                    rs1: 7,
                    imm: Immediate::Direct(10),
                }
                .into(),
                Instruction::XOR {
                    rd: 8,
                    rs1: 8,
                    imm: Immediate::Direct(1),
                }
                .into(),
                Instruction::MUL {
                    rd: 9,
                    rs1: 9,
                    imm: Immediate::Direct(2),
                }
                .into(),
                Instruction::DIV {
                    rd: 10,
                    rs1: 10,
                    imm: Immediate::Direct(3),
                }
                .into(),
                Instruction::REM {
                    rd: 11,
                    rs1: 11,
                    imm: Immediate::Direct(4),
                }
                .into(),
                Instruction::SHL {
                    rd: 12,
                    rs1: 12,
                    imm: Immediate::Direct(1),
                }
                .into(),
                Instruction::SHR {
                    rd: 13,
                    rs1: 13,
                    imm: Immediate::Direct(2),
                }
                .into(),
                Instruction::SAR {
                    rd: 14,
                    rs1: 14,
                    imm: Immediate::Direct(3),
                }
                .into(),
                Instruction::NOT { rd: 15, rs1: 15 }.into(),
                Instruction::LUI {
                    rd: 16,
                    imm: Immediate::Direct(0x1234),
                }
                .into(),
            ],
        )?;

        Ok(())
    }

    #[test]
    fn test_memory_and_control_flow_parsing() -> Result<()> {
        let instr = instruction_parsing::parse_program(
            "
load r1 r2 16
store r3 r4 32
loadb r5 r6 1
storeb r7 r8 2
loadh r9 r10 4
storeh r11 r12 8
loadpc r13 64

jmp target
call target
ret
jmpr r1 8
apc r2 12

:target
halt
",
        )?;

        assert_instr_eq(
            &instr,
            &vec![
                Instruction::LOAD {
                    rd: 1,
                    rs1: 2,
                    imm: Immediate::Direct(16),
                }
                .into(),
                Instruction::STORE {
                    rs1: 3,
                    rs2: 4,
                    imm: Immediate::Direct(32),
                }
                .into(),
                Instruction::LOADB {
                    rd: 5,
                    rs1: 6,
                    imm: Immediate::Direct(1),
                }
                .into(),
                Instruction::STOREB {
                    rs1: 7,
                    rs2: 8,
                    imm: Immediate::Direct(2),
                }
                .into(),
                Instruction::LOADH {
                    rd: 9,
                    rs1: 10,
                    imm: Immediate::Direct(4),
                }
                .into(),
                Instruction::STOREH {
                    rs1: 11,
                    rs2: 12,
                    imm: Immediate::Direct(8),
                }
                .into(),
                Instruction::LOADPC {
                    rd: 13,
                    imm: Immediate::Direct(64),
                }
                .into(),
                Instruction::JMP {
                    imm: Immediate::Label("target".to_string()),
                }
                .into(),
                Instruction::CALL {
                    imm: Immediate::Label("target".to_string()),
                }
                .into(),
                Instruction::RET.into(),
                Instruction::JMPR {
                    rs1: 1,
                    imm: Immediate::Direct(8),
                }
                .into(),
                Instruction::APC {
                    rd: 2,
                    imm: Immediate::Direct(12),
                }
                .into(),
                Command::Label("target".to_string()),
                Instruction::HALT.into(),
            ],
        )?;

        Ok(())
    }

    #[test]
    fn test_branch_syscall_and_macros() -> Result<()> {
        let instr = instruction_parsing::parse_program(
            "
beq r1 r2 10
bne r3 r4 20
blt r5 r6 30
bgt r7 r8 40
ble r9 r10 50
bge r11 r12 60

scall
sret

sysr r1 3
sysw r2 4

add r5 r0 42
set32 r6 0x1122334
",
        )?;

        assert_instr_eq(
            &instr,
            &vec![
                Instruction::BEQ {
                    rs1: 1,
                    rs2: 2,
                    imm: Immediate::Direct(10),
                }
                .into(),
                Instruction::BNE {
                    rs1: 3,
                    rs2: 4,
                    imm: Immediate::Direct(20),
                }
                .into(),
                Instruction::BLT {
                    rs1: 5,
                    rs2: 6,
                    imm: Immediate::Direct(30),
                }
                .into(),
                Instruction::BGT {
                    rs1: 7,
                    rs2: 8,
                    imm: Immediate::Direct(40),
                }
                .into(),
                Instruction::BLE {
                    rs1: 9,
                    rs2: 10,
                    imm: Immediate::Direct(50),
                }
                .into(),
                Instruction::BGE {
                    rs1: 11,
                    rs2: 12,
                    imm: Immediate::Direct(60),
                }
                .into(),
                Instruction::SCALL.into(),
                Instruction::SRET.into(),
                Instruction::SYSR {
                    rd: 1,
                    imm: Immediate::Direct(3),
                }
                .into(),
                Instruction::SYSW {
                    rs1: 2,
                    imm: Immediate::Direct(4),
                }
                .into(),
                Instruction::ADD {
                    rd: 5,
                    rs1: 0,
                    imm: Immediate::Direct(42),
                }
                .into(),
                Macro::Set32 {
                    rd: 6,
                    imm: Immediate::Direct(0x1122334),
                }
                .into(),
            ],
        )?;

        Ok(())
    }
}
