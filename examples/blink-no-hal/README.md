# Blink without HAL

This example shows how to access a register mapped peripheral using the crate `volatile-register`.

Like in C, a `struct` is used to map the peripheral registers in the memory. The `volatile-register` crate grants **read-write** and **read-only** accesses to these memory regions.

The GPIOA peripheral is accessed in order to control the NUCLEO-F103RB board LED on pin 5.

## Building
```bash
cargo build --release
```

## Flashing
```bash
cargo flash --chip stm32f103rb --release
```