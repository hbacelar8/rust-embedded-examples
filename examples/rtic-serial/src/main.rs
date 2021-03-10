// src/main.rs

// This example show how to use the Real-Time Interrupt-driven Concurrency (RTIC) framework to handle UART2 interruptions and control the board LED.

// std and main are not available for bare metal software
#![no_std]
#![no_main]

use panic_halt as _;

use embedded_hal::digital::v2::OutputPin;
use hal::{
    gpio::gpioa::Parts,
    serial::{Config, Event, Serial, StopBits},
};
use hal::{gpio::*, pac, prelude::*};
use rtic::app;
use stm32f1xx_hal as hal;

#[app(device = stm32f1xx_hal::pac, peripherals = true)]
const APP: () = {
    // Late Resources initialized at runtime after the init function
    struct Resources {
        serial_handler:
            Serial<pac::USART2, (gpioa::PA2<Alternate<PushPull>>, gpioa::PA3<Input<Floating>>)>,
        led: gpioa::PA5<Output<PushPull>>,
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        // Set up peripherals
        let mut rcc = cx.device.RCC.constrain();
        let mut flash = cx.device.FLASH.constrain();
        let mut afio = cx.device.AFIO.constrain(&mut rcc.apb2);
        let mut gpioa: Parts = cx.device.GPIOA.split(&mut rcc.apb2);

        // Freeze clocks
        let clocks = rcc.cfgr.freeze(&mut flash.acr);

        // Set up LED and UART2 pins
        let led = gpioa.pa5.into_push_pull_output(&mut gpioa.crl);
        let tx = gpioa.pa2.into_alternate_push_pull(&mut gpioa.crl);
        let rx = gpioa.pa3;

        // Get UART2 instance
        let mut serial_handler = Serial::usart2(
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
        serial_handler.listen(Event::Rxne);

        // Set up late resources
        init::LateResources {
            serial_handler,
            led,
        }
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        // Infinite loop so system doesn't go to sleep
        loop {}
    }

    /// USART2 ISR
    #[task(binds = USART2, resources = [serial_handler, led])]
    fn USART2(c: USART2::Context) {
        let serial = c.resources.serial_handler;
        let led = c.resources.led;

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
