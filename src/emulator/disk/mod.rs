pub mod entry_traversal;
pub mod test;

use anyhow::{Context, Ok, Result, bail};
use bytemuck::{Pod, Zeroable, bytes_of};
use log::{debug, info};
use std::{
    fs::{File, OpenOptions},
    io::{Seek, Write},
    os::unix::fs::FileExt,
    sync::LazyLock,
};

use crate::dir_handling::project_dir;

pub(crate) const INTERNAL_FILE_PATH: &'static str = "./disk/disk.fr2";
pub static DISK: LazyLock<Disk> = LazyLock::new(|| Disk::new());
unsafe impl Sync for Disk {}
#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy, Debug, PartialEq)]
pub struct SuperBlock {
    pub total_blocks: u32,
    pub total_inodes: u32,
    /// NEEDS TO BE DIVIDABLE BY 8
    pub blocks_per_group: u32,
    /// NEEDS TO BE DIVIDABLE BY 8
    pub inodes_per_group: u32,
    pub group_count: u32,
    pub inode_size: u32,
    pub root_inode: u32,
    pub first_data_block: u32,
    pub flags: u32,
}
impl SuperBlock {
    pub fn parse_from_internal_file(file: &File) -> Result<Self> {
        let magic = {
            let mut buffer = [0u8; 4];
            file.read_exact_at(&mut buffer, 0)?;
            u32::from_le_bytes(buffer)
        };
        if magic != 0x00325246 {
            bail!(
                "Magic number is invalid for fr2 file system!:'{magic:#x}' expected:'0x00325246'"
            );
        }

        let mut buffer = [0u8; size_of::<SuperBlock>()];
        file.read_exact_at(&mut buffer, 4)?; // 4-> skip magic number

        Ok(*bytemuck::from_bytes(&buffer))
    }
}
pub fn set_bit_in_block(block_bit_idx: u32, block: &mut Vec<u8>) {
    let byte_idx = block_bit_idx / 8;
    let bit = block_bit_idx % 8;
    let mask = 1 << bit;
    block[byte_idx as usize] |= mask;
}

pub struct FsInfo {
    pub group_descriptor_table_blocks: u32,
    pub inode_table_blocks: u32,
    pub total_group_size_blocks: u32,
    pub total_fs_size: u32,
}
impl FsInfo {
    pub fn get_start_block_for_group(&self, group_idx: u32) -> u32 {
        1 + self.group_descriptor_table_blocks + self.total_group_size_blocks * group_idx
    }

    pub fn new(superblock: &SuperBlock) -> Self {
        let group_descriptor_table_blocks = (size_of::<GroupDescriptor>() as u32
            * superblock.group_count)
            .div_ceil(BLOCK_SIZE_BYTES);

        let inode_table_blocks =
            (size_of::<INode>() as u32 * superblock.inodes_per_group).div_ceil(BLOCK_SIZE_BYTES);
        let total_group_size_blocks = superblock.blocks_per_group
        + 1/* Block Bitmap */ + 1 /*inode Bitmap*/ + inode_table_blocks;
        let total_fs_size = 1 /* super block */ + group_descriptor_table_blocks +superblock.group_count * total_group_size_blocks;
        Self {
            group_descriptor_table_blocks,
            inode_table_blocks,
            total_group_size_blocks,
            total_fs_size,
        }
    }
}
#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy, Debug, PartialEq)]
pub struct Extent {
    base_block_index: u32,
    block_count: u32,
}
impl Default for Extent {
    fn default() -> Self {
        Self {
            base_block_index: Default::default(),
            block_count: Default::default(),
        }
    }
}
impl Extent {
    pub fn single(base_block_index: u32) -> Self {
        Self {
            base_block_index,
            block_count: 1,
        }
    }
    pub fn bytes(&self) -> u32 {
        self.block_count * BLOCK_SIZE_BYTES
    }

