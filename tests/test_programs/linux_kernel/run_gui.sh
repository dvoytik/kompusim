set -e

# cargo build --release
cargo build

# --breakpoint 0x0000000080000038 \
./target/debug/kompusim-gui \
  exec \
  --ram 256M \
  --load-addr  0x0000000080000000 \
  --breakpoint 0x000000008002051e \
  --bin tests/test_programs/linux_kernel/fw_payload.bin_debug1_dtb \
  $@

# --bin tests/test_programs/linux_kernel/fw_payload.bin \
