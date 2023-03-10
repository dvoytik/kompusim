.align 2
.equ UART0_BASE,         0x10010000 # UART 0
.equ UART_REG_TXFIFO,   0

.section .text
.globl _start

_start:
    csrr  t0, mhartid   # read hardware thread id (i.e., CPU ID)
    bnez  t0, halt      # halt other CPUs except first (with CPU ID == 0)

    la    a0, msg       # load address of "msg" to a0 argument register
    jal   print         # jump to "print" subroutine,
                        # return address is stored in ra regster

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

# 00000000  f3 22 40 f1 63 98 02 00  17 05 00 00 13 05 45 03  |."@.c.........E.|
# 00000010  ef 00 80 00 6f 00 00 00  b7 02 01 10 03 43 05 00  |....o........C..|
# 00000020  63 0c 03 00 83 a3 02 00  e3 ce 03 fe 23 a0 62 00  |c...........#.b.|
# 00000030  13 05 15 00 6f f0 9f fe  67 80 00 00 48 65 6c 6c  |....o...g...Hell|
# 00000040  6f 2c 20 57 6f 72 6c 64  21 0a 00                 |o, World!..|
# 0000004b
