# I/O Architecture

## Serial Debug Device

Base: 0xE0300000 Interrupt ID: 3 (fixed)

---

### Registers

- +0x00 OUT_INFO (write, 8-bit used)
- +0x04 OUT_DEBUG (write, 8-bit used)
- +0x08 OUT_WARN (write, 8-bit used)
- +0x0C OUT_ERR (write, 8-bit used)

- +0x10 IN_DATA (read, 8-bit)
- +0x14 STATUS (read)
- +0x18 CONTROL (write)

---

### STATUS (Bitfield)

bit 0 → IN_READY (1 = input available) bit 1 → ENABLED (1 = device enabled)

---

### CONTROL (Bitfield)

bit 0 → ENABLE (1 = enable device) bit 1 → ACK (write 1 to acknowledge input /
clear interrupt)

---

### Behavior

Output:

- writing to OUT_* prints a single byte to stdout
- channel determines log level (INFO/DEBUG/WARN/ERR)

Input:

- emulator collects user input into internal buffer
- when at least one byte is available: → STATUS.IN_READY = 1 → raise interrupt
  (ID = 3)

Reading input:

- CPU reads IN_DATA → returns next byte
- if buffer becomes empty: → STATUS.IN_READY = 0

Acknowledging:

- CPU writes CONTROL.ACK = 1 → clears interrupt

Enable:

- if ENABLE = 0: → no output → input buffer paused
