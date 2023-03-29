cargo build --release

./target/release/kompusim load -a 0x0000000080000000 \
  --bin tests/uart_hello_world/out/uart_hello_world.bin
