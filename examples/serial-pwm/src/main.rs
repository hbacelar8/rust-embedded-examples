// src/main.rs

/*
In this example, UART2 is polled and according to the received bytes
some actions are made. The commands are shown below.

crXXXgXXXbXXX: Configures the RGB LED color. The X's must be a three digit number from 0 to 254.
bp: Plays the buzzer
bs: Stops the buzzer
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

    // A buffer with 16 bytes of capacity
    let mut buffer: Vec<u8, consts::U16> = Vec::new();

    loop {
        let byte_received = block!(serial.read()).unwrap();

        if byte_received == b'c' {
            let mut byte_counter: u8 = 0;
            buffer.clear();

            while byte_counter < 12 {
                let byte_received = block!(serial.read()).unwrap();

                buffer.push(byte_received).ok();
                byte_counter += 1;
            }

            led_handler(&mut pwm, &buffer);
            buffer.clear();
        }

        if byte_received == b'b' {
            let byte_received = block!(serial.read()).unwrap();

            if byte_received == b'p' {
                buzzer_handler(&mut pwm, true);
            } else if byte_received == b's' {
                buzzer_handler(&mut pwm, false);
            }
        }
    }
}

fn led_handler(
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
    buffer: &Vec<u8, consts::U16>,
) {
    // Get max duty cycle and divide it by steps of 255 for the color range
    let step = pwm.get_max_duty() / 255;

    let r = (((buffer[1] - 48) * 100) + ((buffer[2] - 48) * 10) + ((buffer[3] - 48) * 1)) & 0xFF;
    let g = (((buffer[5] - 48) * 100) + ((buffer[6] - 48) * 10) + ((buffer[7] - 48) * 1)) & 0xFF;
    let b = (((buffer[9] - 48) * 100) + ((buffer[10] - 48) * 10) + ((buffer[11] - 48) * 1)) & 0xFF;

    pwm.set_duty(Channel::C1, step * (r as u16));
    pwm.set_duty(Channel::C2, step * (g as u16));
    pwm.set_duty(Channel::C3, step * (b as u16));
}

fn buzzer_handler(
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
    sound: bool,
) {
    let max = pwm.get_max_duty();

    if sound {
        pwm.set_duty(Channel::C4, max / 2);
    } else {
        pwm.set_duty(Channel::C4, 0);
    }
}
