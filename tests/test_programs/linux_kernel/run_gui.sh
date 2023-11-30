set -e

# cargo build --release
cargo build

./target/debug/kompusim-gui \
  exec \
  --ram 256M \
  --load-addr 0x0000000080000000 \
  --breakpoint 0x0000000080000014 \
  --bin tests/test_programs/linux_kernel/fw_payload.bin \
  $@
