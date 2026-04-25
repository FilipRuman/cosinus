# Instruction Parsing Spec

## PROGRAM STRUCTURE

A program is a sequence of commands:

Command := Instruction | Macro | Label | RawData

All ISA instructions are supported and parsed via the opcode dispatcher.

---

## LABELS

Syntax: label: :label

Recommended canonical form: label:

Semantics:

- Labels mark an instruction address (index in instruction stream)
- Labels are not emitted into the final program
- Labels can be referenced from immediates

Resolution:

- Labels are resolved to instruction indices
- Branches use PC-relative offsets: offset = target - current_pc - 1
- Jumps may use absolute or relative addressing depending on instruction

---

## REGISTERS

Syntax: r0 ... r31

Rules:

- Parsed as u8
- Range: 0–31 inclusive

---

## IMMEDIATES

Syntax: 123 decimal -123 signed decimal 0xFF hex 0b1010 binary label symbolic
reference

Semantic model:

- All immediates are SIGNED in interpretation
- Stored as fixed-width bitfields (no sign metadata in encoding layer)

Resolution rules:

1. Parse as signed integer (i64)
2. If value fits signed immediate range(2^15/2^25) → accept
3. Else if token is identifier → treat as label
4. Else → error

Immediate widths:

- imm16: i16 semantic range
- imm26: i26 semantic range

Encoding rule:

- Final encoding always masks to width (no runtime semantics)

---

## RAW DATA

Syntax: .data v1 v2 v3 ...

Semantics:

- Emits raw 32-bit signed integers
- Stored directly in output stream
- Not executed as instructions
- Used for constants, tables, and embedded data

---

## MACROS

Macros expand into one or more instructions before encoding.

Supported macros:

- SET rd imm16 Expands to: load immediate into register (via instruction
  sequence)

- SET32 rd imm32 Expands to: 32-bit immediate construction using upper/lower
  split LUI + OR

---

## ASSEMBLY PIPELINE

1. Lexing / line parsing
2. Convert lines → Command stream
3. Expand macros → Instruction stream
4. Build label table (or resolve on-demand)
5. Resolve immediates:
   - integers
   - labels → PC-relative or absolute values
6. Encode instructions → u32 machine words

---

## INVARIANTS

- Registers are always u8
- Immediates are semantically signed
- Encoding is purely bit-level (masking/truncation only)
- Labels never appear in final binary
- Program order defines instruction addresses
- All ISA instructions are supported via opcode dispatcher
