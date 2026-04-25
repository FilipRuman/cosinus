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

        out.push(parse_op(line).with_context(|| format!("line {}", lineno))?);
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

/* imm16: signed-first, fallback unsigned */
fn parse_imm16(s: Option<&&str>) -> Result<Immediate<u16>> {
    let s = *s.context("missing imm16")?;

    match parse_int(s) {
        Ok(v) => {
            if v >= i16::MIN as i64 && v <= i16::MAX as i64 {
                return Ok(Immediate::Direct(v as i16 as u16));
            }

            if v >= 0 && v <= u16::MAX as i64 {
                return Ok(Immediate::Direct(v as u16));
            }

            anyhow::bail!("imm16 out of range: {}", v);
        }
        Err(_) => Ok(Immediate::Label(s.to_string())),
    }
}

/* imm26: signed-first, fallback unsigned */
fn parse_imm26(s: Option<&&str>) -> Result<Immediate<u32>> {
    let s = *s.context("missing imm26")?;

    match parse_int(s) {
        Ok(v) => {
            const MIN: i64 = -(1 << 25);
            const MAX: i64 = (1 << 25) - 1;

            if v >= MIN && v <= MAX {
                return Ok(Immediate::Direct(v as i32 as u32));
            }

            if v >= 0 && v <= ((1 << 26) - 1) {
                return Ok(Immediate::Direct(v as u32));
            }

            anyhow::bail!("imm26 out of range: {}", v);
        }
        Err(_) => Ok(Immediate::Label(s.to_string())),
    }
}

fn rrr<F>(args: &[&str], f: F) -> Result<Command>
where
    F: Fn(u8, u8, u8) -> Instruction,
{
    if args.len() != 3 {
        anyhow::bail!("expected: rd rs1 rs2");
    }

    Ok(f(
        parse_reg(args[0])?,
        parse_reg(args[1])?,
        parse_reg(args[2])?,
    )
    .into())
}

fn rri<F>(args: &[&str], f: F) -> Result<Command>
where
    F: Fn(u8, u8, Immediate<u16>) -> Instruction,
{
    if args.len() != 3 {
        anyhow::bail!("expected: rd rs1 imm");
    }

    Ok(f(
        parse_reg(args[0])?,
        parse_reg(args[1])?,
        parse_imm16(args.get(2))?,
    )
    .into())
}

fn ri<F>(args: &[&str], f: F) -> Result<Command>
where
    F: Fn(u8, Immediate<u16>) -> Instruction,
{
    if args.len() != 2 {
        anyhow::bail!("expected: rd imm");
    }

    Ok(f(parse_reg(args[0])?, parse_imm16(args.get(1))?).into())
}

fn branch<F>(args: &[&str], f: F) -> Result<Command>
where
    F: Fn(u8, u8, Immediate<u16>) -> Instruction,
{
    if args.len() != 3 {
        anyhow::bail!("expected: rs1 rs2 imm");
    }

    Ok(f(
        parse_reg(args[0])?,
        parse_reg(args[1])?,
        parse_imm16(args.get(2))?,
    )
    .into())
}

fn cmp<F>(args: &[&str], f: F) -> Result<Command>
where
    F: Fn(u8, u8, Immediate<u16>) -> Instruction,
{
    if args.len() != 3 {
        anyhow::bail!("expected: rd rs2 imm");
    }

    Ok(f(
        parse_reg(args[0])?,
        parse_reg(args[1])?,
        parse_imm16(args.get(2))?,
    )
    .into())
}

fn parse_store(args: &[&str]) -> Result<(u8, u8, Immediate<u16>)> {
    if args.len() != 3 {
        anyhow::bail!("expected: rs1 rs2 imm");
    }

    Ok((
        parse_reg(args[0])?,
        parse_reg(args[1])?,
        parse_imm16(args.get(2))?,
    ))
}

fn parse_storeb(args: &[&str]) -> Result<(u8, u8, Immediate<u16>)> {
    if args.len() != 3 {
        anyhow::bail!("expected: rs1 rs2 imm");
    }

    Ok((
        parse_reg(args[1])?, // rs2
        parse_reg(args[0])?, // rs1 (swapped per ISA)
        parse_imm16(args.get(2))?,
    ))
}

