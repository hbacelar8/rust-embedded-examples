// src/lcd.rs
// LCD module

use cortex_m::asm::nop;
use embedded_hal::digital::v2::OutputPin;
use panic_halt as _;

use hal::{
    delay::Delay,
    gpio::{
        gpioa::{PA0, PA1},
        gpioc::{PC0, PC1, PC2, PC3},
        Output, PushPull,
    },
    prelude::*,
};
use stm32f1xx_hal as hal;

pub struct LCD {
    rs: PA0<Output<PushPull>>,
    en: PA1<Output<PushPull>>,
    d4: PC0<Output<PushPull>>,
    d5: PC1<Output<PushPull>>,
    d6: PC2<Output<PushPull>>,
    d7: PC3<Output<PushPull>>,
    delay: Delay,
}

impl LCD {
    pub fn new(
        rs: PA0<Output<PushPull>>,
        en: PA1<Output<PushPull>>,
        d4: PC0<Output<PushPull>>,
        d5: PC1<Output<PushPull>>,
        d6: PC2<Output<PushPull>>,
        d7: PC3<Output<PushPull>>,
        delay: Delay,
    ) -> LCD {
        LCD {
            rs,
            en,
            d4,
            d5,
            d6,
            d7,
            delay,
        }
    }

    /* -------------------- Private Functions -------------------- */

    /// Pulse enable
    fn _pulse_enable(&mut self) {
        self.en.set_high().ok();
        nop();
        self.en.set_low().ok();
    }

    /// Send half a byte to the LCD
    ///
    /// # Arguments
    ///
    /// * `nibble` - Nibble to send
    fn _send_nibble(&mut self, nibble: u8) {
        if ((nibble >> 3) & 0x01) == 0x01 {
            self.d7.set_high().ok();
        } else {
            self.d7.set_low().ok();
        }

        if ((nibble >> 2) & 0x01) == 0x01 {
            self.d6.set_high().ok();
        } else {
            self.d6.set_low().ok();
        }

        if ((nibble >> 1) & 0x01) == 0x01 {
            self.d5.set_high().ok();
        } else {
            self.d5.set_low().ok();
        }

        if ((nibble >> 0) & 0x01) == 0x01 {
            self.d4.set_high().ok();
        } else {
            self.d4.set_low().ok();
        }

        self._pulse_enable();
    }

    /* -------------------- Public Functions -------------------- */

    /// Initialize the LCD
    pub fn init(&mut self) {
        // Power on delay
        self.delay.delay_ms(100_u16);

        // Send command
        self.rs.set_low().ok();

        // First nibble 0b0011
        self._send_nibble(0x03);
        self.delay.delay_us(4100_u16);

        // Second nibble 0b0011
        self._pulse_enable();
        self.delay.delay_us(100_u16);

        // Third nibble 0b0011
        self._pulse_enable();
        self.delay.delay_us(100_u16);

        // Configure LCD in 4-bit mode
        self._send_nibble(0x02);
        self.delay.delay_us(100_u16);

        // Function set to configure the interface, number of lines and the font
        self.send_cmd(0x28);
        self.delay.delay_us(53_u16);

        // Display off
        self.send_cmd(0x08);
        self.delay.delay_us(53_u16);

        // Clear display (demands a longer delay)
        self.send_cmd(0x01);
        self.delay.delay_us(3000_u16);

        // Entry mode set
        self.send_cmd(0x06);
        self.delay.delay_us(53_u16);

        // Display on
        self.send_cmd(0x0C);
        self.delay.delay_us(53_u16);
    }

    /// Send command to the LCD
    ///
    /// # Arguments
    ///
    /// * `cmd` - Command to send
    pub fn send_cmd(&mut self, cmd: u8) {
        self.rs.set_low().ok();

        let higher_nibble = (cmd >> 4) & 0x0F;
        let lower_nibble = (cmd >> 0) & 0x0F;

        self._send_nibble(higher_nibble);
        self._send_nibble(lower_nibble);
    }

    /// Send data to the LCD
    ///
    /// # Arguments
    ///
    /// * `data` - Byte to send
    #[allow(dead_code)]
    pub fn send_data(&mut self, data: u8) {
        self.rs.set_high().ok();

        let higher_nibble = (data >> 4) & 0x0F;
        let lower_nibble = (data >> 0) & 0x0F;

        self._send_nibble(higher_nibble);
        self._send_nibble(lower_nibble);

        self.delay.delay_us(40_u16);
    }

    /// Send a string to the LCD
    ///
    /// # Arguments
    ///
    /// * `string` - String to send
    #[allow(dead_code)]
    pub fn send_string(&mut self, string: &str) {
        for byte in string.chars() {
            self.send_data(byte as u8);
        }
    }
}
