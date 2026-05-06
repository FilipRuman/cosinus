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
|Block 0. Superblock Metadata
|---|   >Group Descriptor Table<  
    | Block 1. GroupDescriptor 1
    | Block 2. GroupDescriptor 2
    | Block X. GroupDescriptor X
|---|       >Data Blocks<
    |---|       >Block Group 0<
        | Block X + 1. Block Group 0 Metadata
        | Block X + 2. Data Block 0 
        | Block X + 3. Data Block 1 
        | Block X + 2 + Y. Data Block Y 

    |---|       >Block Group 1<
        | Block X + Y + 1. Block Group 1 Metadata
        | Block X + Y + 2. Data Block 0 
        | Block X + Y + 3. Data Block 1 
        | Block X + 2 * Y. Data Block Y

    |---|       >Block Group X<
        | Block X + X * Y + 1. Block Group X Metadata
        | Block X + X * Y + 2. Data Block 0 
        | Block X + X * Y + 3. Data Block 1 
        | Block X + (X + 1) *  * Y. Data Block Y
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

#### Allocation Bitmaps

Two bitmaps per group:

##### Block Bitmap

1 bit per block in group: 1 = used, 0 = free

##### Inode Bitmap

1 bit per inode in group: 1 = used, 0 = free

### Descriptor

```rust
struct GroupDescriptor {
    block_bitmap_block_index: u32,
    inode_bitmap_block_index: u32,
    inode_table_start_block_index: u32,
    
    free_blocks_count: u32,
    free_inodes_count: u32,
}
```

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

    extents: [u64; 12], // [(high 32 bits: u32 base block index, low 32 bits: block count);12]

    flags:u32,
}
```

### Mode

```text
15            13 12       9 8       6 5       3 2       0
+---------------+-----------+---------+---------+---------+
|  File Type    | Reserved  | User    | Group   | Other   |
|   (3 bits)    | (4 bits)  |  rwx    |  rwx    |  rwx    |
+---------------+-----------+---------+---------+---------+
```

#### File Type (bits 15–13)

- 000 → Regular file
- 001 → Directory
- 010 → Symbolic link (optional)

#### Permission Bits (bits 8–0)

- User:
  - read = 0b100 << 6
  - write = 0b010 << 6
  - execute = 0b001 << 6

- Group:
  - read = 0b100 << 3
  - write = 0b010 << 3
  - execute = 0b001 << 3

- Other:
  - read = 0b100
  - write = 0b010
  - execute = 0b001

## 7. Directories

Directories are files containing directory entries. Entries are packed
sequentially within data blocks. `inode = 0` means unused entry.

```rust
struct DirEntry { 
        inode:u32,
        name_len: u32,
        name: [char],
}
```
