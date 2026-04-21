# Instruction Set Architecture

## Hardware

- 32 bit
- 32 general purpose registers
- Real multithreading - at least 3 threads
- Sequential instructions execution
- Flat Memory Layout - No MMU
- LR/SC Atomic operations
- Context switching support: software managed.

## Address Space

2^32 memory. I/O memory segment is maped for hardware. Privelaged memory space
read/write safety is ensured by checking the psr privilege flag at the
instruction execution time.

```s
User Space
| RAM          (< 0xD0000000)
|-| user code
  | user data
  | user stacks
| Framebuffer  (0xD0000000..0xE0000000) 
| I/O          (0xE0000000..0xF0000000)
|-| Disk                  (0xE0000000...)
  | Timer                 (0xE0100000...)
  | Audio                 (0xE0200000...)
  | Some other device     (0xE0400000...)
  | ...
Mode Space -> Memory access throws an exception if the privelage flag in the psr register is false   
| Mode       (> 0xF0000000)   
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
- r4–r11 -> arguments / caller-saved
- r12–r23 -> callee-saved
- r24–r31 -> reserved / future ABI extension

### System Registers

You interact with those by special instructions. Some of the instructions check
the privelage mode-> look at the instruction set

- pc -> programable counter
- psr -> processor Status Register: TODO: add data format
- ivt -> interrupt table base address
- imr -> interrupt mask register
- epc -> stores PC when interrupt occurs
- tid -> thread id register

### Core Private Registers

- resv_addr -> reserved address for the atomic operations.
- resv_valid -> is the reservation valid flag for the atomic operations.

## Reservation Invalidation Triggers

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

---

### 1. Arithmetic & Logical (Register-Register)

| Mode | Opcode |  Bin   | Hex  |    Inputs    |     Meaning      |
| :--: | :----: | :----: | :--: | :----------: | :--------------: |
|  U   |  ADDR  | 000000 | 0x00 | rd, rs1, rs2 |  rd = rs1 + rs2  |
|  U   |  SUBR  | 000001 | 0x01 | rd, rs1, rs2 |  rd = rs1 - rs2  |
|  U   |  ANDR  | 000010 | 0x02 | rd, rs1, rs2 |  rd = rs1 & rs2  |
|  U   |  ORR   | 000011 | 0x03 | rd, rs1, rs2 |     rd = rs1     |
|  U   |  XORR  | 000100 | 0x04 | rd, rs1, rs2 |  rd = rs1 ^ rs2  |
|  U   |  MULR  | 000101 | 0x05 | rd, rs1, rs2 |  rd = rs1 * rs2  |
|  U   |  DIVR  | 000110 | 0x06 | rd, rs1, rs2 |  rd = rs1 / rs2  |
|  U   |  REMR  | 000111 | 0x07 | rd, rs1, rs2 |  rd = rs1 % rs2  |
|  U   |  SHLR  | 001000 | 0x08 | rd, rs1, rs2 | rd = rs1 << rs2  |
|  U   |  SHRR  | 001001 | 0x09 | rd, rs1, rs2 | rd = rs1 >> rs2  |
|  U   |  SARR  | 001010 | 0x0A | rd, rs1, rs2 | arithmetic shift |

```
31          26 25    21 20    16 15    11 10        0
+-------------+--------+--------+--------+-----------+
|   opcode    |   rd   |  rs1   |  rs2   |  unused   |
+-------------+--------+--------+--------+-----------+
```

---

### 2. Arithmetic & Logical (Immediate)

| Mode | Opcode |  Bin   | Hex  |     Inputs     |     Meaning      |
| :--: | :----: | :----: | :--: | :------------: | :--------------: |
|  U   |  ADD   | 001011 | 0x0B | rd, rs1, imm16 |  rd = rs1 + imm  |
|  U   |  SUB   | 001100 | 0x0C | rd, rs1, imm16 |  rd = rs1 - imm  |
|  U   |  AND   | 001101 | 0x0D | rd, rs1, imm16 |  rd = rs1 & imm  |
|  U   |   OR   | 001110 | 0x0E | rd, rs1, imm16 |     rd = rs1     |
|  U   |  XOR   | 001111 | 0x0F | rd, rs1, imm16 |  rd = rs1 ^ imm  |
|  U   |  MUL   | 010000 | 0x10 | rd, rs1, imm16 |  rd = rs1 * imm  |
|  U   |  DIV   | 010001 | 0x11 | rd, rs1, imm16 |  rd = rs1 / imm  |
|  U   |  REM   | 010010 | 0x12 | rd, rs1, imm16 |  rd = rs1 % imm  |
|  U   |  SHLI  | 010011 | 0x13 | rd, rs1, imm16 | rd = rs1 << imm  |
|  U   |  SHRI  | 010100 | 0x14 | rd, rs1, imm16 | rd = rs1 >> imm  |
|  U   |  SARI  | 010101 | 0x15 | rd, rs1, imm16 | arithmetic shift |

```
31          26 25    21 20    16 15                 0
+-------------+--------+--------+--------------------+
|   opcode    |   rd   |  rs1   |       imm16        |
+-------------+--------+--------+--------------------+
```

### 2. LUI-Type

| Mode | Opcode |  Bin   | Hex  |  Inputs   |    Meaning     |
| :--: | :----: | :----: | :--: | :-------: | :------------: |
|  U   |  LUI   | 010110 | 0x16 | rd, imm16 | rd = imm << 16 |

```
31          26 25    21 20                     5 4   0
+-------------+--------+------------------------+-----+
|   opcode    |   rd   |        imm16           |  0  |
+-------------+--------+------------------------+-----+
```

---

### 3. Memory (Base + Immediate)

| Mode | Opcode |  Bin   | Hex  |     Inputs      |      Meaning       |
| :--: | :----: | :----: | :--: | :-------------: | :----------------: |
|  U   |  LOAD  | 010111 | 0x17 | rd, rs1, imm16  | rd = *(rs1 + imm)  |
|  U   | STORE  | 011000 | 0x18 | rs2, rs1, imm16 | *(rs1 + imm) = rs2 |
|  U   | LOADB  | 011001 | 0x19 | rd, rs1, imm16  |     load 8-bit     |
|  U   | STOREB | 011010 | 0x1A | rs2, rs1, imm16 |    store 8-bit     |
|  U   | LOADH  | 011011 | 0x1B | rd, rs1, imm16  |    load 16-bit     |
|  U   | STOREH | 011100 | 0x1C | rs2, rs1, imm16 |    store 16-bit    |
|  U   | LOADPC | 011101 | 0x1D |    rd, imm16    |  rd = *(PC + imm)  |

```
31          26 25    21 20    16 15                 0
+-------------+--------+--------+--------------------+
|   opcode    |  rs2   |  rs1   |       imm16        |
+-------------+--------+--------+--------------------+
```

LOAD PC:

```
31          26 25    21 20                     5 4   0
+-------------+--------+------------------------+-----+
|   opcode    |   rd   |        imm16           |  0  |
+-------------+--------+------------------------+-----+
```

---

### 4. Control Flow

| Mode | Opcode |  Bin   | Hex  | Inputs |      Meaning      |
| :--: | :----: | :----: | :--: | :----: | :---------------: |
|  U   |  JMP   | 011110 | 0x1E | imm26  |   PC = PC + imm   |
|  U   |  CALL  | 011111 | 0x1F | imm26  | ra = PC + 1; jump |
|  U   |  RET   | 100000 | 0x20 |   —    |      PC = ra      |

```
31          26 25                             0
+-------------+--------------------------------+
|   opcode    |            imm26               |
+-------------+--------------------------------+
```

---

### 5. Branching

| Mode | Opcode |  Bin   | Hex  |     Inputs      |  Meaning   |
| :--: | :----: | :----: | :--: | :-------------: | :--------: |
|  U   |  BEQ   | 100001 | 0x21 | rs1, rs2, imm16 | if == jump |
|  U   |  BNE   | 100010 | 0x22 | rs1, rs2, imm16 | if != jump |
|  U   |  BLT   | 100011 | 0x23 | rs1, rs2, imm16 | if < jump  |
|  U   |  BGT   | 100100 | 0x24 | rs1, rs2, imm16 | if > jump  |
|  U   |  BLE   | 100101 | 0x25 | rs1, rs2, imm16 | if <= jump |
|  U   |  BGE   | 100110 | 0x26 | rs1, rs2, imm16 | if >= jump |

```
31          26 25    21 20    16 15                 0
+-------------+--------+--------+--------------------+
|   opcode    |  rs1   |  rs2   |       imm16        |
+-------------+--------+--------+--------------------+
```

---

### 6. System Calls

| Mode | Opcode |  Bin   | Hex  | Inputs |       Meaning       |
| :--: | :----: | :----: | :--: | :----: | :-----------------: |
|  U   | SCALL  | 100111 | 0x27 |   —    |       syscall       |
|  K   |  SRET  | 101000 | 0x28 |   —    | return from syscall |

```
31          26 25                              0
+-------------+--------------------------------+
|   opcode    |            unused              |
+-------------+--------------------------------+
```

---

### 7. System Registers

| Mode | Opcode |  Bin   | Hex  |   Inputs   |        Meaning        |
| :--: | :----: | :----: | :--: | :--------: | :-------------------: |
|  U   |  SYSR  | 101001 | 0x29 | rd, imm16  | read system register  |
|  K   |  SYSW  | 101010 | 0x2A | rs1, imm16 | write system register |

```
31          26 25    21 20    16 15           0
+-------------+--------+--------+--------------+
|   opcode    |  reg   |  id    |   unused     |
+-------------+--------+--------+--------------+
```

System Register IDs (imm16):

0. PSR
1. IVT
2. IMR
3. EPC
4. TID

---

### 8. Atomic (LR/SC)

| Mode | Opcode |  Bin   | Hex  |    Inputs    |      Meaning      |
| :--: | :----: | :----: | :--: | :----------: | :---------------: |
|  U   |   LR   | 101011 | 0x2B |   rd, rs1    |   load-reserved   |
|  U   |   SC   | 101100 | 0x2C | rd, rs1, rs2 | conditional store |

SC result:

- rd = 1 success
- rd = 0 failure

LR:

```
31          26 25    21 20    16 15        0
+-------------+--------+--------+-----------+
|   opcode    |   rd   |  rs1   |  unused   |
+-------------+--------+--------+-----------+
```

SC:

```
31          26 25    21 20    16 15    11 10        0
+-------------+--------+--------+--------+-----------+
|   opcode    |   rd   |  rs1   |  rs2   |  unused   |
+-------------+--------+--------+--------+-----------+
```

---

### 9. Misc

| Mode | Opcode |  Bin   | Hex  | Inputs | Meaning |
| :--: | :----: | :----: | :--: | :----: | :-----: |
|  U   |  NOP   | 101101 | 0x2D |   —    |  no-op  |
|  U   |  HALT  | 101110 | 0x2E |   —    |  stop   |

```
31          26 25                              0
+-------------+--------------------------------+
|   opcode    |            unused              |
+-------------+--------------------------------+
```
