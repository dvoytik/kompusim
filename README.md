# Kompusim

Kompusim is a work-in-progress simple CPU / SoC ISA level simulator with focus on education and debugging functionality. Currently, it supports only `RISC-V` ISA.

There are GUI and TUI (terminal user interface) versions of the simulator.
GUI is currently the primary focus and TUI is lagging behind in debugging and learning capabilities.

See [screenshots](https://github.com/dvoytik/kompusim/wiki/Screenshots).

## How to build and run the GUI simulator

It is expected that [Rust tool chain is installed](https://www.rust-lang.org/tools/install).
```
cargo run -p kompusim-gui
```

## How to build and run TUI simulator

It is expected that [Rust tool chain is installed](https://www.rust-lang.org/tools/install).

A demo of running the bare metal program that prints "Hello, World!" to the UART in TUI simulator:
```
./tests/test_programs/uart_hello_world/run.sh
```

Run the demo program in the interactive mode:
```
./tests/test_programs/uart_hello_world/run.sh -i
```
Press `s` repeatedly to step over instructions.  
Press `h` to see the full list of commands.