    pub fn base_byte_address(&self) -> u32 {
        self.base_block_index * BLOCK_SIZE_BYTES
    }
}
pub struct Disk {
    pub internal_file: File,
    pub metadata: SuperBlock,
    pub fs_info: FsInfo,
}
pub(crate) const BLOCK_SIZE_BYTES: u32 = 0x1000;
impl Disk {
    pub fn write_extent(&self, extent: Extent, value: Vec<u8>) -> Result<()> {
        if value.len() as u32 != extent.bytes() {
            bail!(
                "The amount of bytes:'{}' in the input_buffer wasn't equal to the amount of bytes required by the block count- BLOCK_SIZE_BYTES*block_count:'{}' ",
                value.len(),
                extent.bytes()
            );
        }
        self.internal_file
            .write_all_at(&value[..], extent.base_byte_address() as u64)
            .context("write data to internal file")
    }
    pub fn modify_extent<F>(&self, extent: Extent, modify_fn: F) -> Result<()>
    where
        F: Fn(&mut Vec<u8>),
    {
        let mut value = self.read_extent(extent)?;
        modify_fn(&mut value);
        self.write_extent(extent, value)?;
        Ok(())
    }
    pub fn read_extent(&self, extent: Extent) -> Result<Vec<u8>> {
        let mut buffer = vec![0u8; extent.bytes() as usize];
        let offset = extent.base_byte_address();
        self.internal_file
            .read_at(&mut buffer[..], offset as u64)
            .context("Read from a block did not succeed this probably means that the file system was corrupted.")?;
        Ok(buffer)
    }
    pub fn new() -> Self {
        let path = project_dir(INTERNAL_FILE_PATH);

        let internal_file = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(path)
            .expect("internal file path for disk is invalid!");
        let metadata = SuperBlock::parse_from_internal_file(&internal_file)
            .expect("parsing metadata for a file system did not succeed!");
        Self {
            internal_file,
            fs_info: FsInfo::new(&metadata),
            metadata,
        }
    }
    pub fn read_inode(&self, inode_index: u32) -> Result<INode> {
        let group_idx = inode_index / self.metadata.inodes_per_group;
        let idx_inside_group = inode_index - group_idx * self.metadata.inodes_per_group;
        let block_of_the_inode_table =
            size_of::<INode>() as u32 * idx_inside_group / BLOCK_SIZE_BYTES;
        if block_of_the_inode_table
            != size_of::<INode>() as u32 * (idx_inside_group + 1) / BLOCK_SIZE_BYTES
        {
            bail!(
                "Current implementation doesn't support data of inode being split across many blocks"
            );
        }
        let block_to_read_idx = self.fs_info.get_start_block_for_group(group_idx) + 2 /*Move past bitmaps*/ + block_of_the_inode_table;

        let block = self.read_extent(Extent {
            base_block_index: block_to_read_idx,
            block_count: 1,
        })?;
        let start_byte = idx_inside_group * size_of::<INode>() as u32
            - block_of_the_inode_table * BLOCK_SIZE_BYTES;
        let bytes_of_inode = &block[start_byte as usize..start_byte as usize + size_of::<INode>()];
        Ok(*INode::from_bytes_le(bytes_of_inode))
    }
    pub fn write_inode(&self, inode_index: u32, value: &INode) -> Result<()> {
        let group_idx = inode_index / self.metadata.inodes_per_group;
        let idx_inside_group = inode_index - group_idx * self.metadata.inodes_per_group;
        let block_of_the_inode_table =
            size_of::<INode>() as u32 * idx_inside_group / BLOCK_SIZE_BYTES;
        if block_of_the_inode_table
            != size_of::<INode>() as u32 * (idx_inside_group + 1) / BLOCK_SIZE_BYTES
        {
            bail!(
                "Current implementation doesn't support data of inode being split across many blocks"
            );
        }
        let block_to_read_idx = self.fs_info.get_start_block_for_group(group_idx) + 2 /*Move past bitmaps*/ + block_of_the_inode_table;
        let block_extent = Extent {
            base_block_index: block_to_read_idx,
            block_count: 1,
        };
        self.modify_extent(block_extent, |block| {
            let start_byte = idx_inside_group * size_of::<INode>() as u32
                - block_of_the_inode_table * BLOCK_SIZE_BYTES;
            let bytes = value.to_bytes_le();
            for i in 0..size_of::<INode>() {
                block[start_byte as usize + i] = bytes[i];
            }
        })
    }
    pub fn write_group_descriptor(&self, group_index: u32, value: GroupDescriptor) -> Result<()> {
        let block_idx = group_index * size_of::<GroupDescriptor>() as u32 / BLOCK_SIZE_BYTES + 1;
        let address_in_block =
            (group_index * size_of::<GroupDescriptor>() as u32) % BLOCK_SIZE_BYTES;
        let descriptor_bytes = value.to_bytes_le();

        self.modify_extent(
            Extent {
                base_block_index: block_idx,
                block_count: 1,
            },
            |bytes| {
                for i in 0..descriptor_bytes.len() {
                    bytes[address_in_block as usize + i] = descriptor_bytes[i]
                }
            },
        )
    }

