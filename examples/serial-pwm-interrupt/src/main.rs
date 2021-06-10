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

#![no_std]
#![no_main]

use panic_halt as _;
use cortex_m_rt::entry;
use heapless::{consts, Vec};
use stm32f1xx_hal::{
    gpio::{
        gpioa::{PA2, PA3},
        gpiob::{PB6, PB7, PB8, PB9},
        Alternate, Floating, Input, PushPull,
    },
    pac::{self, interrupt, NVIC, TIM4, USART2},
    prelude::*,
    pwm::{Channel, Pwm, C1, C2, C3, C4},
    serial::{Config, Event, Serial, StopBits},
    time::U32Ext,
    timer::{Tim4NoRemap, Timer},
};

// Global static variables
static mut SERIAL: Option<Serial<USART2, (PA2<Alternate<PushPull>>, PA3<Input<Floating>>)>> = None;
static mut PWM: Option<
    Pwm<
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
> = None;
static mut BUFFER: Option<Vec<u8, consts::U8>> = None;
static mut COUNTER: u8 = 0;
static mut APP: u8 = 0;
static mut CMD: u8 = 0;
static mut LEN: u8 = 0;

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
    let buffer: Vec<u8, heapless::consts::U8> = Vec::new();

    // Enable USART2 interruptions
    unsafe {
        NVIC::unmask(interrupt::USART2);
    }

    // Listen to RX interruptions
    serial.listen(Event::Rxne);

    // Assign static global variables
    unsafe {
        SERIAL = Some(serial);
        PWM = Some(pwm);
        BUFFER = Some(buffer);
    }

    // Do nothing; wait for interruptions
    loop {}
}

/// USART2 ISR
#[interrupt]
fn USART2() {
    // Get local access to static global variables
    let serial = unsafe { SERIAL.as_mut().unwrap() };
    let buffer = unsafe { BUFFER.as_mut().unwrap() };
    let counter = unsafe { &mut COUNTER };
    let app = unsafe { &mut APP };
    let cmd = unsafe { &mut CMD };
    let len = unsafe { &mut LEN };

    // Read received byte, cleaning RX flag
    let byte_received = serial.read().unwrap();

    match *counter {
        0 => {
            if byte_received == 0xA0 || byte_received == 0xB0 {
                *app = byte_received;
                *counter += 1;
            }
        }
        1 => {
            *cmd = byte_received;
            *counter += 1;
        }
        2 => {
            *len = byte_received;
            *counter += 1;

            if *len == 0 {
                *counter = 0;
                msg_handler();
            }
        }
        _ => {
            buffer.push(byte_received).ok();
            *counter += 1;

            if *counter == *len + 3 {
                *counter = 0;
                msg_handler();
            }
        }
    }
}

/// USART2 callback function
fn msg_handler() {
    // Get local access to static global variables
    let pwm = unsafe { PWM.as_mut().unwrap() };
    let buffer = unsafe { BUFFER.as_mut().unwrap() };
    let app = unsafe { APP };
    let cmd = unsafe { CMD };

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
