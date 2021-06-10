// src/main.rs

/*
    APP     CMD     Length (bytes)      Payload
    0xA0    0x00    0x03                red, green, blue
    0xA0    0x01    0x01                red
    0xA0    0x02    0x01                green
    0xA0    0x03    0x01                blue

    0xB0    0x01    0x00                --
    0xB0    0x02    0x00                --
*/

// std and main are not available for bare metal software
#![no_std]
#![no_main]

use panic_halt as _;

use cortex_m_rt::entry;
use heapless::{consts, Vec};
use nb::block;
use stm32f1xx_hal::{
    gpio::{
        gpiob::{PB6, PB7, PB8, PB9},
        Alternate, PushPull,
    },
    pac::{self, TIM4},
    prelude::*,
    pwm::{Channel, Pwm, C1, C2, C3, C4},
    serial::{Config, Serial, StopBits},
    time::U32Ext,
    timer::{Tim4NoRemap, Timer},
};

#[entry]
fn main() -> ! {
    // Get access to device peripherals
    let dp = pac::Peripherals::take().unwrap();

    // Get access to RCC, AFIO, FLASH, GPIOA and GPIOB
    let mut rcc = dp.RCC.constrain();
    let mut flash = dp.FLASH.constrain();
    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);
    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);
    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);

    // Freeze clocks
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    // Set up UART2 pins
    let tx = gpioa.pa2.into_alternate_push_pull(&mut gpioa.crl);
    let rx = gpioa.pa3;

    // Set up TIM4 PWM pins
    let c1 = gpiob.pb6.into_alternate_push_pull(&mut gpiob.crl);
    let c2 = gpiob.pb7.into_alternate_push_pull(&mut gpiob.crl);
    let c3 = gpiob.pb8.into_alternate_push_pull(&mut gpiob.crh);
    let c4 = gpiob.pb9.into_alternate_push_pull(&mut gpiob.crh);
    let pins = (c1, c2, c3, c4);

    // Get PWM instance
    let mut pwm = Timer::tim4(dp.TIM4, &clocks, &mut rcc.apb1).pwm::<Tim4NoRemap, _, _, _>(
        pins,
        &mut afio.mapr,
        1.khz(),
    );

    // Enable clock on each of the channels
    pwm.enable(Channel::C1);
    pwm.enable(Channel::C2);
    pwm.enable(Channel::C3);
    pwm.enable(Channel::C4);

    // Get UART2 instance
    let mut serial = Serial::usart2(
        dp.USART2,
        (tx, rx),
        &mut afio.mapr,
        Config::default()
            .baudrate(9600.bps())
            .stopbits(StopBits::STOP1)
            .parity_none(),
        clocks,
        &mut rcc.apb1,
    );

    // A buffer with 8 bytes of capacity
    let mut buffer: Vec<u8, consts::U8> = Vec::new();

    let mut app: u8 = 0;
    let mut cmd: u8 = 0;
    let mut len: u8 = 0;
    let mut counter: u8 = 0;

    loop {
        // Poll RX
        let byte_received = block!(serial.read()).unwrap();

        match counter {
            0 => {
                if byte_received == 0xA0 || byte_received == 0xB0 {
                    app = byte_received;
                    counter += 1;
                }
            }
            1 => {
                cmd = byte_received;
                counter += 1;
            }
            2 => {
                len = byte_received;
                counter += 1;
            }
            _ => {
                buffer.push(byte_received).ok();
                counter += 1;

                if counter == len + 3 {
                    counter = 0;

                    msg_handler(&mut pwm, &mut buffer, &app, &cmd);
                }
            }
        }
    }
}

/// Message handler function
fn msg_handler(
    pwm: &mut Pwm<
        TIM4,
        Tim4NoRemap,
        (C1, C2, C3, C4),
        (
            PB6<Alternate<PushPull>>,
            PB7<Alternate<PushPull>>,
            PB8<Alternate<PushPull>>,
            PB9<Alternate<PushPull>>,
        ),
    >,
    buffer: &mut Vec<u8, consts::U8>,
    app: &u8,
    cmd: &u8,
) {
    match app {
        0xA0 => {
            // Get max duty cycle and divide it by steps of 255 for the color range
            let step = pwm.get_max_duty() / 255;

            match cmd {
                0x00 => {
                    let red = buffer[0];
                    let green = buffer[1];
                    let blue = buffer[2];

                    pwm.set_duty(Channel::C1, step * red as u16);
                    pwm.set_duty(Channel::C2, step * green as u16);
                    pwm.set_duty(Channel::C3, step * blue as u16);
                }
                0x01 => {
                    let red = buffer[0];

                    pwm.set_duty(Channel::C1, step * red as u16);
                }
                0x02 => {
                    let green = buffer[0];

                    pwm.set_duty(Channel::C2, step * green as u16);
                }
                0x03 => {
                    let blue = buffer[0];

                    pwm.set_duty(Channel::C3, step * blue as u16);
                }
                _ => {}
            }
        }
        0xB0 => match cmd {
            0x01 => {
                let max = pwm.get_max_duty();

                pwm.set_duty(Channel::C4, max / 2);
            }
            0x02 => pwm.set_duty(Channel::C4, 0),
            _ => {}
        },
        _ => {}
    }

    buffer.clear();
}
