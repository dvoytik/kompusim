.align 2
.equ UART0_BASE,         0x10010000 # UART 0
.equ UART_REG_TXFIFO,   0

.section .text
.globl _start

_start:
    c.addi x10, -1
    c.addi x10, 31
    c.addi x10, -31
    lb x6, 2047(x10)
    lb x6, -1(x10)
    c.sd x14, 248(x15)
    c.lw x15, 0x5c(x15)
    ld x15, 255(x15)
    auipc x10, 0xfffff
    auipc x10, 0
    lui x1, 0xfffff
    slliw x15, x15, 0
    slliw x15, x15, 1
    slliw x15, x15, 7
    slliw x15, x15, 14
    slliw x15, x15, 31
    slliw x15, x15, 4
    c.addiw	x15, 31
    li x4, 0xbadc0ffe
    la x3, beef_addr # x6 <= 1
    # amoswap.w.aq rd, rs2, rs1 # rd <= mem[rs1]; mem[rs1] <= rs2
    amoswap.w.aq x1, x4, (x3)
    # amoswap.w.aq  x6, x5, (x10)
    amoswap.w.aq x2, x1, (x3)
    # amoswap.w.aq  x6, x5, (x10)
    amoadd.w.aq x2, x1, (x0)

    sd x6, 0x0(x5)
    ld x6, 0x0(x5)

    lr.w x1, (x0)
    # lr.w.aq x1, (x0)
    c.nop

    li x4, 0xbadc0ffe
    csrrw x1, mtvec, x4
    csrrw x1, mtvec, x4

    addi  x1, x0, 1
    add   x1, x1, x1
    sub   x1, x1, x1
    # 16-bit compressed
    c.li  x2, -1
    csrr  t0, mhartid   # read hardware thread id (i.e., CPU ID)
    bnez  t0, halt      # halt other CPUs except first (with CPU ID == 0)

    la    a0, msg       # load address of "msg" to a0 argument register
    jal   print         # jump to "print" subroutine,
                        # return address is stored in ra regster
    c.lui x1, 31

halt:
    j     halt          # enter the infinite loop

print:  # "print" subroutine writes null-terminated string
        # to UART (serial communication port)
        # input: a0 register specifies the starting address
        # of a null-terminated string
        # clobbers: t0, t1, t2 temporary registers

    li    t0, UART0_BASE # t0 = UART0_BASE
1:
    lbu   t1, 0(a0)      # t1 = load unsigned byte
                         # from memory address specified by a0 register
    beqz  t1, 3f         # break the loop, if loaded byte was null
                         # wait until UART is ready
2:
    lw    t2, UART_REG_TXFIFO(t0) # t2 = uart[UART_REG_TXFIFO]
    bltz  t2, 2b                  # bit 31 == 1 --> FIFO full
                                  # is ready for transmission
    sw    t1, UART_REG_TXFIFO(t0) # send byte, uart[UART_REG_TXFIFO] = t1

    addi  a0, a0, 1               # increment a0 address by 1 byte
    j     1b
3:
    ret                           # jump tp address stored in ra

.section .rodata
msg:
    .string "Hello, World!\n"
beef_addr:
    .long 0xdeadbeef
