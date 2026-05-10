use anyhow::Result;

use crate::emulator::disk::*;
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
impl Disk {
    pub fn defrag_inode_allocation(&self, inode: &mut INode, inode_index: u32) -> Result<()> {
        // Try to allocate the biggest amount of block possible.
        for extents_to_combine in (1..inode.extents.len()).rev() {
            let mut blocks_to_allocate = 0;
            for i in 0..extents_to_combine {
                blocks_to_allocate += inode.extents[i].block_count;
            }

            if let Ok(allocation) = self.allocate_blocks(blocks_to_allocate) {
                debug!(
                    "defrag_inode_allocation: allocation:{allocation:?}, blocks_to_allocate:{blocks_to_allocate}, extents_to_combine:{extents_to_combine}, inode:{inode:?}"
                );
                // TODO: Don't use writer- slow for just moving whole blocks
                let mut used_blocks_of_new_allocation = 0;
                for i in 0..extents_to_combine {
                    let extent = inode.extents[i];
                    let value_to_move = self.read_extent(extent)?;

                    self.write_extent(
                        Extent {
                            base_block_index: allocation.base_block_index
                                + used_blocks_of_new_allocation,
                            block_count: extent.block_count,
                        },
                        &value_to_move,
                    )?;
                    used_blocks_of_new_allocation += extent.block_count;
                    self.mark_extend_as_free_data_blocks(extent)?;
                    inode.extents[i] = Extent::zero();
                }
                inode.extents[0] = allocation;
                self.write_inode(inode_index, inode)?;

                return Ok(());
            }
            // allocate smaller amount of blocks
        }
        bail!("inode defragmentation did not succeed- couldn't defrag it");
    }
    pub fn mark_extend_as_free_data_blocks(&self, extend: Extent) -> Result<()> {
        let (group_info, group_index) = self
            .fs_info
            .get_group_info_for_block(extend.base_block_index);
        if extend.base_block_index < group_info.first_data_block_index {
            bail!("Some blocks of this extent aren't data blocks");
        }

        let base_block_in_group = extend.base_block_index - group_info.first_data_block_index;
        if base_block_in_group + extend.block_count > self.superblock.data_blocks_per_group {
            bail!("Blocks of this extent are on multiple different block groups");
        }

        // Group Descriptor

        let mut group_descriptor = self.read_group_descriptor(group_index)?;
        group_descriptor.free_blocks_count += extend.block_count;
        self.write_group_descriptor(group_index, group_descriptor)?;

        // Bitmap
        self.modify_extent(Extent::single(group_info.inode_bitmap_index), |bytes| {
            for i in 0..extend.block_count {
                set_bit_in_block(extend.block_count + i, bytes);
            }
        })?;

        Ok(())
    }
    /// Returns global index of the first free inode.
    /// WARNING: group descriptor's free inodes value isn't modified
    pub fn allocate_free_inode_for_group(
        &self,
        group: u32,
        mark_bitmap_as_used: bool,
    ) -> Result<Option<usize>> {
        let bitmap_block = self.fs_info.get_group_info(group).inode_bitmap_index;
        let block = DISK.read_extent(Extent::single(bitmap_block))?;
        Ok(
            match first_zero_run(&block[0..DISK.superblock.inodes_per_group as usize / 8], 1) {
                Some(local_inode_index) => {
                    if mark_bitmap_as_used {
                        DISK.modify_extent(Extent::single(bitmap_block), |bytes| {
                            set_bit_in_block(local_inode_index as u32, bytes);
                        })?;
                    }
                    Some(local_inode_index + (self.superblock.inodes_per_group * group) as usize)
                }
                None => None,
            },
        )
    }
    /// Returns global index of the first free block for a continuous segment of x free blocks.
    /// WARNING: group descriptor's free blocks value isn't modified
    pub fn allocate_free_blocks_in_group(
        &self,
        needed_blocks: u32,
        group: u32,
        mark_bitmap_as_used: bool,
    ) -> Result<Option<usize>> {
        let group_info = self.fs_info.get_group_info(group);
        let bitmap_block = group_info.data_block_bitmap_index;
        let block = DISK.read_extent(Extent::single(bitmap_block))?;
        Ok(
            match first_zero_run(
                &block[0..DISK.superblock.data_blocks_per_group as usize / 8],
                needed_blocks as usize,
            ) {
                Some(local_block_index) => {
                    if mark_bitmap_as_used {
                        DISK.modify_extent(Extent::single(bitmap_block), |bytes| {
                            set_bit_in_block(local_block_index as u32, bytes);
                        })?;
                    }
                    debug!(
                        "allocate_free_blocks_for_group: local_block_index:'{local_block_index}', group_info.first_data_block_index:'{}'",
                        group_info.first_data_block_index
                    );
                    Some(local_block_index + group_info.first_data_block_index as usize)
                }
                None => None,
            },
        )
    }

    pub fn allocate_blocks(&self, block_count: u32) -> Result<Extent> {
        for (i, mut group_descriptor) in GroupDescriptorIterator::new() {
            debug!(
                "allocate_blocks: block_count:{block_count}, group_descriptor:{group_descriptor:?}  "
            );

            if group_descriptor.free_blocks_count < block_count {
                continue;
            }
            if let Some(base_block_index) =
                self.allocate_free_blocks_in_group(block_count, i, true)?
            {
                group_descriptor.free_blocks_count -= block_count;
                self.write_group_descriptor(i, group_descriptor)?;
                return Ok(Extent {
                    base_block_index: base_block_index as u32,
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
            if let Some(inode_index) = self.allocate_free_inode_for_group(i, true)? {
                group_descriptor.free_blocks_count -= 1;
                self.write_group_descriptor(i, group_descriptor)?;
                return Ok(inode_index as u32);
            }
        }
        bail!("All groups were invalid!")
    }
}
