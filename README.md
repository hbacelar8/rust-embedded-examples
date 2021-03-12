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
- `serial-pwm`: This project handles a simple serial communication protocol for receiving commands in order to control a RGB LED using PWM and a buzzer.
- `rtic-serial`: Simple interrupt-driven serial communication using the RTIC framework.
- `rtic`: This is a more complete example of the RTIC implementing a UART protocol for receiving commands from the serial communication and thus controlling an LCD display, a RGB LED and a the board LCD blink delay.

## References

[The Embedded Rust Book](https://docs.rust-embedded.org/book/intro/index.html) \
[Real-Time Interrupt-driven Concurrency](https://rtic.rs/0.5/book/en/preface.html) \
[Rust on STM32: Getting started](https://jonathanklimt.de/electronics/programming/embedded-rust/rust-on-stm32-2/)

### Related Projects

[stm32f1xx_hal](https://github.com/stm32-rs/stm32f1xx-hal) \
[cortex-m-rtic](https://github.com/rtic-rs/cortex-m-rtic)

## Author

Henrique Bacelar \
[LinkedIn](https://www.linkedin.com/in/bacelarhenrique/)