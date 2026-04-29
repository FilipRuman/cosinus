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