    pub fn allocate_extent(&self, block_count: u32) -> Result<Extent> {
        for (i, mut group_descriptor) in GroupDescriptorIterator::new() {
            if group_descriptor.free_blocks_count < block_count {
                continue;
            }
            if let Some(block_indx) = group_descriptor.get_free_blocks_segment(block_count)? {
                // mark as allocated
                let bitmap_extent = Extent::single(group_descriptor.block_bitmap_block_index);
                self.modify_extent(bitmap_extent, |bytes| {
                    set_bit_in_block(block_indx as u32, bytes);
                })?;

                group_descriptor.free_blocks_count -= block_count;
                self.write_group_descriptor(i, group_descriptor)?;
                return Ok(Extent {
                    base_block_index: block_indx as u32,
                    block_count: block_count,
                });
            }
        }
        bail!("All groups were invalid!")
    }
    pub fn allocate_inode_idx(&self) -> Result<u32> {
        for (i, mut group_descriptor) in GroupDescriptorIterator::new() {
            if group_descriptor.free_inodes_count == 0 {
                continue;
            }
            if let Some(inode_index) = group_descriptor.get_free_inode_index()? {
                // mark as allocated
                let bitmap_extent = Extent::single(group_descriptor.inode_bitmap_block_index);
                self.modify_extent(bitmap_extent, |bytes| {
                    set_bit_in_block(inode_index as u32, bytes);
                })?;

                group_descriptor.free_blocks_count -= 1;
                self.write_group_descriptor(i, group_descriptor)?;
                return Ok(inode_index as u32);
            }
        }
        bail!("All groups were invalid!")
    }
    pub fn append_to_inode_contents(&self, inode: &INode, to_append: Vec<u8>) -> Result<()> {
        let to_write = inode.get_blocks_to_write_to(to_append.len() as u32)?;
        debug!(
            "append_to_inode_contents: to_write:'{to_write:?}', bytes_to_write{:?}",
            to_append.len()
        );
        let mut bytes_already_written = 0;
        for block_operation in to_write {
            let value = &to_append[bytes_already_written..block_operation.bytes_count as usize];
            bytes_already_written += block_operation.bytes_count as usize;

            let mut block = self.read_extent(Extent::single(block_operation.block_index))?;
            for i in 0..block_operation.bytes_count as usize {
                let addr = i + block_operation.base_block_local_address as usize;
                block[addr] = value[i];
            }
            self.write_extent(Extent::single(block_operation.block_index), block)?;
        }
        Ok(())
    }
    pub fn append_with_alloc_to_inode_contents(
        &self,
        inode: &mut INode,
        inode_index: u32,
        to_append: Vec<u8>,
    ) -> Result<()> {
        while inode.allocated_bytes() - inode.size < to_append.len() as u32 {
            let alloc = self.allocate_extent(1)?;
            for i in 0..inode.extents.len() {
                if inode.extents[i].base_block_index == 0 {
                    inode.extents[i] = alloc;
                    break;
                }
            }
        }
        inode.size += to_append.len() as u32;
        self.write_inode(inode_index, inode)?;
        self.append_to_inode_contents(inode, to_append)
    }

