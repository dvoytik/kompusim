#!/usr/bin/env bash

P=all_instructions

CC=riscv64-unknown-elf-gcc
READELF=riscv64-unknown-elf-readelf
OBJDUMP=riscv64-unknown-elf-objdump
OBJCOPY=

mkdir -p out

$CC \
    -Os \
    -march=rv64g -mabi=lp64 -static -mcmodel=medany \
    -fvisibility=hidden -nostdlib -nostartfiles \
    -T$P.ld -I. \
    $P.s -o out/$P

#riscv64-unknown-elf-objcopy --info
riscv64-unknown-elf-objcopy -O binary out/$P out/$P.bin

$READELF -a out/$P > out/$P.readelf
$OBJDUMP -a -f -h -p -r -t -d -s out/$P > out/$P.objdump
$OBJDUMP -a -f -h -p -r -t -d -s -M no-aliases out/$P > out/$P.objdump_no_aliases
hexdump -C out/$P.bin > out/$P.bin_hexdump
# -d - disassemble
# -F - disaplay file offset of the region of data
# -f
# -p - Print information that is specific to the object file format.
# -r - Print the relocation entries of the file.
# -s - Display the full contents of any sections requested.  By default all non-empty sections are displayed.
# -M no-aliases   - Disassemble only into canonical instructions.
# --disassembler-color=extended-color
# --visualize-jumps=extended-color
