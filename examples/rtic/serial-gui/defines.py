class APP:
    LCD = 0xA0
    LED = 0xB0
    RGB = 0xC0


class LCD_CMD:
    SEND_CMD = 0x01
    SEND_DATA = 0x02


class LED_CMD:
    SET_FREQ = 0x01
    LED_OFF = 0x02
    LED_ON = 0x03


class RGB_CMD:
    OFF = 0x01
    FULL = 0x01
    SET_BLUE = 0x02
    SET_GREEN = 0x03
    SET_RED = 0x04
