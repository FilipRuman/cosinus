# I/O Architecture

## Serial Debug Device

Base: 0xE0300000 Interrupt ID: 3 (fixed)

---

### Registers

- +0x00 OUT (write, 8-bit used)
- +0x10 IN_DATA (read, 8-bit)
- +0x14 STATUS (read)
- +0x18 CONTROL (write)

---

### STATUS (Bitfield)

bit 0 → IN_READY (1 = input available)

---

### CONTROL (Bitfield)

bit 0 → ACK (write 1 to acknowledge input / clear interrupt)

---

### Behavior

Output:

- writing to OUT prints a single byte to stdout

Input:

- emulator collects user input into internal buffer
- when at least one byte is available: → STATUS.IN_READY = 1 → raise interrupt
  (ID = 3)

Reading input:

- CPU reads IN_DATA → returns next byte
- if buffer becomes empty: → STATUS.IN_READY = 0

Acknowledging:

- CPU writes CONTROL.ACK = 1 → clears interrupt

## Disk

Base: 0xE0000000 Interrupt ID: 0 (fixed)

BLOCK_SIZE = 4096B;

### Registers

- +0x00 block_index (write)
- +0x04 buffer_address (write)
- +0x08 block_count(will be used as a buffer size- buffer size = BLOCK_SIZE *
  block count) (write)
- +0x0C core_id_to_interrupt (write)
- +0x10 status (read)
- +0x14 control (write)

### Status (Bitfield)

- bit 0 → READY (0 = busy 1 = ready to use)
- bit 1 → ERROR (0 = operation succeeded 1 = error)
- bit 2-7 -> ERROR CODES TODO:

### Control (Bitfield)

Upon write starts executing command with the specified configuration:

- bit 0 → READ/WRITE (0 = read 1 = write) bit 0 → READ/WRITE (0 = read 1 =
  write)

### USAGE

0. Load data as needed to a buffer.
1. Check if other threads don't use this device(kernel level not device) and
   mark it as being used.
2. Wait until the device is ready- STATUS
3. Set registers with the desired values - block_index, buffer_address,
   buffer_length, core_id_to_interrupt

#### Read

5. Set the control bit to read.
6. mark drive as free to use(kernel level not device).
7. Do something else until gets interrupted by the drive.
8. Data is ready to use

#### Write

5. Set the control bit to write.
6. mark drive as free to use(kernel level not device).
7. Do something else until gets interrupted by the drive.
8. Upon interrupt data, from the buffer, was written to the specified blocks.
