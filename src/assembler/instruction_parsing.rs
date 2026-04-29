use crate::assembler::instruction::{Command, Immediate, Instruction, Macro};
use anyhow::{Context, Result};

pub fn parse_program(input: &str) -> Result<Vec<Command>> {
    let mut out = Vec::new();

    for (lineno, raw) in input.lines().enumerate() {
        let line = strip_comments(raw).trim();
        if line.is_empty() {
            continue;
        }

        // label
        if let Some(label) = line.strip_prefix(':') {
            out.push(Command::Label(label.trim().to_string()));
            continue;
        }

        // raw data
        if let Some(rest) = line.strip_prefix(".data") {
            let values = rest
                .split_whitespace()
                .map(|x| x.parse::<i32>())
                .collect::<Result<Vec<_>, _>>()
                .with_context(|| format!("line {}: invalid .data", lineno))?;

            out.push(Command::RawData(values));
            continue;
        }

        out.push(parse_op(line).with_context(|| format!("line {}, contents:'{raw}'", lineno))?);
    }

    Ok(out)
}

fn strip_comments(s: &str) -> &str {
    s.split('#').next().unwrap_or("")
}

fn parse_reg(s: &str) -> Result<u8> {
    let s = s.trim().strip_prefix('r').unwrap_or(s);

    let v: u8 = s
        .parse()
        .with_context(|| format!("invalid register '{}'", s))?;

    if v > 31 {
        anyhow::bail!("register out of range: {}", v);
    }

    Ok(v)
}

fn parse_int(s: &str) -> Result<i64> {
    if let Some(hex) = s.strip_prefix("0x") {
        Ok(i64::from_str_radix(hex, 16)?)
    } else if let Some(bin) = s.strip_prefix("0b") {
        Ok(i64::from_str_radix(bin, 2)?)
    } else {
        Ok(s.parse::<i64>()?)
    }
}

fn parse_imm16(s: Option<&&str>) -> Result<Immediate<i16>> {
    let s = *s.context("missing imm16")?;

    match parse_int(s) {
        Ok(v) => {
            if v >= i16::MIN as i64 && v <= i16::MAX as i64 {
                return Ok(Immediate::Direct(v as i16 as i16));
            }

            anyhow::bail!("imm16 out of range: {}", v);
        }
        Err(_) => Ok(Immediate::Label(s.to_string())),
    }
}

fn parse_imm32(s: Option<&&str>) -> Result<Immediate<i32>> {
    let s = *s.context("missing imm32")?;

    match parse_int(s) {
        Ok(v) => {
            let v = v as u32 as i32; // interpret as 32-bit signed two's complement

            return Ok(Immediate::Direct(v));
        }
        Err(_) => Ok(Immediate::Label(s.to_string())),
    }
}
fn parse_imm26(s: Option<&&str>) -> Result<Immediate<i32>> {
    let s = *s.context("missing imm26")?;

    match parse_int(s) {
        Ok(v) => {
            const MIN: i64 = -(1 << 25);
            const MAX: i64 = (1 << 25) - 1;

            if v >= MIN && v <= MAX {
                return Ok(Immediate::Direct(v as i32 as i32));
            }

            anyhow::bail!("imm26 out of range: {}", v);
        }
        Err(_) => Ok(Immediate::Label(s.to_string())),
    }
}

fn rrr<F>(args: &[&str], f: F) -> Result<Command>
where
    F: Fn(u8, u8, u8) -> Command,
{
    if args.len() != 3 {
        anyhow::bail!("expected: rd rs1 rs2");
    }

    Ok(f(
        parse_reg(args[0])?,
        parse_reg(args[1])?,
        parse_reg(args[2])?,
    ))
}

fn rri<F>(args: &[&str], f: F) -> Result<Command>
where
    F: Fn(u8, u8, Immediate<i16>) -> Command,
{
    if args.len() != 3 {
        anyhow::bail!("expected: rd rs1 imm");
    }

    Ok(f(
        parse_reg(args[0])?,
        parse_reg(args[1])?,
        parse_imm16(args.get(2))?,
    ))
}

fn ri<F>(args: &[&str], f: F) -> Result<Command>
where
    F: Fn(u8, Immediate<i16>) -> Command,
{
    if args.len() != 2 {
        anyhow::bail!("expected: rd imm");
    }

    Ok(f(parse_reg(args[0])?, parse_imm16(args.get(1))?))
}

