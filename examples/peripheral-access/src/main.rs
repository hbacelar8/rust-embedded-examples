// src/main.rs

// This example shows how to access a register mapped peripheral using the crate `volatile-register`

// std and main are not available for bare metal software
#![no_std]
#![no_main]

use panic_halt as _;

use cortex_m_rt::entry;
use hal::{delay::Delay, pac, prelude::*};
use stm32f1xx_hal as hal;
use volatile_register::RW;

#[repr(C)]
struct GPIOBlock {
    pub crl: RW<u32>,  // Port configuration register low
    pub crh: RW<u32>,  // Port configuration register high
    pub idr: RW<u32>,  // Port input data register
    pub odr: RW<u32>,  // Port output data register
    pub bsrr: RW<u32>, // Port bit set/reset register
    pub brr: RW<u16>,  // Port bit reset register
    pub lckr: RW<u32>, // Port configuration lock register
}

/// GPIOA Struct
pub struct GPIOA {
    p: &'static mut GPIOBlock,
}

/// GPIOA Implementation
impl GPIOA {
    pub fn new() -> GPIOA {
        GPIOA {
            p: unsafe { &mut *(0x4001_0800 as *mut GPIOBlock) },
        }
    }

    /// Configures GPIOA as output push-pull
    pub fn into_push_pull_output(&mut self) {
        unsafe { self.p.crl.modify(|r| (r & 0x1101_1111) | 0x0010_0000) }
    }

    /// Sets pin
    pub fn set_pin(&mut self, pin: u8) {
        unsafe { self.p.bsrr.write(0x0000_0001 << (pin & 0xFF)) }
    }

    /// Clears pin
    pub fn clear_pin(&mut self, pin: u8) {
        unsafe { self.p.brr.write(0x0000_0001 << (pin & 0xFF)) }
    }
}

const LED_PIN: u8 = 5;

#[entry]
fn main() -> ! {
    // Get access to device and core peripherals
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    // Get access to RCC, AFIO and GPIOA
    let rcc = dp.RCC;
    let mut flash = dp.FLASH.constrain();
    let mut gpioa = GPIOA::new();

    // Enable PORTA clock
    rcc.apb2enr.write(|w| w.iopaen().enabled());

    // Set up LED pin
    gpioa.into_push_pull_output();

    // Freeze clocks
    let clocks = rcc.constrain().cfgr.freeze(&mut flash.acr);

    // Set up systick delay
    let mut delay = Delay::new(cp.SYST, clocks);

    loop {
        gpioa.set_pin(LED_PIN);
        delay.delay_ms(500_u16);
        gpioa.clear_pin(LED_PIN);
        delay.delay_ms(500_u16);
    }
}
