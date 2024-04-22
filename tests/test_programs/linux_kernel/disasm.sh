#!/usr/bin/env bash
# BIN=fw_payload.bin
# BIN=fw_payload.bin_debug1
BIN=fw_payload.bin_debug1_dtb
READELF=riscv64-unknown-elf-readelf
OBJDUMP=riscv64-unknown-elf-objdump

# --adjust-vma=0x800000
# $OBJDUMP -b binary -a -f -h -p -r -t -d -s -D $P > $P.objdump
# $OBJDUMP -b binary -a -f -h -p -r -t -d -s -M no-aliases -D $P > $P.objdump_no_aliases
# $OBJDUMP -m riscv -b binary -D $P > $P.objdump
$OBJDUMP \
  --source \
  --architecture=riscv:rv64 \
  -b binary \
  -M no-aliases,numeric \
  --disassemble-all ${BIN} \
  > ${BIN}.objdump_no_aliases
# --show-all-symbols - Not supported
# -d - disassemble
# -F - disaplay file offset of the region of data
# -f
# -p - Print information that is specific to the object file format.
# -r - Print the relocation entries of the file.
# -s - Display the full contents of any sections requested.  By default all non-empty sections are displayed.
# -M no-aliases   - Disassemble only into canonical instructions.
# --disassembler-color=extended-color
# --visualize-jumps=extended-color
# --target=elf64-little - doesn't work
