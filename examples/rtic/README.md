# RTIC Complete Example
This is a more complete example of the Real-Time Interrupt-driven Concurrency (RTIC) framework.

It implements a UART protocol for receiving commands from the serial communication and thus controlling an LCD display, a RGB LED and a the board LED blink frequency.

## Commands
The commands are separated by peripheral. The `App` byte distinguishes between LCD, LED and RGB commands. The `Cmd` byte defines the different commands of the same category. Finally, the `Length` byte indicates the number of bytes to arrive as payload.

### LCD Commands
| Command             | App  | Cmd  | Length | Payload     |
|---------------------|------|------|--------|-------------|
| Send Command to LCD | 0xA0 | 0x01 | 0x01   | LCD Command |
| Send Data to LCD    | 0xA0 | 0x02 | 0x0X   | Data        |

### LED Commands
| Command           | App  | Cmd  | Length | Payload |
|-------------------|------|------|--------|---------|
| Set LED Frequency | 0xB0 | 0x01 | 0x01   | Freq    |

### RGB Commands
| Command         | App  | Cmd  | Length | Payload   |
|-----------------|------|------|--------|-----------|
| Turn RGB Off    | 0xC0 | 0x01 | 0x01   | 0x01      |
| Set RGB Full    | 0xC0 | 0x01 | 0x01   | 0x02      |
| Set Blue Value  | 0xC0 | 0x02 | 0x01   | blue_val  |
| Set Green Value | 0xC0 | 0x03 | 0x01   | green_val |
| Set Red Value   | 0xC0 | 0x04 | 0x01   | blue_val  |

## Building
```bash
cargo build --release
```

## Flashing
```bash
cargo flash --chip stm32f103rb --release
```
