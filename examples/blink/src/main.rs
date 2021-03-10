// src/main.rs

// Blinks the NUCLEO-F103RB board LED connected to pin 5 on Port A.

// std and main are not available for bare metal software
#![no_std]
#![no_main]

use panic_halt as _;

use cortex_m_rt::entry;
use embedded_hal::digital::v2::OutputPin;
use hal::{delay::Delay, pac, prelude::*};
use stm32f1xx_hal as hal;

#[entry]
fn main() -> ! {
    // Get access to device and core peripherals
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    // Get access to RCC, AFIO and GPIOA
    let mut rcc = dp.RCC.constrain();
    let mut flash = dp.FLASH.constrain();
    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);

    // Set up LED pin
    let mut led = gpioa.pa5.into_push_pull_output(&mut gpioa.crl);

    // Freeze clocks
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    /* Set up systick delay */
    let mut delay = Delay::new(cp.SYST, clocks);

    loop {
        led.set_high().ok();
        delay.delay_ms(1_000_u16);
        led.set_low().ok();
        delay.delay_ms(1_000_u16);
    }
}
