RISC-V ISA terminology
----------------------
EEI - Execution Environment Interface (E.g., Linux ABI, RISC-V SBI)

Source code terminology and abbreviations
-----------------------------------------
aq - acquisition
rl - relinquishment
rvc - RISC-V Compressed (Instruction set) - used to denote 16-bit compressed RISC-V instructions
halfword - 16 bits
word - 32 bits
doubleword - 64 bits
quadword - 128 bits


Registers
---------
```
Reg    ABI  Preserverd?   Description
-------------------------------------------------------------------
x0     zero      -        read as zero, write is ignored
x1     ra        n        ret addr - used to hold subroutine return address (link register) in standard SW convention
x2     sp        y        used to hold stack pointer in standard SW convention
x3     gp        -        global pointer
x4     tp        -        thread pointer
x5     t0        n        temp reg 0, alternate link register
x6     t1        n        temp reg 1
x7     t2        n        temp reg 2
x8     s0        y        saved register 0 or frame pointer
x9     s1        y        saved register 1
x10    a0        n        return value or function argument 0
x11    a1        n        return value or function argument 1
x12    a2        n        function argument 2
x13    a3        n        function argument 3
x14    a4        n        function argument 4
x15    a5        n        function argument 5
x16    a6        n        function argument 6
x17    a7        n        function argument 7
x18    s2        y        saved register 2
x19    s3        y        saved register 3
x20    s4        y        saved register 4
x21    s5        y        saved register 5
x22    s6        y        saved register 6
x23    s7        y        saved register 7
x24    s8        y        saved register 8
x25    s9        y        saved register 9
x26    s10       y        saved register 10
x27    s11       y        saved register 11
x28    t3        n        temp reg 2
x29    t4        n        temp reg 2
x30    t5        n        temp reg 2
x31    t6        n        temp reg 6
pc     ---       -        Program counter
```
