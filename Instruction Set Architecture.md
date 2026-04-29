# Instruction Set Architecture

Legend:

- imm16, imm26 -> 16/26 bit immidiate value
- rd -> input register specified in the instruction
- rs1, rs2, rs3 -> input register specified in the instruction\
  *(rs1) -> memory at address specified by the contents of the rs1 register
  *(rs2 + imm16) -> memory at address specified by the contents of the rs2
  register + imm16 immediate value

## Hardware

- 32 bit
- 32 general purpose registers
- Real multithreading - at least 3 threads
- Sequential instructions execution
- Flat Memory Layout - No MMU
- LR/SC Atomic operations
- Context switching support: software managed.
- Little-endian
- All instructions as well as PC use byte addresses.
- Immidiates on all expressions except binary operations(or, and, xor -> zero
  extended imm16) are sign extended up to 32 bit

## Address Space

2^32 memory. I/O memory segment is maped for hardware. Privelaged memory space
read/write safety is ensured by checking the psr privilege flag at the
instruction execution time.

```s
|-User Space
|-|RAM          (< 0xD0000000)
  |-| user code
    | user data
    | user stacks
  | Framebuffer  (0xD0000000..0xE0000000) 
  | I/O          (0xE0000000..0xF0000000)
  |-| Disk                  (0xE0000000...)
    | Timer                 (0xE0100000...)
    | Audio                 (0xE0200000...)
    | Serial                (0xE0300000...)
    | Some other device     (0xE0400000...)
    | ...
|Kernel Space -> Memory access throws an exception if the privelage flag in the psr register is false   
|-|Kernel       (> 0xF0000000)   
  |-| kernel code
    | kernel data
    | kernel stacks
    | interrupt vector table
```

## Registers

### General Purpose

- r0 = zero -> hard wired zero register
- r1 = sp -> stack pointer
- r2 = ra -> return address
- r3 = fp -> frame pointer
- r4–r10 -> arguments / caller-saved
- r11–r16 -> return / callee-saved
- r17–r31 -> scratch

### System Registers

You interact with those by special instructions. Some of the instructions check
the privelage mode-> look at the instruction set

- pc -> programmable counter - byte address of the current instruction.
  Automatically grows by 4(4*8=32) after each instruction.
- psr -> processor status register
- ivt -> interrupt table base address
- imr -> interrupt mask register
- epc -> stores PC when interrupt occurs
- tid -> thread id register
- etr -> exception type register

### PSR- Processor Status Register

1. PM — Privilege Mode (1 bit)

- PM = 0 → user mode
- PM = 1 → kernel mode

Used for:

- Simple kernel memory protection
- Privileged instruction checks (SYSW, SRET, etc.)

2. IE — Interrupt Enable (2 bit)

- IE = 1 → interrupts enabled
- IE = 0 → interrupts disabled

Used by CPU:

if (IE == 1) and (IPR & ~IMR != 0) → take interrupt

3. HALT -> Makes thread sleep until the next interrupt (3 bit)

- HALT= 1 → HALT enabled
- HALT= 0 → HALT disabled

```
31                                      0
+----+----+--------------------------------+
| IE | PM |            Un used             |
+----+----+--------------------------------+
```

### Core Private Registers

- ipr -> interrupt pending register
- resv_addr -> reserved address for the atomic operations.
- resv_valid -> is the reservation valid flag for the atomic operations.

## Atomic Instructions Behavior

You can have only one lock at a time. Creating a new lock before using the
previous one (SC) is an undefined behavior.

