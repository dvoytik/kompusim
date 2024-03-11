#!/usr/bin/env bash
P=fw_payload.bin
READELF=riscv64-unknown-elf-readelf
OBJDUMP=riscv64-unknown-elf-objdump

# --adjust-vma=0x800000
# $OBJDUMP -b binary -a -f -h -p -r -t -d -s -D $P > $P.objdump
# $OBJDUMP -b binary -a -f -h -p -r -t -d -s -M no-aliases -D $P > $P.objdump_no_aliases
$OBJDUMP -m riscv -b binary -D $P > $P.objdump
$OBJDUMP -m riscv -b binary -M no-aliases -D $P > $P.objdump_no_aliases 
# -d - disassemble
# -F - disaplay file offset of the region of data
# -f
# -p - Print information that is specific to the object file format.
# -r - Print the relocation entries of the file.
# -s - Display the full contents of any sections requested.  By default all non-empty sections are displayed.
# -M no-aliases   - Disassemble only into canonical instructions.
# --disassembler-color=extended-color
# --visualize-jumps=extended-color
