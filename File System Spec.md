# Simple Block-Group Filesystem Specification FR2

---

## 1. Overview

The filesystem is a block-based, inode-driven storage system with:

- Fixed-size blocks - ALWAYS 4096 bytes wide
- Block groups for locality and scalability
- Bitmaps for allocation tracking
- Fixed-size inodes
- Direct extent block addressing- no indirect addressing

---

## 2. On-Disk Layout

The filesystem is organized as follows:

```
+---------------------------+
| Superblock |
+---------------------------+
| Group Descriptor Table |
+---------------------------+
| Block Group 0 |
| Block Bitmap |
| Inode Bitmap |
| Inode Table |
| Data Blocks |
+---------------------------+
| Block Group 1 |
| Block Bitmap |
| Inode Bitmap |
| Inode Table |
| Data Blocks |
+---------------------------+
| ... |
+---------------------------+
```

---

## 3. Superblock

Located at a fixed 0 block offset

```rust
struct Superblock {

    magic:u32  = 0x00325246,  // Used to validate FR2 file system presence
    total_blocks:u32, 
    total_inodes:u32,
    blocks_per_group:u32,
    inodes_per_group:u32,
    group_count:u32,
    inode_size:u32,  // fixed (e.g. 128 bytes)
    root_inode:u32,         
    first_data_block:u32,   
    flags:u32,              // optional feature flags
}
```

---

## 4. Block Groups

### Layout

```
+------------------+
| Block Bitmap     |
+------------------+
| Inode Bitmap     |
+------------------+
| Inode Table      |
+------------------+
| Data Blocks      |
+------------------+
```

### Descriptor

```rust
struct GroupDescriptor {
    block_bitmap_start: u32,
    inode_bitmap_start: u32,
    inode_table_start: u32,
    
    free_blocks_count: u32,
    free_inodes_count: u32,
}
```

#### Allocation Bitmaps

Two bitmaps per group:

##### Block Bitmap

1 bit per block in group: 1 = used, 0 = free

##### Inode Bitmap

1 bit per inode in group: 1 = used, 0 = free

---

## 6. Inodes

Fixed-size metadata structure representing files or directories.

```rust
struct Inode {
    mode: u16,        // file type + permissions
    links: u16,       // link count - how many other nodes reference this node

    uid: u32,
    gid: u32,

    size: u32,        // bytes

    extents:[u64;12],  // [(u32 base block adress, block count);12] 

    flags:u32,
}
```

## 7. Directories

Directories are files containing directory entries. Entries are packed
sequentially within data blocks. `inode =` 0 means unused entry.

```rust
struct DirEntry { 
        inode:u32,
        name_len: u32,
        name: [char],
}
```
