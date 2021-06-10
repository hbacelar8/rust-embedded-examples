# A Simple Blink Project

Blinks the NUCLEO-F103RB board LED connected to pin 5 on Port A.

## Building
```bash
cargo build --release
```

## Flashing
```bash
cargo flash --chip stm32f103rb --release
```