fn parse_op(line: &str) -> Result<Command> {
    let mut parts = line.split_whitespace();
    let op = parts.next().context("empty line")?.to_uppercase();
    let args: Vec<&str> = parts.collect();

    match op.as_str() {
        // macros
        "SET32" => {
            let (rd, imm) = parse_rd_imm32(&args)?;
            Ok(Macro::Set32 { rd, imm }.into())
        }
        "GETBIT" => rri(&args, |rd, rs1, imm| Macro::GetBit { rd, rs1, imm }.into()),

        // zero
        "RET" => Ok(Instruction::RET.into()),
        "NOP" => Ok(Instruction::NOP.into()),
        "HALT" => Ok(Instruction::HALT.into()),
        "SCALL" => Ok(Instruction::SCALL.into()),
        "SRET" => Ok(Instruction::SRET.into()),

        // rrr
        "ADDR" => rrr(&args, |rd, rs1, rs2| {
            Instruction::ADDR { rd, rs1, rs2 }.into()
        }),
        "SUBR" => rrr(&args, |rd, rs1, rs2| {
            Instruction::SUBR { rd, rs1, rs2 }.into()
        }),
        "ANDR" => rrr(&args, |rd, rs1, rs2| {
            Instruction::ANDR { rd, rs1, rs2 }.into()
        }),
        "ORR" => rrr(&args, |rd, rs1, rs2| {
            Instruction::ORR { rd, rs1, rs2 }.into()
        }),
        "XORR" => rrr(&args, |rd, rs1, rs2| {
            Instruction::XORR { rd, rs1, rs2 }.into()
        }),
        "MULR" => rrr(&args, |rd, rs1, rs2| {
            Instruction::MULR { rd, rs1, rs2 }.into()
        }),
        "DIVR" => rrr(&args, |rd, rs1, rs2| {
            Instruction::DIVR { rd, rs1, rs2 }.into()
        }),
        "REMR" => rrr(&args, |rd, rs1, rs2| {
            Instruction::REMR { rd, rs1, rs2 }.into()
        }),
        "SHLR" => rrr(&args, |rd, rs1, rs2| {
            Instruction::SHLR { rd, rs1, rs2 }.into()
        }),
        "SHRR" => rrr(&args, |rd, rs1, rs2| {
            Instruction::SHRR { rd, rs1, rs2 }.into()
        }),
        "SARR" => rrr(&args, |rd, rs1, rs2| {
            Instruction::SARR { rd, rs1, rs2 }.into()
        }),

        // rri
        "ADD" => rri(&args, |rd, rs1, imm| {
            Instruction::ADD { rd, rs1, imm }.into()
        }),
        "SUB" => rri(&args, |rd, rs1, imm| {
            Instruction::SUB { rd, rs1, imm }.into()
        }),
        "AND" => rri(&args, |rd, rs1, imm| {
            Instruction::AND { rd, rs1, imm }.into()
        }),
        "OR" => rri(&args, |rd, rs1, imm| {
            Instruction::OR { rd, rs1, imm }.into()
        }),
        "XOR" => rri(&args, |rd, rs1, imm| {
            Instruction::XOR { rd, rs1, imm }.into()
        }),
        "MUL" => rri(&args, |rd, rs1, imm| {
            Instruction::MUL { rd, rs1, imm }.into()
        }),
        "DIV" => rri(&args, |rd, rs1, imm| {
            Instruction::DIV { rd, rs1, imm }.into()
        }),
        "REM" => rri(&args, |rd, rs1, imm| {
            Instruction::REM { rd, rs1, imm }.into()
        }),
        "SHL" => rri(&args, |rd, rs1, imm| {
            Instruction::SHL { rd, rs1, imm }.into()
        }),
        "SHR" => rri(&args, |rd, rs1, imm| {
            Instruction::SHR { rd, rs1, imm }.into()
        }),
        "SAR" => rri(&args, |rd, rs1, imm| {
            Instruction::SAR { rd, rs1, imm }.into()
        }),

        // ri
        "LUI" => ri(&args, |rd, imm| Instruction::LUI { rd, imm }.into()),
        "LOADPC" => ri(&args, |rd, imm| Instruction::LOADPC { rd, imm }.into()),
        "APC" => ri(&args, |rd, imm| Instruction::APC { rd, imm }.into()),

        // memory
        "LOAD" => rri(&args, |rd, rs1, imm| {
            Instruction::LOAD { rd, rs1, imm }.into()
        }),
        "LOADB" => rri(&args, |rd, rs1, imm| {
            Instruction::LOADB { rd, rs1, imm }.into()
        }),
        "LOADH" => rri(&args, |rd, rs1, imm| {
            Instruction::LOADH { rd, rs1, imm }.into()
        }),
        "STORE" => rri(&args, |rs1, rs2, imm| {
            Instruction::STORE { rs1, rs2, imm }.into()
        }),
        "STOREB" => rri(&args, |rs1, rs2, imm| {
            Instruction::STOREB { rs1, rs2, imm }.into()
        }),

        "STOREH" => rri(&args, |rs1, rs2, imm| {
            Instruction::STOREH { rs1, rs2, imm }.into()
        }),
        // control
        "JMP" => Ok(Instruction::JMP {
            imm: parse_imm26(args.get(0))?,
        }
        .into()),
        "CALL" => Ok(Instruction::CALL {
            imm: parse_imm26(args.get(0))?,
        }
        .into()),

        "JMPR" => ri(&args, |rs1, imm| Instruction::JMPR { rs1, imm }.into()),
        // branch
        "BEQ" => rri(&args, |rs1, rs2, imm| {
            Instruction::BEQ { rs1, rs2, imm }.into()
        }),

        "BNE" => rri(&args, |rs1, rs2, imm| {
            Instruction::BNE { rs1, rs2, imm }.into()
        }),

        "BLT" => rri(&args, |rs1, rs2, imm| {
            Instruction::BLT { rs1, rs2, imm }.into()
        }),

        "BGT" => rri(&args, |rs1, rs2, imm| {
            Instruction::BGT { rs1, rs2, imm }.into()
        }),

        "BLE" => rri(&args, |rs1, rs2, imm| {
            Instruction::BLE { rs1, rs2, imm }.into()
        }),

        "BGE" => rri(&args, |rs1, rs2, imm| {
            Instruction::BGE { rs1, rs2, imm }.into()
        }),

        // sys
        "SYSR" => ri(&args, |rd, imm| Instruction::SYSR { rd, imm }.into()),

        "SYSW" => ri(&args, |rs1, imm| Instruction::SYSW { rs1, imm }.into()),
        // atomics
        "LR" => {
            let (rd, rs1) = (parse_reg(args[0])?, parse_reg(args[1])?);
            Ok(Instruction::LR { rd, rs1 }.into())
        }
        "SC" => rrr(&args, |a, b, c| {
            Instruction::SC {
                rd: a,
                rs1: b,
                rs2: c,
            }
            .into()
        }),

        // misc
        "SEL" => {
            let (a, b, c, d) = (
                parse_reg(args[0])?,
                parse_reg(args[1])?,
                parse_reg(args[2])?,
                parse_reg(args[3])?,
            );
            Ok(Instruction::SEL {
                rd: a,
                rs1: b,
                rs2: c,
                rs3: d,
            }
            .into())
        }

        "CTZ" => {
            let (rd, rs1) = (parse_reg(args[0])?, parse_reg(args[1])?);
            Ok(Instruction::CTZ { rd, rs1 }.into())
        }

        "CLZ" => {
            let (rd, rs1) = (parse_reg(args[0])?, parse_reg(args[1])?);
            Ok(Instruction::CLZ { rd, rs1 }.into())
        }

        "NOT" => {
            let (rd, rs1) = (parse_reg(args[0])?, parse_reg(args[1])?);
            Ok(Instruction::NOT { rd, rs1 }.into())
        }

        // compare
        "LTR" => rrr(&args, |rd, rs1, rs2| {
            Instruction::LTR { rd, rs1, rs2 }.into()
        }),

        "EQR" => rrr(&args, |rd, rs1, rs2| {
            Instruction::EQR { rd, rs1, rs2 }.into()
        }),
        "LT" => rri(&args, |rd, rs1, imm| {
            Instruction::LT { rd, rs1, imm }.into()
        }),
        "EQ" => rri(&args, |rd, rs1, imm| {
            Instruction::EQ { rd, rs1, imm }.into()
        }),

        _ => anyhow::bail!("unknown opcode: {}", op),
    }
}

fn parse_rd_imm16(args: &[&str]) -> Result<(u8, Immediate<i16>)> {
    if args.len() != 2 {
        anyhow::bail!("expected: rd imm");
    }
    Ok((parse_reg(args[0])?, parse_imm16(args.get(1))?))
}

fn parse_rd_imm32(args: &[&str]) -> Result<(u8, Immediate<i32>)> {
    if args.len() != 2 {
        anyhow::bail!("expected: rd imm");
    }
    Ok((parse_reg(args[0])?, parse_imm32(args.get(1))?))
}
