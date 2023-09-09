set -e

# cargo build --release
cargo build

# optional switches:
# --trace - print cpu/devices state
./target/debug/kompusim-gui \
  exec \
  --load-addr 0x0000000080000000 \
  --breakpoint 0x0000000080000014 \
  --bin tests/test_programs/uart_hello_world/out/uart_hello_world.bin \
  $@
