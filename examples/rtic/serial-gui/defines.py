class APP:
    RGB = 0xA0
    LED = 0xB0
    LCD = 0xC0


class RGB_CMD:
    SET_COLORS = 0x00
    SET_RED = 0x01
    SET_GREEN = 0x02
    SET_BLUE = 0x03


class LED_CMD:
    SET_FREQ = 0x01
    LED_OFF = 0x02
    LED_ON = 0x03


class LCD_CMD:
    SEND_CMD = 0x01
    SEND_DATA = 0x02
