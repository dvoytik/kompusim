set -e

cargo build --release

./target/release/kompusim exec \
  --load-addr 0x0000000080000000 \
  --breakpoint 0x0000000080000014 \
  --bin tests/uart_hello_world/out/uart_hello_world.bin