    pub fn add_entry_at_root(
        &self,
        file_name: &str,
        mode: u16,
        uid: u32,
        gid: u32,
        flags: u32,
        contents: Vec<u8>,
    ) -> Result<()> {
        let root_inode_idx = {
            let root_inode_data_addr = 6 * 4;
            let super_block = self.read_extent(Extent::single(0))?;
            u32::from_le_bytes(
                super_block[root_inode_data_addr..root_inode_data_addr + 4].try_into()?,
            )
        };
        let allocated_inode_idx: u32 = {
            let new_inode_idx = self.allocate_inode_idx()?;
            let mut root_inode = self.read_inode(root_inode_idx)?;
            let file_entry = DirEntry {
                inode: new_inode_idx,
                name_len: file_name.len() as u32,
                name: file_name.bytes().collect(),
            };

            self.append_with_alloc_to_inode_contents(
                &mut root_inode,
                root_inode_idx,
                file_entry.to_bytes_le(),
            )?;
            new_inode_idx
        };

        let blocks_needed_for_contents = (contents.len() as u32).div_ceil(BLOCK_SIZE_BYTES);
        let allocation = self.allocate_extent(blocks_needed_for_contents)?;
        let mut extents = [Extent::default(); 12];
        extents[0] = allocation;
        let mut inode = INode {
            mode,
            links: 1,
            uid,
            gid,
            size: contents.len() as u32,
            extents,
            flags,
            _pad: 0,
        };
        self.append_with_alloc_to_inode_contents(&mut inode, allocated_inode_idx, contents)?;
        Ok(())
    }
}
pub struct GroupDescriptorIterator {
    i: u32,
    current_block: Vec<u8>,
    current_block_index: u32,
}

impl GroupDescriptorIterator {
    pub fn new() -> Self {
        Self {
            i: 0,
            current_block: vec![],
            current_block_index: 0,
        }
    }
}

const GROUP_DESCRIPTOR_BYTES: u32 = size_of::<GroupDescriptor>() as u32;
impl Iterator for GroupDescriptorIterator {
    type Item = (u32, GroupDescriptor);

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= DISK.metadata.group_count {
            return None;
        }
        let base_block_index = 1 + self.i * GROUP_DESCRIPTOR_BYTES / BLOCK_SIZE_BYTES;
        if self.current_block_index != base_block_index {
            self.current_block_index = base_block_index;

            self.current_block = DISK
                .read_extent(Extent {
                    base_block_index,
                    block_count: 1,
                })
                .ok()?;
        }
        let base_address_in_block = ((self.i * GROUP_DESCRIPTOR_BYTES) % BLOCK_SIZE_BYTES) as usize;
        let bytes = &self.current_block
            [base_address_in_block..base_address_in_block + GROUP_DESCRIPTOR_BYTES as usize];
        self.i += 1;
        Some((self.i, *GroupDescriptor::from_bytes_le(bytes)))
    }
}
#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy)]
pub struct GroupDescriptor {
    block_bitmap_block_index: u32,
    inode_bitmap_block_index: u32,
    inode_table_start_block_index: u32,

    free_blocks_count: u32,
    free_inodes_count: u32,
}
fn first_zero_run(buf: &[u8], x: usize) -> Option<usize> {
    let mut count = 0;
    let mut start = 0;

    for (byte_idx, &byte) in buf.iter().enumerate() {
        for bit in 0..8 {
            let bit_idx = byte_idx * 8 + bit;

            if (byte & (1 << bit)) == 0 {
                if count == 0 {
                    start = bit_idx;
                }
                count += 1;

                if count >= x {
                    return Some(start);
                }
            } else {
                count = 0;
            }
        }
    }

    None
}
impl GroupDescriptor {
    pub fn get_free_inode_index(&self) -> Result<Option<usize>> {
        let block = DISK.read_extent(Extent::single(self.inode_bitmap_block_index))?;
        Ok(first_zero_run(
            &block[0..DISK.metadata.inodes_per_group as usize / 8],
            1,
        ))
    }
    pub fn get_free_blocks_segment(&self, needed_blocks: u32) -> Result<Option<usize>> {
        let block = DISK.read_extent(Extent::single(self.block_bitmap_block_index))?;
        Ok(first_zero_run(
            &block[0..DISK.metadata.blocks_per_group as usize / 8],
            needed_blocks as usize,
        ))
    }
    pub fn to_bytes_le(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
    pub fn from_bytes_le(bytes: &[u8]) -> &Self {
        bytemuck::from_bytes(bytes)
    }
}

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Debug, PartialEq, Copy)]
pub struct INode {
    mode: u16,  // File type + permissions
    links: u16, // Link count - how many other nodes reference this node

    uid: u32,
    gid: u32,

    size: u32, // Bytes

    flags: u32,
    _pad: u32,

