# Serial and PWM Communication Protocol

This project handles a simple serial communication protocol for receiving commands in order to control a RGB LED using PWM and a buzzer.

The project uses the `stm32f1xx-hal` crate.

## Building
```bash
cargo build --release
```

## Flashing
```bash
cargo flash --chip stm32f103rb --release
```

## Commands
- `crXXXgXXXbXXX`: Configures the RGB LED color. The X's must be a three digit number from 0 to 254.
- `bp`: Plays the buzzer.
- `bs`: Stops the buzzer.