// src/main.rs

// std and main are not available for bare metal software
#![no_std]
#![no_main]

mod lcd;

use core::mem::MaybeUninit;
use embedded_hal::digital::v2::OutputPin;
use heapless::{self, Vec};
use lcd::LCD;
use panic_halt as _;

use hal::{
    delay::Delay,
    gpio::{
        gpioa::{self, PA2, PA3, PA5},
        gpiob::{self, PB6, PB7, PB8},
        gpioc::{self},
        Alternate, Floating, Input, Output, PushPull, State,
    },
    pac::{self, TIM1, TIM4},
    prelude::*,
    pwm::{Channel, Pwm, C1, C2, C3},
    serial::{Config, Event, Serial, StopBits},
    timer::{CountDownTimer, Tim4NoRemap, Timer},
};
use rtic::app;
use stm32f1xx_hal as hal;

#[app(device = stm32f1xx_hal::pac, peripherals = true)]
const APP: () = {
    // Late Resources initialized at runtime after the init function
    struct Resources {
        serial_handler:
            Serial<pac::USART2, (gpioa::PA2<Alternate<PushPull>>, gpioa::PA3<Input<Floating>>)>,
        timer_handler: CountDownTimer<TIM1>,
        pwm_handler: Pwm<
            TIM4,
            Tim4NoRemap,
            (C1, C2, C3),
            (
                PB6<Alternate<PushPull>>,
                PB7<Alternate<PushPull>>,
                PB8<Alternate<PushPull>>,
            ),
        >,

        lcd: &'static mut LCD,
        led: gpioa::PA5<Output<PushPull>>,

        rx_buffer: Vec<u8, heapless::consts::U32>,

        #[init(0)]
        byte_counter: u8,

        #[init(false)]
        led_state: bool,

        #[init(1)]
        led_freq: u8,
    }

    /// Initialization task
    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        // Set up peripherals
        let mut rcc = cx.device.RCC.constrain();
        let mut flash = cx.device.FLASH.constrain();
        let mut afio = cx.device.AFIO.constrain(&mut rcc.apb2);
        let mut gpioa: gpioa::Parts = cx.device.GPIOA.split(&mut rcc.apb2);
        let mut gpiob: gpiob::Parts = cx.device.GPIOB.split(&mut rcc.apb2);
        let mut gpioc: gpioc::Parts = cx.device.GPIOC.split(&mut rcc.apb2);

        // Freeze clocks
        let clocks = rcc.cfgr.freeze(&mut flash.acr);

        // Get delay instance
        let delay = Delay::new(cx.core.SYST, clocks);

        // Configure board LED
        let led: PA5<Output<PushPull>> = gpioa
            .pa5
            .into_push_pull_output_with_state(&mut gpioa.crl, State::High);

        // Configure timer TIM1
        let mut timer =
            Timer::tim1(cx.device.TIM1, &clocks, &mut rcc.apb2).start_count_down(1.hz());

        // Configure PWM
        let c1: PB6<Alternate<PushPull>> = gpiob.pb6.into_alternate_push_pull(&mut gpiob.crl);
        let c2: PB7<Alternate<PushPull>> = gpiob.pb7.into_alternate_push_pull(&mut gpiob.crl);
        let c3: PB8<Alternate<PushPull>> = gpiob.pb8.into_alternate_push_pull(&mut gpiob.crh);
        let pwm_pins = (c1, c2, c3);

        let mut pwm = Timer::tim4(cx.device.TIM4, &clocks, &mut rcc.apb1)
            .pwm::<Tim4NoRemap, _, _, _>(pwm_pins, &mut afio.mapr, 1.khz());

        pwm.enable(Channel::C1);
        pwm.enable(Channel::C2);
        pwm.enable(Channel::C3);

        // Configure LCD
        let rs = gpioa.pa0.into_push_pull_output(&mut gpioa.crl);
        let en = gpioa.pa1.into_push_pull_output(&mut gpioa.crl);
        let d4 = gpioc.pc0.into_push_pull_output(&mut gpioc.crl);
        let d5 = gpioc.pc1.into_push_pull_output(&mut gpioc.crl);
        let d6 = gpioc.pc2.into_push_pull_output(&mut gpioc.crl);
        let d7 = gpioc.pc3.into_push_pull_output(&mut gpioc.crl);

        let lcd = unsafe {
            static mut LCD: MaybeUninit<LCD> = MaybeUninit::uninit();

            LCD.as_mut_ptr()
                .write(LCD::new(rs, en, d4, d5, d6, d7, delay));
            &mut *LCD.as_mut_ptr()
        };

        lcd.init();

        // Configure UART2
        let tx: PA2<Alternate<PushPull>> = gpioa.pa2.into_alternate_push_pull(&mut gpioa.crl);
        let rx: PA3<Input<Floating>> = gpioa.pa3;

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

        // Create RX buffer
        let rx_buffer: Vec<u8, heapless::consts::U32> = Vec::new();

        // Enable RX interruption
        serial.listen(Event::Rxne);

        // Enable timer coutdown interruption
        timer.listen(hal::timer::Event::Update);

        // Set late resources
        init::LateResources {
            serial_handler: serial,
            timer_handler: timer,
            pwm_handler: pwm,
            lcd,
            led,
            rx_buffer,
        }
    }

    /// Idle task
    #[idle]
    fn idle(_: idle::Context) -> ! {
        // Infinite loop so system doesn't go to sleep
        loop {}
    }

    /// TIM1 ISR
    #[task(binds = TIM1_UP, priority = 1, resources = [timer_handler, led, led_state])]
    fn TIM1_UP(cx: TIM1_UP::Context) {
        if *cx.resources.led_state {
            cx.resources.led.set_high().ok();
            *cx.resources.led_state = false;
        } else {
            cx.resources.led.set_low().ok();
            *cx.resources.led_state = true;
        }

        cx.resources.timer_handler.clear_update_interrupt_flag();
        // cx.spawn.uart_handler2().unwrap();
    }

    /// UART2 ISR
    #[task(binds = USART2, spawn = [uart_handler], resources = [serial_handler, byte_counter, rx_buffer])]
    fn USART2(c: USART2::Context) {
        // Read serial data register, automatic clearing RX interruption flag
        let received = c.resources.serial_handler.read().unwrap();
        let rx_buffer: &mut Vec<u8, heapless::consts::U32> = c.resources.rx_buffer;

        if *c.resources.byte_counter < 3 {
            rx_buffer.push(received).unwrap();
            *c.resources.byte_counter += 1;
        } else {
            if *c.resources.byte_counter == rx_buffer[2] + 2 {
                *c.resources.byte_counter = 0;
                rx_buffer.push(received).unwrap();
                c.spawn.uart_handler().unwrap();
            } else {
                rx_buffer.push(received).unwrap();
                *c.resources.byte_counter += 1;
            }
        }
    }

    /// Handle commands received through UART
    #[task(resources = [lcd, led, timer_handler, pwm_handler, rx_buffer, led_freq])]
    fn uart_handler(c: uart_handler::Context) {
        let rx_buffer: &mut Vec<u8, heapless::consts::U32> = c.resources.rx_buffer;
        let timer_handler: &mut CountDownTimer<TIM1> = c.resources.timer_handler;
        let pwm_handler: &mut Pwm<
            TIM4,
            Tim4NoRemap,
            (C1, C2, C3),
            (
                PB6<Alternate<PushPull>>,
                PB7<Alternate<PushPull>>,
                PB8<Alternate<PushPull>>,
            ),
        > = c.resources.pwm_handler;
        let lcd: &mut LCD = c.resources.lcd;
        let app = rx_buffer[0];
        let cmd = rx_buffer[1];
        let len = rx_buffer[2];

        match app {
            0xC0 => match cmd {
                0x01 => {
                    if rx_buffer[3] == 1 {
                        pwm_handler.set_duty(Channel::C1, 0);
                        pwm_handler.set_duty(Channel::C2, 0);
                        pwm_handler.set_duty(Channel::C3, 0);
                    } else if rx_buffer[3] == 2 {
                        let max_duty_cycle = pwm_handler.get_max_duty();
                        pwm_handler.set_duty(Channel::C1, max_duty_cycle);
                        pwm_handler.set_duty(Channel::C2, max_duty_cycle);
                        pwm_handler.set_duty(Channel::C3, max_duty_cycle);
                    }
                }
                0x02 => {
                    let step = pwm_handler.get_max_duty() / 255;
                    let red_val = rx_buffer[3];
                    pwm_handler.set_duty(Channel::C1, step * (red_val as u16));
                }
                0x03 => {
                    let step = pwm_handler.get_max_duty() / 255;
                    let green_val = rx_buffer[3];
                    pwm_handler.set_duty(Channel::C2, step * (green_val as u16));
                }
                0x04 => {
                    let step = pwm_handler.get_max_duty() / 255;
                    let blue_val = rx_buffer[3];
                    pwm_handler.set_duty(Channel::C3, step * (blue_val as u16));
                }
                _ => (),
            },
            0xB0 => match cmd {
                0x01 => {
                    let new_freq = rx_buffer[3];
                    timer_handler.start((new_freq as u32).hz());
                    *c.resources.led_freq = new_freq;

                    // Enable timer coutdown interruption
                    timer_handler.listen(hal::timer::Event::Update);
                }
                0x02 => {
                    // Unable timer coutdown interruption
                    timer_handler.unlisten(hal::timer::Event::Update);
                    c.resources.led.set_low().ok();
                }
                0x03 => {
                    // Enable timer coutdown interruption
                    timer_handler.listen(hal::timer::Event::Update);
                }
                _ => (),
            },
            0xA0 => match cmd {
                0x01 => lcd.send_cmd(rx_buffer[3]),
                0x02 => {
                    for n in 3..(3 + len) {
                        lcd.send_data(rx_buffer[n as usize]);
                    }
                }
                _ => (),
            },
            _ => (),
        }

        rx_buffer.clear();
    }

    // /// Handle commands received through UART
    // #[task(priority = 2)]
    // fn uart_handler2(_: uart_handler2::Context) {
    //     let _i= 0;
    // }

    extern "C" {
        fn TAMPER();
    // fn EXTI1();
    }
};