    extents: [Extent; 12], // [(high 32 bits: u32 base block address, low 32 bits: block count);12]
}
impl INode {
    pub fn get_blocks_to_write_to(&self, needed_bytes: u32) -> Result<Vec<BlockOperation>> {
        let mut output_operations = vec![];
        let mut bytes_to_write = needed_bytes;
        let mut totall_used_bytes_left = self.size;
        for extent in self.extents {
            for block_index in extent.base_block_index..extent.base_block_index + extent.block_count
            {
                let mut free_bytes = BLOCK_SIZE_BYTES;
                let base_block_local_address = totall_used_bytes_left;
                let used = totall_used_bytes_left.min(free_bytes);
                totall_used_bytes_left -= used;
                free_bytes -= used;
                if free_bytes == 0 {
                    continue;
                }
                if bytes_to_write == 0 {
                    break;
                }
                let bytes_count = free_bytes.min(bytes_to_write);
                bytes_to_write -= bytes_count;
                output_operations.push(BlockOperation {
                    block_index,
                    base_block_local_address,
                    bytes_count,
                });
            }
        }
        Ok(output_operations)
    }
    pub fn allocated_bytes(&self) -> u32 {
        let mut sum = 0;
        self.extents
            .iter()
            .for_each(|extent| sum += extent.block_count);
        sum * BLOCK_SIZE_BYTES
    }

    pub fn from_bytes_le(bytes: &[u8]) -> &Self {
        bytemuck::from_bytes(bytes)
    }
    pub fn to_bytes_le(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

pub fn init_file_system(metadata: SuperBlock, internal_file: &mut File) -> Result<()> {
    let fs_info = FsInfo::new(&metadata);
    // Allocate enough space
    {
        const BATCHES: usize = 100;
        let buff = &vec![0u8; fs_info.total_fs_size as usize / BATCHES];
        for _ in 0..BATCHES {
            internal_file.write_all(buff)?;
        }
        internal_file.write_all(&vec![
            0u8;
            fs_info.total_fs_size as usize
                - buff.len() * BATCHES as usize
        ])?;
    }

    internal_file.rewind()?;
    // Super Block
    internal_file.write_all(&(0x00325246_u32.to_le_bytes()))?; // magic
    internal_file.write_all(&metadata.total_blocks.to_le_bytes())?;
    internal_file.write_all(&metadata.total_inodes.to_le_bytes())?;
    internal_file.write_all(&metadata.blocks_per_group.to_le_bytes())?;
    internal_file.write_all(&metadata.inodes_per_group.to_le_bytes())?;
    internal_file.write_all(&metadata.group_count.to_le_bytes())?;
    internal_file.write_all(&metadata.inode_size.to_le_bytes())?;
    internal_file.write_all(&metadata.root_inode.to_le_bytes())?;
    internal_file.write_all(&metadata.first_data_block.to_le_bytes())?;
    internal_file.write_all(&metadata.flags.to_le_bytes())?;

    internal_file.seek(std::io::SeekFrom::Start(1 * BLOCK_SIZE_BYTES as u64))?;
    let base_block_index_for_group = |i: u32| -> u32 {
        1 + fs_info.group_descriptor_table_blocks + fs_info.total_group_size_blocks * i
    };
    // Group Descriptor Table
    for i in 0..metadata.group_count {
        let base_block = base_block_index_for_group(i);
        let value: GroupDescriptor = GroupDescriptor {
            block_bitmap_block_index: base_block,
            inode_bitmap_block_index: base_block + 1,
            inode_table_start_block_index: base_block + 2 + base_block,

            free_blocks_count: metadata.blocks_per_group,
            free_inodes_count: metadata.inodes_per_group,
        };
        let bytes = &value.to_bytes_le();
        internal_file.write_all(bytes)?;
    }
    // Groups can be skipped because they will be all zeros

    Ok(())
}

pub struct DirEntry {
    pub inode: u32,
    pub name_len: u32,
    pub name: Vec<u8>,
}
impl DirEntry {
    pub fn to_bytes_le(&self) -> Vec<u8> {
        let mut output = vec![];
        output.extend_from_slice(&self.inode.to_le_bytes());
        output.extend_from_slice(&self.name_len.to_le_bytes());
        output.extend(&self.name);
        output
    }
}
#[derive(Debug)]
pub struct BlockOperation {
    pub block_index: u32,
    pub base_block_local_address: u32,
    pub bytes_count: u32,
}
