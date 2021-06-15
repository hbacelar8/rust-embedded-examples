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

use cortex_m_rt::entry;
use heapless::{consts, Vec};
use panic_halt as _;
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

struct SerialStruct {
    counter: u8,
    app: u8,
    cmd: u8,
    len: u8,
    data: Vec<u8, consts::U8>,
    buffer_free: bool,
}

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
static mut SERIAL_STRUCT: Option<SerialStruct> = None;

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

    // Initialize serial struct
    let serial_struct = SerialStruct {
        counter: 0,
        app: 0,
        cmd: 0,
        len: 0,
        data: Vec::new(),
        buffer_free: true,
    };

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
        SERIAL_STRUCT = Some(serial_struct);
    }

    // Do nothing; wait for interruptions
    loop {}
}

/// USART2 ISR
#[interrupt]
fn USART2() {
    // Get local access to static global variables
    let serial = unsafe { SERIAL.as_mut().unwrap() };
    let serial_struct = unsafe { SERIAL_STRUCT.as_mut().unwrap() };

    // Read received byte, cleaning RX flag
    let byte_received = serial.read().unwrap();

    if serial_struct.buffer_free {
        match serial_struct.counter {
            0 => {
                if byte_received == 0xA0 || byte_received == 0xB0 {
                    serial_struct.app = byte_received;
                    serial_struct.counter += 1;
                }
            }
            1 => {
                serial_struct.cmd = byte_received;
                serial_struct.counter += 1;
            }
            2 => {
                serial_struct.len = byte_received;
                serial_struct.counter += 1;

                if serial_struct.len == 0 {
                    serial_struct.counter = 0;
                    msg_handler();
                }
            }
            _ => {
                serial_struct.data.push(byte_received).ok();
                serial_struct.counter += 1;

                if serial_struct.counter == serial_struct.len + 3 {
                    serial_struct.counter = 0;
                    serial_struct.buffer_free = false;
                    msg_handler();
                }
            }
        }
    }
}

/// USART2 callback function
fn msg_handler() {
    // Get local access to static global variables
    let pwm = unsafe { PWM.as_mut().unwrap() };
    let serial_struct = unsafe { SERIAL_STRUCT.as_mut().unwrap() };

    match serial_struct.app {
        0xA0 => {
            // Get max duty cycle and divide it by steps of 255 for the color range
            let step = pwm.get_max_duty() / 255;

            match serial_struct.cmd {
                0x00 => {
                    let red = serial_struct.data[0];
                    let green = serial_struct.data[1];
                    let blue = serial_struct.data[2];

                    pwm.set_duty(Channel::C1, step * red as u16);
                    pwm.set_duty(Channel::C2, step * green as u16);
                    pwm.set_duty(Channel::C3, step * blue as u16);
                }
                0x01 => {
                    let red = serial_struct.data[0];

                    pwm.set_duty(Channel::C1, step * red as u16);
                }
                0x02 => {
                    let green = serial_struct.data[0];

                    pwm.set_duty(Channel::C2, step * green as u16);
                }
                0x03 => {
                    let blue = serial_struct.data[0];

                    pwm.set_duty(Channel::C3, step * blue as u16);
                }
                _ => {}
            }
        }
        0xB0 => match serial_struct.cmd {
            0x01 => {
                let max = pwm.get_max_duty();

                pwm.set_duty(Channel::C4, max / 2);
            }
            0x02 => pwm.set_duty(Channel::C4, 0),
            _ => {}
        },
        _ => {}
    }

    serial_struct.data.clear();
    serial_struct.buffer_free = true;
}
