## Serial Blocking
This project is a simple echo from UART2 using the `stm32f1xx-hal` crate.

## Building
```bash
cargo build --release
```

## Flashing
```bash
cargo flash --chip stm32f103rb --release
```