use anyhow::{Result, bail};
use bitflags::bitflags;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
    SymLink = 0b001,
    Directory = 0b010,
    File = 0b100,
}

impl TryFrom<u8> for EntryType {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            0b001 => Ok(Self::SymLink),
            0b010 => Ok(Self::Directory),
            0b100 => Ok(Self::File),
            _ => bail!("invalid entry type: {value:#05b}"),
        }
    }
}

impl From<EntryType> for u8 {
    fn from(value: EntryType) -> Self {
        value as u8
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Permissions: u8 {
        const EXECUTE = 0b001;
        const WRITE   = 0b010;
        const READ    = 0b100;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mode {
    pub entry_type: EntryType,
    pub user: Permissions,
    pub group: Permissions,
    pub other: Permissions,
}

impl From<Mode> for u16 {
    fn from(mode: Mode) -> Self {
        ((u8::from(mode.entry_type) as u16) << 13)
            | ((mode.user.bits() as u16) << 6)
            | ((mode.group.bits() as u16) << 3)
            | (mode.other.bits() as u16)
    }
}

impl TryFrom<u16> for Mode {
    type Error = anyhow::Error;

    fn try_from(value: u16) -> Result<Self> {
        let entry_bits = ((value >> 13) & 0b111) as u8;
        let user_bits = ((value >> 6) & 0b111) as u8;
        let group_bits = ((value >> 3) & 0b111) as u8;
        let other_bits = (value & 0b111) as u8;

        Ok(Self {
            entry_type: entry_bits.try_into()?,
            user: Permissions::from_bits_retain(user_bits),
            group: Permissions::from_bits_retain(group_bits),
            other: Permissions::from_bits_retain(other_bits),
        })
    }
}