fn parse_op(line: &str) -> Result<Command> {
    let mut parts = line.split_whitespace();
    let op = parts.next().context("empty line")?.to_uppercase();
    let args: Vec<&str> = parts.collect();

    Ok(match op.as_str() {
        // macros
        "SET" => {
            let (rd, imm) = parse_rd_imm16(&args)?;
            Macro::Set { rd, imm }.into()
        }
        "SET32" => {
            let (rd, imm) = parse_rd_imm32(&args)?;
            Macro::Set32 { rd, imm }.into()
        }

        // zero
        "RET" => Instruction::RET.into(),
        "NOP" => Instruction::NOP.into(),
        "HALT" => Instruction::HALT.into(),
        "SCALL" => Instruction::SCALL.into(),
        "SRET" => Instruction::SRET.into(),

        // rrr
        "ADDR" => rrr(&args, |rd, rs1, rs2| Instruction::ADDR { rd, rs1, rs2 })?,
        "SUBR" => rrr(&args, |rd, rs1, rs2| Instruction::SUBR { rd, rs1, rs2 })?,
        "ANDR" => rrr(&args, |rd, rs1, rs2| Instruction::ANDR { rd, rs1, rs2 })?,
        "ORR" => rrr(&args, |rd, rs1, rs2| Instruction::ORR { rd, rs1, rs2 })?,
        "XORR" => rrr(&args, |rd, rs1, rs2| Instruction::XORR { rd, rs1, rs2 })?,
        "MULR" => rrr(&args, |rd, rs1, rs2| Instruction::MULR { rd, rs1, rs2 })?,
        "DIVR" => rrr(&args, |rd, rs1, rs2| Instruction::DIVR { rd, rs1, rs2 })?,
        "REMR" => rrr(&args, |rd, rs1, rs2| Instruction::REMR { rd, rs1, rs2 })?,
        "SHLR" => rrr(&args, |rd, rs1, rs2| Instruction::SHLR { rd, rs1, rs2 })?,
        "SHRR" => rrr(&args, |rd, rs1, rs2| Instruction::SHRR { rd, rs1, rs2 })?,
        "SARR" => rrr(&args, |rd, rs1, rs2| Instruction::SARR { rd, rs1, rs2 })?,

        // rri
        "ADD" => rri(&args, |rd, rs1, imm| Instruction::ADD { rd, rs1, imm })?,
        "SUB" => rri(&args, |rd, rs1, imm| Instruction::SUB { rd, rs1, imm })?,
        "AND" => rri(&args, |rd, rs1, imm| Instruction::AND { rd, rs1, imm })?,
        "OR" => rri(&args, |rd, rs1, imm| Instruction::OR { rd, rs1, imm })?,
        "XOR" => rri(&args, |rd, rs1, imm| Instruction::XOR { rd, rs1, imm })?,
        "MUL" => rri(&args, |rd, rs1, imm| Instruction::MUL { rd, rs1, imm })?,
        "DIV" => rri(&args, |rd, rs1, imm| Instruction::DIV { rd, rs1, imm })?,
        "REM" => rri(&args, |rd, rs1, imm| Instruction::REM { rd, rs1, imm })?,
        "SHL" => rri(&args, |rd, rs1, imm| Instruction::SHL { rd, rs1, imm })?,
        "SHR" => rri(&args, |rd, rs1, imm| Instruction::SHR { rd, rs1, imm })?,
        "SAR" => rri(&args, |rd, rs1, imm| Instruction::SAR { rd, rs1, imm })?,

        // ri
        "LUI" => ri(&args, |rd, imm| Instruction::LUI { rd, imm })?,
        "LOADPC" => ri(&args, |rd, imm| Instruction::LOADPC { rd, imm })?,
        "APC" => ri(&args, |rd, imm| Instruction::APC { rd, imm })?,

        // memory
        "LOAD" => rri(&args, |rd, rs1, imm| Instruction::LOAD { rd, rs1, imm })?,
        "LOADB" => rri(&args, |rd, rs1, imm| Instruction::LOADB { rd, rs1, imm })?,
        "LOADH" => rri(&args, |rd, rs1, imm| Instruction::LOADH { rd, rs1, imm })?,

        "STORE" => {
            let (rs1, rs2, imm) = parse_store(&args)?;
            Instruction::STORE { rs1, rs2, imm }.into()
        }
        "STOREB" => {
            let (rs2, rs1, imm) = parse_storeb(&args)?;
            Instruction::STOREB { rs2, rs1, imm }.into()
        }
        "STOREH" => {
            let (rs2, rs1, imm) = parse_storeb(&args)?;
            Instruction::STOREH { rs2, rs1, imm }.into()
        }

        // control
        "JMP" => Instruction::JMP {
            imm: parse_imm26(args.get(0))?,
        }
        .into(),
        "CALL" => Instruction::CALL {
            imm: parse_imm26(args.get(0))?,
        }
        .into(),

        "JMPR" => {
            if args.len() != 2 {
                anyhow::bail!("expected: rs imm");
            }
            Instruction::JMPR {
                rs: parse_reg(args[0])?,
                imm: parse_imm16(args.get(1))?,
            }
            .into()
        }

        // branch
        "BEQ" => branch(&args, |a, b, i| Instruction::BEQ {
            rs1: a,
            rs2: b,
            imm: i,
        })?,
        "BNE" => branch(&args, |a, b, i| Instruction::BNE {
            rs1: a,
            rs2: b,
            imm: i,
        })?,
        "BLT" => branch(&args, |a, b, i| Instruction::BLT {
            rs1: a,
            rs2: b,
            imm: i,
        })?,
        "BGT" => branch(&args, |a, b, i| Instruction::BGT {
            rs1: a,
            rs2: b,
            imm: i,
        })?,
        "BLE" => branch(&args, |a, b, i| Instruction::BLE {
            rs1: a,
            rs2: b,
            imm: i,
        })?,
        "BGE" => branch(&args, |a, b, i| Instruction::BGE {
            rs1: a,
            rs2: b,
            imm: i,
        })?,

        // sys
        "SYSR" => {
            if args.len() != 2 {
                anyhow::bail!("expected: rd imm");
            }
            Instruction::SYSR {
                rd: parse_reg(args[0])?,
                imm: parse_imm16(args.get(1))?,
            }
            .into()
        }

        "SYSW" => {
            if args.len() != 2 {
                anyhow::bail!("expected: rs1 imm");
            }
            Instruction::SYSW {
                rs1: parse_reg(args[0])?,
                imm: parse_imm16(args.get(1))?,
            }
            .into()
        }

        // atomics
        "LR" => {
            let (rd, rs1) = (parse_reg(args[0])?, parse_reg(args[1])?);
            Instruction::LR { rd, rs1 }.into()
        }
        "SC" => {
            let command = rrr(&args, |a, b, c| Instruction::SC {
                rd: a,
                rs1: b,
                rs2: c,
            })?;
            command.into()
        }

        // misc
        "SEL" => {
            let (a, b, c, d) = (
                parse_reg(args[0])?,
                parse_reg(args[1])?,
                parse_reg(args[2])?,
                parse_reg(args[3])?,
            );
            Instruction::SEL {
                rd: a,
                rs1: b,
                rs2: c,
                rs3: d,
            }
            .into()
        }

        "CTZ" => {
            let (rd, rs1) = (parse_reg(args[0])?, parse_reg(args[1])?);
            Instruction::CTZ { rd, rs1 }.into()
        }

        "CLZ" => {
            let (rd, rs1) = (parse_reg(args[0])?, parse_reg(args[1])?);
            Instruction::CLZ { rd, rs1 }.into()
        }

        "NOT" => {
            let (rd, rs1) = (parse_reg(args[0])?, parse_reg(args[1])?);
            Instruction::NOT { rd, rs1 }.into()
        }

        // compare
        "LTR" => cmp(&args, |a, b, i| Instruction::LTR {
            rd: a,
            rs2: b,
            imm: i,
        })?,
        "EQR" => cmp(&args, |a, b, i| Instruction::EQR {
            rd: a,
            rs2: b,
            imm: i,
        })?,
        "LTU" => cmp(&args, |a, b, i| Instruction::LTU {
            rd: a,
            rs2: b,
            imm: i,
        })?,
        "EQU" => cmp(&args, |a, b, i| Instruction::EQU {
            rd: a,
            rs2: b,
            imm: i,
        })?,
        "LTS" => cmp(&args, |a, b, i| Instruction::LTS {
            rd: a,
            rs2: b,
            imm: i,
        })?,
        "EQS" => cmp(&args, |a, b, i| Instruction::EQS {
            rd: a,
            rs2: b,
            imm: i,
        })?,

        _ => anyhow::bail!("unknown opcode: {}", op),
    })
}

/* missing small helpers */

fn parse_rd_imm16(args: &[&str]) -> Result<(u8, Immediate<u16>)> {
    if args.len() != 2 {
        anyhow::bail!("expected: rd imm");
    }
    Ok((parse_reg(args[0])?, parse_imm16(args.get(1))?))
}

fn parse_rd_imm32(args: &[&str]) -> Result<(u8, Immediate<u32>)> {
    if args.len() != 2 {
        anyhow::bail!("expected: rd imm");
    }
    Ok((parse_reg(args[0])?, parse_imm26(args.get(1))?))
}
