use bytemuck::{Pod, Zeroable};

use crate::emulator::disk::*;

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Debug, PartialEq, Copy)]
pub struct INode {
    pub mode: u16,  // Entry type + permissions
    pub links: u16, // Link count - how many other nodes reference this node

    pub uid: u32,
    pub gid: u32,

    pub content_size: u32, // Bytes

    pub flags: u32,
    pub _pad: u32,

    pub extents: [Extent; 12], // [(high 32 bits: u32 base block address, low 32 bits: block count);12]
}
impl INode {
    pub fn blocks(&self) -> Vec<u32> {
        let mut output = vec![];
        self.extents
            .iter()
            .for_each(|extent| output.append(&mut extent.blocks()));
        output
    }
    pub fn allocated_bytes(&self) -> u64 {
        let mut sum = 0;
        self.extents
            .iter()
            .for_each(|extent| sum += extent.block_count);
        sum as u64 * BLOCK_SIZE_BYTES as u64
    }

    pub fn from_bytes_le(bytes: &[u8]) -> &Self {
        bytemuck::from_bytes(bytes)
    }
    pub fn to_bytes_le(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}
