# Serial PWM Interrupt

This project handles a serial communication protocol through serial interruptions in order to control a RGB LED and a buzzer using PWM.

## Wiring
| Pin | Function  |
|-----|-----------|
| PB6 | LED Red   |
| PB7 | LED Green |
| PB8 | LED Blue  |
| PB9 | Buzzer    |

## Serial Protocol
| App  | Cmd  | Len (bytes) | Payload          | Function                            |
|------|------|-------------|------------------|-------------------------------------|
| 0xA0 | 0x00 | 0x03        | red, green, blue | Configures the 3 LED RGB colors     |
| 0xA0 | 0x01 | 0x01        | red              | Configure the red color intensity   |
| 0xA0 | 0x02 | 0x01        | green            | Configure the green color intensity |
| 0xA0 | 0x03 | 0x01        | blue             | Configure the blue color intensity  |
| 0xB0 | 0x01 | 0x00        | --               | Plays the buzzer                    |
| 0xB0 | 0x02 | 0x00        | --               | Stops the buzzer                    |

## Building
```bash
cargo build --release
```

## Flashing
```bash
cargo flash --chip stm32f103rb --release
```