- LR(Load Reserved) rd, rs1: Loads data from *(rs1) and creates
  reservation(lock) for that address. The reservation may be invalidated by any
  [of these](#reservation-invalidation-triggers).
- SC (store-conditional) rd, rs1, rs2: Stores value from rs2 at *(rs1) **ONLY**
  if there is a reservation for that **EXACT** ADDRESS that **WASN'T YET
  INVALIDATED**. Calling SC removes reservation for this thread from any
  address.\
  The rd register says whether that operation succeeded:
  - rd == 0: False, that operation did not succeed the *(rs1) has it's original
    contents.
  - rd != 0: True, that operation did succeed and the *(rs1) = rs2.

### Reservation Invalidation Triggers

- any write to same address
- context switch
- interrupt

## Instruction Set

Notes:

- All instructions are 32-bit
- Opcode is 6 bits (0–63)
- imm16 = signed 16-bit immediate
- Mode: U = user K = kernel-only
- Unused bits are irrelevant
- Decoder only needs to look at opcode to determine format
- All arithmetic operations will wrapp values-> over/under-flows will not cause
  exceptions
- Division by zero returns 0 without throwing any exceptions

### Boolean Operations Rules

- Bool is declared as: TRUE: value != 0, FALSE: value == 0.
- Boolean instructions will return 0 for false and 1 for true.
- Try to use != 0 for bool checking in your code.

### Example Encoding

```bs
31          26 25    21 20    16 15    11 10        0
+-------------+--------+--------+--------+-----------+
|   opcode    |   rd   |  rs1   |  rs2   |  unused   |
+-------------+--------+--------+--------+-----------+
```

---

### Arithmetic & Logical (Register-Register)

| Mode | Opcode | Hex  | Inputs       | Meaning                |
| :--: | :----: | :--: | :----------- | :--------------------- |
|  U   |  ADDR  | 0x00 | rd, rs1, rs2 | rd = rs1 + rs2         |
|  U   |  SUBR  | 0x01 | rd, rs1, rs2 | rd = rs1 - rs2         |
|  U   |  ANDR  | 0x02 | rd, rs1, rs2 | rd = rs1 & rs2         |
|  U   |  ORR   | 0x03 | rd, rs1, rs2 | rd = rs1 or rs2        |
|  U   |  XORR  | 0x04 | rd, rs1, rs2 | rd = rs1 ^ rs2         |
|  U   |  MULR  | 0x05 | rd, rs1, rs2 | rd = rs1 * rs2         |
|  U   |  DIVR  | 0x06 | rd, rs1, rs2 | rd = rs1 / rs2         |
|  U   |  REMR  | 0x07 | rd, rs1, rs2 | rd = rs1 % rs2         |
|  U   |  SHLR  | 0x08 | rd, rs1, rs2 | rd = rs1 << rs2        |
|  U   |  SHRR  | 0x09 | rd, rs1, rs2 | rd = rs1 >> rs2        |
|  U   |  SARR  | 0x0A | rd, rs1, rs2 | arithmetic shift right |

---

### Arithmetic & Logical (Immediate)

| Mode | Opcode | Hex  | Inputs         | Meaning                |
| :--: | :----: | :--: | :------------- | :--------------------- |
|  U   |  ADD   | 0x0B | rd, rs1, imm16 | rd = rs1 + imm         |
|  U   |  SUB   | 0x0C | rd, rs1, imm16 | rd = rs1 - imm         |
|  U   |  AND   | 0x0D | rd, rs1, imm16 | rd = rs1 & imm         |
|  U   |   OR   | 0x0E | rd, rs1, imm16 | rd = rs1 or imm        |
|  U   |  XOR   | 0x0F | rd, rs1, imm16 | rd = rs1 ^ imm         |
|  U   |  MUL   | 0x10 | rd, rs1, imm16 | rd = rs1 * imm         |
|  U   |  DIV   | 0x11 | rd, rs1, imm16 | rd = rs1 / imm         |
|  U   |  REM   | 0x12 | rd, rs1, imm16 | rd = rs1 % imm         |
|  U   |  SHL   | 0x13 | rd, rs1, imm16 | rd = rs1 << imm        |
|  U   |  SHR   | 0x14 | rd, rs1, imm16 | logical shift right    |
|  U   |  SAR   | 0x15 | rd, rs1, imm16 | arithmetic shift right |
|  U   |  NOT   | 0x3A | rd, rs1        | bitwise NOT            |
|  U   |  LUI   | 0x16 | rd, imm16      | rd = imm16 << 16       |

---

### Memory (Base + Immediate)

| Mode | Opcode | Hex  | Inputs          | Meaning            |
| :--: | :----: | :--: | :-------------- | :----------------- |
|  U   |  LOAD  | 0x17 | rd, rs1, imm16  | rd = *(rs1 + imm)  |
|  U   | STORE  | 0x18 | rs1, rs2, imm16 | *(rs1 + imm) = rs2 |
|  U   | LOADB  | 0x19 | rd, rs1, imm16  | load 8-bit         |
|  U   | STOREB | 0x1A | rs1, rs2, imm16 | store 8-bit        |
|  U   | LOADH  | 0x1B | rd, rs1, imm16  | load 16-bit        |
|  U   | STOREH | 0x1C | rs1, rs2, imm16 | store 16-bit       |
|  U   | LOADPC | 0x1D | rd, imm16       | rd = *(PC + imm)   |

---

### Control Flow

| Mode | Opcode | Hex  | Inputs    | Meaning       |
| :--: | :----: | :--: | :-------- | :------------ |
|  U   |  JMP   | 0x1E | imm26     | PC += imm     |
|  U   |  CALL  | 0x1F | imm26     | ra = PC; jump |
|  U   |  RET   | 0x20 | —         | PC = ra       |
|  U   |  JMPR  | 0x21 | rs, imm16 | PC = rs + imm |
|  U   |  APC   | 0x22 | rd, imm16 | rd = PC + imm |

---

### Branching

| Mode | Opcode | Hex  | Inputs          | Meaning                         |
| :--: | :----: | :--: | :-------------- | :------------------------------ |
|  U   |  BEQ   | 0x23 | rs1, rs2, imm16 | if rs1 equal rs2 ->jump imm     |
|  U   |  BNE   | 0x24 | rs1, rs2, imm16 | if rs1 not equal rs2 ->jump imm |
|  U   |  BLT   | 0x25 | rs1, rs2, imm16 | if rs1 less rs2 ->jump imm      |
|  U   |  BGT   | 0x26 | rs1, rs2, imm16 | if rs1 greater rs2 ->jump imm   |
|  U   |  BLE   | 0x27 | rs1, rs2, imm16 | if rs1 <= rs2 ->jump imm        |
|  U   |  BGE   | 0x28 | rs1, rs2, imm16 | if rs1 >= rs2 ->jump imm        |

---

### System Calls

| Mode | Opcode | Hex  | Inputs | Meaning |
| :--: | :----: | :--: | :----- | :------ |
|  U   | SCALL  | 0x29 | —      | syscall |
|  K   |  SRET  | 0x2A | —      | return  |

---

### System Registers

| Mode | Opcode | Hex  | Inputs     | Meaning      |
| :--: | :----: | :--: | :--------- | :----------- |
|  U   |  SYSR  | 0x2B | rd, imm16  | read sysreg  |
|  K   |  SYSW  | 0x2C | rs1, imm16 | write sysreg |

Sysregs: 0. PSR

1. IVT
2. IMR
3. EPC
4. TID
5. ETR

---

### Atomic (LR/SC)

| Mode | Opcode | Hex  | Inputs       | Meaning           |
| :--: | :----: | :--: | :----------- | :---------------- |
|  U   |   LR   | 0x2D | rd, rs1      | load-reserved     |
|  U   |   SC   | 0x2E | rd, rs1, rs2 | conditional store |

---

### Misc

| Mode | Opcode | Hex  | Inputs | Meaning  |
| :--: | :----: | :--: | :----- | :------- |
|  U   |  NOP   | 0x2F | —      | no-op    |
|  U   |  HALT  | 0x30 | —      | stop CPU |

---

### Compare

| Mode | Opcode | Hex  | Inputs            | Meaning              |
| :--: | :----: | :--: | :---------------- | :------------------- |
|  U   |  LTR   | 0x31 | rd, rs1, rs2      | rd = (rs1 < rs2)     |
|  U   |  EQR   | 0x32 | rd, rs1, rs2      | rd = (rs1 == rs2)    |
|  U   |   LT   | 0x33 | rd, rs1, imm16    | rd = rs1 < imm16     |
|  U   |   EQ   | 0x34 | rd, rs1, imm16    | rd = rs1 == imm16    |
|  U   |  SEL   | 0x37 | rd, rs1, rs2, rs3 | rd = rs3 ? rs1 : rs2 |
|  U   |  CTZ   | 0x38 | rd, rs1           | trailing zeros       |
|  U   |  CLZ   | 0x39 | rd, rs1           | leading zeros        |
