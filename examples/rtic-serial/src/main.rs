// src/main.rs

// This example show how to use the Real-Time Interrupt-driven Concurrency (RTIC) framework to handle UART2 interruptions and control the board LED

// std and main are not available for bare metal software
#![no_std]
#![no_main]

use embedded_hal::digital::v2::OutputPin;
use hal::{
    afio, flash,
    gpio::{
        gpioa::{self, PA2, PA3, PA5},
        Alternate, Floating, Input, Output, PushPull,
    },
    pac::USART2,
    prelude::*,
    rcc::Rcc,
    serial::{Config, Event, Serial, StopBits},
};
use panic_halt as _;
use rtic::app;
use stm32f1xx_hal as hal;

#[app(device = stm32f1xx_hal::pac, peripherals = true)]
const APP: () = {
    // Late Resources initialized at runtime after the init function
    struct Resources {
        SERIAL: Serial<USART2, (PA2<Alternate<PushPull>>, PA3<Input<Floating>>)>,
        LED: PA5<Output<PushPull>>,
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        // Set up peripherals
        let mut rcc: Rcc = cx.device.RCC.constrain();
        let mut flash: flash::Parts = cx.device.FLASH.constrain();
        let mut afio: afio::Parts = cx.device.AFIO.constrain(&mut rcc.apb2);
        let mut gpioa: gpioa::Parts = cx.device.GPIOA.split(&mut rcc.apb2);

        // Freeze clocks
        let clocks = rcc.cfgr.freeze(&mut flash.acr);

        // Set up LED and USART2 pins
        let led = gpioa.pa5.into_push_pull_output(&mut gpioa.crl);
        let tx = gpioa.pa2.into_alternate_push_pull(&mut gpioa.crl);
        let rx = gpioa.pa3;

        // Get USART2 instance
        let mut serial = Serial::usart2(
            cx.device.USART2,
            (tx, rx),
            &mut afio.mapr,
            Config::default()
                .baudrate(9600.bps())
                .stopbits(StopBits::STOP1)
                .parity_none(),
            clocks,
            &mut rcc.apb1,
        );

        // Enable RX interruption
        serial.listen(Event::Rxne);

        // Set up late resources
        init::LateResources {
            SERIAL: serial,
            LED: led,
        }
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        // Infinite loop so system doesn't go to sleep
        loop {}
    }

    /// USART2 ISR
    #[task(binds = USART2, resources = [SERIAL, LED])]
    fn usart2_isr(c: usart2_isr::Context) {
        let serial: &mut Serial<USART2, (PA2<Alternate<PushPull>>, PA3<Input<Floating>>)> =
            c.resources.SERIAL;
        let led: &mut PA5<Output<PushPull>> = c.resources.LED;

        // Read serial data register, automatic clearing RX interruption flag
        let received = serial.read().unwrap();

        // Control the board LED
        if received == b's' {
            led.set_high().ok();
        } else if received == b'u' {
            led.set_low().ok();
        }
    }
};
