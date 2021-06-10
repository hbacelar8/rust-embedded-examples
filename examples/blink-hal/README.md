# Blink with HAL

Blinks the NUCLEO-F103RB board LED connected to pin 5 on Port A using the `stm32f1xx-hal` crate.

## Building
```bash
cargo build --release
```

## Flashing
```bash
cargo flash --chip stm32f103rb --release
```