# Peripheral Access

This example shows how to access a register mapped peripheral using the crate `volatile-register`.

Like in C, a `struct` is used to map the peripheral registers in the memory. The `volatile-register` crate grants **read-write** and **read-only** accesses to these memory regions.

## Building
```bash
cargo build --release
```

## Flashing
```bash
cargo flash --chip stm32f103rb --release
```