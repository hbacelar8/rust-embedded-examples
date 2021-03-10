<p align="center">
    <img src="./logo.png" height="128px">
</p>

# Rust Embedded Examples
A collection of embedded Rust examples using the `stm32f1xx-hal` crate and the Real-Time Interrupt-driven Concurrency (RTIC) framework.

The examples can be found in the `examples` folder. Each example is a project itself, meaning they can be runned separately.

## Examples List
- `peripheral-access`: A project showing how to access a peripheral without a HAL library.
- `blink`: A simple blink project using the `stm32f1xx-hal` crate.
- `serial`: A simple project using the UART2 to echo bytes with the `stm32f1xx-hal` crate.
- `rtic-serial`: Simple interrupt-driven serial communication using the RTIC framework.