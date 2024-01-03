#!/usr/bin/env bash

set -e
# set -x

P=all_instructions

PREFIX=riscv64-unknown-elf-
CC=${PREFIX}gcc
READELF=${PREFIX}readelf
OBJDUMP=${PREFIX}objdump
OBJCOPY=${PREFIX}objcopy

cd $(dirname $(realpath -s $0))

mkdir -p ./out

# -Os - optimize for size
# -mno-shorten-memrefs - do not attempt to make more use of compressed load/store instructions
${CC} \
    -march=rv64gc -mabi=lp64 -static -mcmodel=medany \
    -fvisibility=hidden -nostdlib -nostartfiles \
    -T${P}.ld \
    -I. \
    ${P}.s \
    -o ./out/${P}.elf

#${OBJCOPY} --info
${OBJCOPY} -O binary ./out/${P}.elf ./out/${P}.bin

# $READELF -a out/${P} > out/${P}.readelf
# $OBJDUMP -a -f -h -p -r -t -d -s out/${P} > out/${P}.objdump
$OBJDUMP -a -f -h -p -r -t -d -s -M no-aliases out/${P}.elf > out/$P.objdump_no_aliases

rm out/${P}.elf
# hexdump -C out/$P.bin > out/$P.bin_hexdump
# -d - disassemble
# -F - disaplay file offset of the region of data
# -f
# -p - Print information that is specific to the object file format.
# -r - Print the relocation entries of the file.
# -s - Display the full contents of any sections requested.  By default all non-empty sections are displayed.
# -M no-aliases   - Disassemble only into canonical instructions.
# --disassembler-color=extended-color
# --visualize-jumps=extended-color
