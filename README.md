<p align="center">
    <img src="./logo.png" height="128px">
</p>

# Rust Embedded Examples
A collection of embedded Rust examples using the `stm32f1xx-hal` crate and the Real-Time Interrupt-driven Concurrency (RTIC) framework.

The examples can be found in the `examples` folder. Each example is a project itself, meaning they can be run separately.

## Examples List
- `blink-no-hal`: A blink project showing how to access a peripheral without using the HAL crate;
- `blink-hal`: A blink project using the `stm32f1xx-hal` crate;
- `serial-echo`: A project that implements a serial echo on USART2;
- `serial-pwm-polling`: This project handles a simple serial communication protocol by polling the USART2 in order to control a RGB LED and a buzzer using PWM;
- `serial-pwm-interrupt`: This project handles a simple serial communication protocol through serial interruptions in order to control a RGB LED and a buzzer using PWM;
- `rtic-serial`: A project using the RTIC framework in order to control an LED through USART2 peripheral;
- `rtic`: A project using the RTIC framework where a serial communication protocol is implemented in order to control an LED, a LED RGB and a LCD display.

## References

[The Embedded Rust Book](https://docs.rust-embedded.org/book/intro/index.html) \
[Real-Time Interrupt-driven Concurrency (RTIC)](https://rtic.rs/0.5/book/en/preface.html) \
[Rust and STM32: A Quick Start Guide](https://bacelarhenrique.me/2021/02/21/rust-and-stm32-a-quick-start-guide.html)

### Related Projects

[stm32f1xx_hal](https://github.com/stm32-rs/stm32f1xx-hal) \
[cortex-m-rtic](https://github.com/rtic-rs/cortex-m-rtic) \
[rust-lcd-display](https://github.com/bacelarhenrique/rust-lcd-display)