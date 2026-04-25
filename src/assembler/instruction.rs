use std::collections::HashMap;

use anyhow::{Context, Result};

type Label = String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Immediate<T> {
    Direct(T),
    Label(Label),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Macro {
    /// 32-bit immediate load:
    /// LUI rd, upper
    /// OR  rd, rd, lower
    Set32 { rd: u8, imm: Immediate<u32> },

    /// 16-bit immediate load:
    /// LUI rd, upper
    /// OR  rd, rd, lower
    Set { rd: u8, imm: Immediate<u16> },
}
impl Into<Command> for Macro {
    fn into(self) -> Command {
        Command::Macro(self)
    }
}

// impl Into<Command> for String {
//     fn into(self) -> Command {
//
//     }
// }
impl Into<Vec<Instruction>> for &Macro {
    fn into(self) -> Vec<Instruction> {
        match self {
            Macro::Set { rd, imm } => {
                vec![Instruction::ADD {
                    rd: *rd,
                    rs1: 0,
                    imm: imm.clone(),
                }]
            }

            Macro::Set32 { rd, imm } => {
                vec![
                    Instruction::LUI {
                        rd: *rd,
                        imm: match imm {
                            Immediate::Direct(v) => Immediate::Direct((v >> 16) as u16),
                            Immediate::Label(l) => Immediate::Label(l.to_string()),
                        },
                    },
                    Instruction::OR {
                        rd: *rd,
                        rs1: *rd,
                        imm: match imm {
                            Immediate::Direct(v) => Immediate::Direct((v & 0xFFFF) as u16),
                            Immediate::Label(l) => Immediate::Label(l.to_string()),
                        },
                    },
                ]
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Instr(Instruction),
    Label(Label),
    Macro(Macro),
    RawData(Vec<i32>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    // ===== R-type ============
    /// rd = rs1 + rs2
    ADDR {
        rd: u8,
        rs1: u8,
        rs2: u8,
    },

    /// rd = rs1 - rs2
    SUBR {
        rd: u8,
        rs1: u8,
        rs2: u8,
    },

    /// rd = rs1 & rs2
    ANDR {
        rd: u8,
        rs1: u8,
        rs2: u8,
    },

    /// rd = rs1 | rs2
    ORR {
        rd: u8,
        rs1: u8,
        rs2: u8,
    },

    /// rd = rs1 ^ rs2
    XORR {
        rd: u8,
        rs1: u8,
        rs2: u8,
    },

    /// rd = rs1 * rs2
    MULR {
        rd: u8,
        rs1: u8,
        rs2: u8,
    },

    /// rd = rs1 / rs2
    DIVR {
        rd: u8,
        rs1: u8,
        rs2: u8,
    },

    /// rd = rs1 % rs2
    REMR {
        rd: u8,
        rs1: u8,
        rs2: u8,
    },

    /// rd = rs1 << rs2
    SHLR {
        rd: u8,
        rs1: u8,
        rs2: u8,
    },

    /// rd = rs1 >> rs2 (logical)
    SHRR {
        rd: u8,
        rs1: u8,
        rs2: u8,
    },

    /// rd = rs1 >> rs2 (arithmetic)
    SARR {
        rd: u8,
        rs1: u8,
        rs2: u8,
    },

    // ===== I-type ============
    /// rd = rs1 + imm
    ADD {
        rd: u8,
        rs1: u8,
        imm: Immediate<u16>,
    },

    /// rd = rs1 - imm
    SUB {
        rd: u8,
        rs1: u8,
        imm: Immediate<u16>,
    },

    /// rd = rs1 & imm
    AND {
        rd: u8,
        rs1: u8,
        imm: Immediate<u16>,
    },

    /// rd = rs1 | imm
    OR {
        rd: u8,
        rs1: u8,
        imm: Immediate<u16>,
    },

    /// rd = rs1 ^ imm
    XOR {
        rd: u8,
        rs1: u8,
        imm: Immediate<u16>,
    },

    /// rd = rs1 * imm
    MUL {
        rd: u8,
        rs1: u8,
        imm: Immediate<u16>,
    },

    /// rd = rs1 / imm
    DIV {
        rd: u8,
        rs1: u8,
        imm: Immediate<u16>,
    },

    /// rd = rs1 % imm
    REM {
        rd: u8,
        rs1: u8,
        imm: Immediate<u16>,
    },

    /// rd = rs1 << imm
    SHL {
        rd: u8,
        rs1: u8,
        imm: Immediate<u16>,
    },

    /// rd = rs1 >> imm (logical)
    SHR {
        rd: u8,
        rs1: u8,
        imm: Immediate<u16>,
    },

    /// rd = rs1 >> imm (arithmetic)
    SAR {
        rd: u8,
        rs1: u8,
        imm: Immediate<u16>,
    },

    /// Load upper immediate
    LUI {
        rd: u8,
        imm: Immediate<u16>,
    },

    // ===== Memory ============
    /// rd = *(rs1 + imm)
    LOAD {
        rd: u8,
        rs1: u8,
        imm: Immediate<u16>,
    },

    /// *(rs1 + imm) = rs2
    STORE {
        rs1: u8,
        rs2: u8,
        imm: Immediate<u16>,
    },

    /// rd = *(rs1 + imm) (8-bit)
    LOADB {
        rd: u8,
        rs1: u8,
        imm: Immediate<u16>,
    },

    /// *(rs1 + imm) = rs2 (8-bit)
    STOREB {
        rs2: u8,
        rs1: u8,
        imm: Immediate<u16>,
    },

    /// rd = *(rs1 + imm) (16-bit)
    LOADH {
        rd: u8,
        rs1: u8,
        imm: Immediate<u16>,
    },

    /// *(rs1 + imm) = rs2 (16-bit)
    STOREH {
        rs2: u8,
        rs1: u8,
        imm: Immediate<u16>,
    },

    /// rd = *(PC + imm)
    LOADPC {
        rd: u8,
        imm: Immediate<u16>,
    },

    // ===== Control ===========
    /// PC = PC + imm
    JMP {
        imm: Immediate<u32>,
    },

    JMPR {
        rs: u8,
        imm: Immediate<u16>,
    },
    APC {
        rd: u8,
        imm: Immediate<u16>,
    },

    /// ra = PC + 1; PC = PC + imm
    CALL {
        imm: Immediate<u32>,
    },

    /// return
    RET,

    // ===== Branch ============
    /// if rs1 == rs2
    BEQ {
        rs1: u8,
        rs2: u8,
        imm: Immediate<u16>,
    },

    /// if rs1 != rs2
    BNE {
        rs1: u8,
        rs2: u8,
        imm: Immediate<u16>,
    },

    /// if rs1 < rs2
    BLT {
        rs1: u8,
        rs2: u8,
        imm: Immediate<u16>,
    },

    /// if rs1 > rs2
    BGT {
        rs1: u8,
        rs2: u8,
        imm: Immediate<u16>,
    },

    /// if rs1 <= rs2
    BLE {
        rs1: u8,
        rs2: u8,
        imm: Immediate<u16>,
    },

    /// if rs1 >= rs2
    BGE {
        rs1: u8,
        rs2: u8,
        imm: Immediate<u16>,
    },

    // ===== System ============
    /// (User permissions command)Like call but changes the permissions to Kernel
    SCALL,
    /// (Kernel permissions command)Like return but changes the permissions to User
    SRET,

    /// Read system registers(User permissions)
    /// System Register IDs (imm16):
    /// 0. PSR
    /// 1. IVT
    /// 2. IMR
    /// 3. EPC
    /// 4. TID
    SYSR {
        rd: u8,
        imm: Immediate<u16>,
    },
    /// Read system registers(User permissions)
    /// System Register IDs (imm16):
    /// 0. PSR
    /// 1. IVT
    /// 2. IMR
    /// 3. EPC
    /// 4. TID
    SYSW {
        rs1: u8,
        imm: Immediate<u16>,
    },

    // ===== Atomics ==========
    LR {
        rd: u8,
        rs1: u8,
    },
    SC {
        rd: u8,
        rs1: u8,
        rs2: u8,
    },

    // ===== Misc =============
    /// No-operation
    NOP,
    /// Halt thread- wait until the next interrupt
    HALT,

    LTR {
        rd: u8,
        rs2: u8,
        imm: Immediate<u16>,
    },

    EQR {
        rd: u8,
        rs2: u8,
        imm: Immediate<u16>,
    },

    LTU {
        rd: u8,
        rs2: u8,
        imm: Immediate<u16>,
    },

    EQU {
        rd: u8,
        rs2: u8,
        imm: Immediate<u16>,
    },

    LTS {
        rd: u8,
        rs2: u8,
        imm: Immediate<u16>,
    },

    EQS {
        rd: u8,
        rs2: u8,
        imm: Immediate<u16>,
    },

    SEL {
        rd: u8,
        rs1: u8,
        rs2: u8,
        rs3: u8,
    },

    CTZ {
        rd: u8,
        rs1: u8,
    },

    CLZ {
        rd: u8,
        rs1: u8,
    },

    NOT {
        rd: u8,
        rs1: u8,
    },
}
impl Into<Command> for Instruction {
    fn into(self) -> Command {
        Command::Instr(self)
    }
}
fn resolve_u16(imm: &Immediate<u16>, labels: &HashMap<String, usize>, pc: usize) -> u16 {
    match imm {
        Immediate::Direct(v) => *v,
        Immediate::Label(name) => {
            let target = labels[name];
            (target as isize - pc as isize - 1) as u16
        }
    }
}

fn resolve_u32(imm: &Immediate<u32>, labels: &HashMap<String, usize>, pc: usize) -> u32 {
    match imm {
        Immediate::Direct(v) => *v,
        Immediate::Label(name) => {
            let target = labels[name];
            (target as isize - pc as isize - 1) as u32
        }
    }
}
impl Instruction {
    pub fn encode(&self, labels: &HashMap<&String, usize>, pc: usize) -> Result<u32> {
        let r = |x: &u8| (*x as u32) & 0x1F;
        let imm16 = |x: u16| (x as u16 as u32) & 0xFFFF;
        let imm26 = |x: u32| (x as u32) & 0x03FF_FFFF;

        let u16 = |imm: &Immediate<u16>| -> Result<u16> {
            match imm {
                Immediate::Direct(v) => Ok(*v),
                Immediate::Label(name) => {
                    let target = *labels.get(name).with_context(|| {
                        format!("There wasn't a label with name: '{name}' in the labels hashMap")
                    })?;
                    Ok((target as isize - pc as isize - 1) as u16)
                }
            }
        };

        let u32 = |imm: &Immediate<u32>| -> Result<u32> {
            match imm {
                Immediate::Direct(v) => Ok(*v),
                Immediate::Label(name) => {
                    let target = *labels.get(name).with_context(|| {
                        format!("There wasn't a label with name: '{name}' in the labels hashMap")
                    })?;
                    Ok((target as isize - pc as isize) as u32)
                }
            }
        };

        Ok(match self {
            // ===== R-type =====
            Instruction::ADDR { rd, rs1, rs2 } => {
                (0x00 << 26) | (r(rd) << 21) | (r(rs1) << 16) | (r(rs2) << 11)
            }

            Instruction::SUBR { rd, rs1, rs2 } => {
                (0x01 << 26) | (r(rd) << 21) | (r(rs1) << 16) | (r(rs2) << 11)
            }

            Instruction::ANDR { rd, rs1, rs2 } => {
                (0x02 << 26) | (r(rd) << 21) | (r(rs1) << 16) | (r(rs2) << 11)
            }

            Instruction::ORR { rd, rs1, rs2 } => {
                (0x03 << 26) | (r(rd) << 21) | (r(rs1) << 16) | (r(rs2) << 11)
            }

            Instruction::XORR { rd, rs1, rs2 } => {
                (0x04 << 26) | (r(rd) << 21) | (r(rs1) << 16) | (r(rs2) << 11)
            }

            Instruction::MULR { rd, rs1, rs2 } => {
                (0x05 << 26) | (r(rd) << 21) | (r(rs1) << 16) | (r(rs2) << 11)
            }

            Instruction::DIVR { rd, rs1, rs2 } => {
                (0x06 << 26) | (r(rd) << 21) | (r(rs1) << 16) | (r(rs2) << 11)
            }

            Instruction::REMR { rd, rs1, rs2 } => {
                (0x07 << 26) | (r(rd) << 21) | (r(rs1) << 16) | (r(rs2) << 11)
            }

            Instruction::SHLR { rd, rs1, rs2 } => {
                (0x08 << 26) | (r(rd) << 21) | (r(rs1) << 16) | (r(rs2) << 11)
            }

            Instruction::SHRR { rd, rs1, rs2 } => {
                (0x09 << 26) | (r(rd) << 21) | (r(rs1) << 16) | (r(rs2) << 11)
            }

            Instruction::SARR { rd, rs1, rs2 } => {
                (0x0A << 26) | (r(rd) << 21) | (r(rs1) << 16) | (r(rs2) << 11)
            }

            // ===== I-type =====
            Instruction::ADD { rd, rs1, imm } => {
                (0x0B << 26) | (r(rd) << 21) | (r(rs1) << 16) | imm16(u16(imm)?)
            }

            Instruction::SUB { rd, rs1, imm } => {
                (0x0C << 26) | (r(rd) << 21) | (r(rs1) << 16) | imm16(u16(imm)?)
            }

            Instruction::AND { rd, rs1, imm } => {
                (0x0D << 26) | (r(rd) << 21) | (r(rs1) << 16) | imm16(u16(imm)?)
            }

            Instruction::OR { rd, rs1, imm } => {
                (0x0E << 26) | (r(rd) << 21) | (r(rs1) << 16) | imm16(u16(imm)?)
            }

            Instruction::XOR { rd, rs1, imm } => {
                (0x0F << 26) | (r(rd) << 21) | (r(rs1) << 16) | imm16(u16(imm)?)
            }

            Instruction::MUL { rd, rs1, imm } => {
                (0x10 << 26) | (r(rd) << 21) | (r(rs1) << 16) | imm16(u16(imm)?)
            }

            Instruction::DIV { rd, rs1, imm } => {
                (0x11 << 26) | (r(rd) << 21) | (r(rs1) << 16) | imm16(u16(imm)?)
            }

            Instruction::REM { rd, rs1, imm } => {
                (0x12 << 26) | (r(rd) << 21) | (r(rs1) << 16) | imm16(u16(imm)?)
            }

            Instruction::SHL { rd, rs1, imm } => {
                (0x13 << 26) | (r(rd) << 21) | (r(rs1) << 16) | imm16(u16(imm)?)
            }

            Instruction::SHR { rd, rs1, imm } => {
                (0x14 << 26) | (r(rd) << 21) | (r(rs1) << 16) | imm16(u16(imm)?)
            }

            Instruction::SAR { rd, rs1, imm } => {
                (0x15 << 26) | (r(rd) << 21) | (r(rs1) << 16) | imm16(u16(imm)?)
            }

            Instruction::LUI { rd, imm } => (0x16 << 26) | (r(rd) << 21) | (imm16(u16(imm)?) << 5),

            // ===== Memory =====
            Instruction::LOAD { rd, rs1, imm } => {
                (0x17 << 26) | (r(rd) << 21) | (r(rs1) << 16) | imm16(u16(imm)?)
            }

            Instruction::STORE { rs2, rs1, imm } => {
                (0x18 << 26) | (r(rs2) << 21) | (r(rs1) << 16) | imm16(u16(imm)?)
            }

            Instruction::LOADB { rd, rs1, imm } => {
                (0x19 << 26) | (r(rd) << 21) | (r(rs1) << 16) | imm16(u16(imm)?)
            }

            Instruction::STOREB { rs2, rs1, imm } => {
                (0x1A << 26) | (r(rs2) << 21) | (r(rs1) << 16) | imm16(u16(imm)?)
            }

            Instruction::LOADH { rd, rs1, imm } => {
                (0x1B << 26) | (r(rd) << 21) | (r(rs1) << 16) | imm16(u16(imm)?)
            }

            Instruction::STOREH { rs2, rs1, imm } => {
                (0x1C << 26) | (r(rs2) << 21) | (r(rs1) << 16) | imm16(u16(imm)?)
            }

            Instruction::LOADPC { rd, imm } => {
                (0x1D << 26) | (r(rd) << 21) | (imm16(u16(imm)?) << 5)
            }

            // ===== Control =====
            Instruction::JMP { imm } => (0x1E << 26) | imm26(u32(imm)?),

            Instruction::CALL { imm } => (0x1F << 26) | imm26(u32(imm)?),

            Instruction::RET => 0x20 << 26,

            Instruction::JMPR { rs, imm } => (0x21 << 26) | (r(rs) << 21) | (imm16(u16(imm)?) << 5),

            Instruction::APC { rd, imm } => (0x22 << 26) | (r(rd) << 21) | (imm16(u16(imm)?) << 5),

            // ===== Branch =====
            Instruction::BEQ { rs1, rs2, imm } => {
                (0x23 << 26) | (r(rs1) << 21) | (r(rs2) << 16) | imm16(u16(imm)?)
            }

            Instruction::BNE { rs1, rs2, imm } => {
                (0x24 << 26) | (r(rs1) << 21) | (r(rs2) << 16) | imm16(u16(imm)?)
            }

            Instruction::BLT { rs1, rs2, imm } => {
                (0x25 << 26) | (r(rs1) << 21) | (r(rs2) << 16) | imm16(u16(imm)?)
            }

            Instruction::BGT { rs1, rs2, imm } => {
                (0x26 << 26) | (r(rs1) << 21) | (r(rs2) << 16) | imm16(u16(imm)?)
            }

            Instruction::BLE { rs1, rs2, imm } => {
                (0x27 << 26) | (r(rs1) << 21) | (r(rs2) << 16) | imm16(u16(imm)?)
            }

            Instruction::BGE { rs1, rs2, imm } => {
                (0x28 << 26) | (r(rs1) << 21) | (r(rs2) << 16) | imm16(u16(imm)?)
            }

            // ===== System =====
            Instruction::SCALL => 0x29 << 26,
            Instruction::SRET => 0x2A << 26,

            Instruction::SYSR { rd, imm } => (0x2B << 26) | (r(rd) << 21) | (imm16(u16(imm)?) << 5),

            Instruction::SYSW { rs1, imm } => {
                (0x2C << 26) | (r(rs1) << 21) | (imm16(u16(imm)?) << 5)
            }

            // ===== Atomics =====
            Instruction::LR { rd, rs1 } => (0x2D << 26) | (r(rd) << 21) | (r(rs1) << 16),

            Instruction::SC { rd, rs1, rs2 } => {
                (0x2E << 26) | (r(rd) << 21) | (r(rs1) << 16) | (r(rs2) << 11)
            }

            // ===== Misc =====
            Instruction::NOP => 0x2F << 26,
            Instruction::HALT => 0x30 << 26,

            // ===== Comparison =====
            Instruction::LTR { rd, rs2, imm } => {
                (0x31 << 26) | (r(rd) << 21) | (r(rs2) << 16) | imm16(u16(imm)?)
            }

            Instruction::EQR { rd, rs2, imm } => {
                (0x32 << 26) | (r(rd) << 21) | (r(rs2) << 16) | imm16(u16(imm)?)
            }

            Instruction::LTU { rd, rs2, imm } => {
                (0x33 << 26) | (r(rd) << 21) | (r(rs2) << 16) | imm16(u16(imm)?)
            }

            Instruction::EQU { rd, rs2, imm } => {
                (0x34 << 26) | (r(rd) << 21) | (r(rs2) << 16) | imm16(u16(imm)?)
            }

            Instruction::LTS { rd, rs2, imm } => {
                (0x35 << 26) | (r(rd) << 21) | (r(rs2) << 16) | imm16(u16(imm)?)
            }

            Instruction::EQS { rd, rs2, imm } => {
                (0x36 << 26) | (r(rd) << 21) | (r(rs2) << 16) | imm16(u16(imm)?)
            }

            // ===== Ternary (3-source) =====
            // layout: [ opcode | rd | rs1 | rs2 | rs3 | unused ]
            Instruction::SEL { rd, rs1, rs2, rs3 } => {
                (0x37 << 26) | (r(rd) << 21) | (r(rs1) << 16) | (r(rs2) << 11) | (r(rs3) << 6)
            }

            // ===== Unary =====
            Instruction::CTZ { rd, rs1 } => (0x38 << 26) | (r(rd) << 21) | (r(rs1) << 16),

            Instruction::CLZ { rd, rs1 } => (0x39 << 26) | (r(rd) << 21) | (r(rs1) << 16),
            Instruction::NOT { rd, rs1 } => (0x3A << 26) | (r(rd) << 21) | (r(rs1) << 16),
        })
    }
}
