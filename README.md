# Kompusim

Kompusim is a simple CPU / SoC ISA level simulator with focus on education and debugging functionality. Currently, it supports only `RISC-V` ISA.  
It is still heavily under construction.

## How to run

It is expected that [Rust tool chain is installed](https://www.rust-lang.org/tools/install).

Demo with UART print:
```
./tests/uart_hello_world/run.sh
```

Compile and run:
```
cargo run -- -h
```
