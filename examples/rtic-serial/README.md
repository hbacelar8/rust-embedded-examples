## RTIC UART Communication
This example show how to use the Real-Time Interrupt-driven Concurrency (RTIC) framework to handle UART2 interruptions and control the board LED.

## Building
```bash
cargo build --release
```

## Flashing
```bash
cargo flash --chip stm32f103rb --release
```