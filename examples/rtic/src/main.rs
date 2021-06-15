// src/main.rs

// std and main are not available for bare metal software
#![no_std]
#![no_main]

mod lcd;

use crate::lcd::LCD;
use core::mem::MaybeUninit;
use embedded_hal::digital::v2::OutputPin;
use heapless::{consts, Vec};
use rtic::app;
use stm32f1xx_hal::{
    self, afio,
    delay::Delay,
    flash,
    gpio::{
        gpioa::{self, PA2, PA3, PA5},
        gpiob::{self, PB6, PB7, PB8},
        gpioc, Alternate, Floating, Input, Output, PushPull, State,
    },
    pac::{TIM1, TIM4, USART2},
    prelude::*,
    pwm::{Channel, Pwm, C1, C2, C3},
    rcc::Rcc,
    serial::{self, Config, Serial, StopBits},
    timer::{self, CountDownTimer, Tim4NoRemap, Timer},
};

pub struct SerialStruct {
    counter: u8,
    app: u8,
    cmd: u8,
    len: u8,
    data: Vec<u8, consts::U32>,
}

#[app(device = stm32f1xx_hal::pac, peripherals = true)]
const APP: () = {
    // Late Resources initialized at runtime after the init function
    struct Resources {
        LED: PA5<Output<PushPull>>,
        TIMER: CountDownTimer<TIM1>,
        PWM: Pwm<
            TIM4,
            Tim4NoRemap,
            (C1, C2, C3),
            (
                PB6<Alternate<PushPull>>,
                PB7<Alternate<PushPull>>,
                PB8<Alternate<PushPull>>,
            ),
        >,
        LCD: &'static mut LCD,
        SERIAL: Serial<USART2, (PA2<Alternate<PushPull>>, PA3<Input<Floating>>)>,
        SERIAL_STRUCT: SerialStruct,

        #[init(1)]
        LED_FREQ: u8,
    }

    /// Initialization task
    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        // Set up peripherals
        let mut rcc: Rcc = cx.device.RCC.constrain();
        let mut flash: flash::Parts = cx.device.FLASH.constrain();
        let mut afio: afio::Parts = cx.device.AFIO.constrain(&mut rcc.apb2);
        let mut gpioa: gpioa::Parts = cx.device.GPIOA.split(&mut rcc.apb2);
        let mut gpiob: gpiob::Parts = cx.device.GPIOB.split(&mut rcc.apb2);
        let mut gpioc: gpioc::Parts = cx.device.GPIOC.split(&mut rcc.apb2);

        // Freeze clocks
        let clocks = rcc.cfgr.freeze(&mut flash.acr);

        // Configure board LED
        let led = gpioa
            .pa5
            .into_push_pull_output_with_state(&mut gpioa.crl, State::High);

        // Configure timer
        let mut timer =
            Timer::tim1(cx.device.TIM1, &clocks, &mut rcc.apb2).start_count_down(1.hz());

        // Configure PWM
        let c1 = gpiob.pb6.into_alternate_push_pull(&mut gpiob.crl);
        let c2 = gpiob.pb7.into_alternate_push_pull(&mut gpiob.crl);
        let c3 = gpiob.pb8.into_alternate_push_pull(&mut gpiob.crh);
        let pwm_pins = (c1, c2, c3);

        let mut pwm = Timer::tim4(cx.device.TIM4, &clocks, &mut rcc.apb1)
            .pwm::<Tim4NoRemap, _, _, _>(pwm_pins, &mut afio.mapr, 1.khz());

        pwm.enable(Channel::C1);
        pwm.enable(Channel::C2);
        pwm.enable(Channel::C3);

        // Get delay instance
        let delay = Delay::new(cx.core.SYST, clocks);

        // Configure LCD
        let rs = gpioa.pa0.into_push_pull_output(&mut gpioa.crl);
        let en = gpioa.pa1.into_push_pull_output(&mut gpioa.crl);
        let d4 = gpioc.pc0.into_push_pull_output(&mut gpioc.crl);
        let d5 = gpioc.pc1.into_push_pull_output(&mut gpioc.crl);
        let d6 = gpioc.pc2.into_push_pull_output(&mut gpioc.crl);
        let d7 = gpioc.pc3.into_push_pull_output(&mut gpioc.crl);

        let lcd = unsafe {
            static mut LCD: MaybeUninit<LCD> = MaybeUninit::uninit();

            // Write directly into the static storage
            LCD.as_mut_ptr()
                .write(LCD::new(rs, en, d4, d5, d6, d7, delay));

            &mut *LCD.as_mut_ptr()
        };

        lcd.init();

        // Configure UART2
        let tx = gpioa.pa2.into_alternate_push_pull(&mut gpioa.crl);
        let rx = gpioa.pa3;

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
        serial.listen(serial::Event::Rxne);

        // Enable timer countdown interruption
        timer.listen(timer::Event::Update);

        // Initialize serial struct
        let serial_struct = SerialStruct {
            counter: 0,
            app: 0,
            cmd: 0,
            len: 0,
            data: Vec::new(),
        };

        // Assign late resources
        init::LateResources {
            LED: led,
            TIMER: timer,
            PWM: pwm,
            LCD: lcd,
            SERIAL: serial,
            SERIAL_STRUCT: serial_struct,
        }
    }

    /// Idle task
    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {}
    }

    /// TIM1 ISR
    #[task(binds=TIM1_UP, resources=[TIMER, LED])]
    fn tim1_up_isr(mut cx: tim1_up_isr::Context) {
        // Clear TIM1 update interrupt flag
        cx.resources.TIMER.lock(|timer| {
            timer.clear_update_interrupt_flag();
        });

        // Toggle LED
        cx.resources.LED.lock(|led| {
            led.toggle().ok();
        });
    }

    /// USART2 ISR
    #[task(binds=USART2, priority=3, spawn=[msg_handler], resources=[SERIAL, SERIAL_STRUCT])]
    fn usart2_isr(cx: usart2_isr::Context) {
        // Get local access to shared resources
        let serial: &mut Serial<USART2, (PA2<Alternate<PushPull>>, PA3<Input<Floating>>)> =
            cx.resources.SERIAL;
        let serial_struct: &mut SerialStruct = cx.resources.SERIAL_STRUCT;

        // Read received byte, automatic clearing RX interruption flag
        let byte_received = serial.read().unwrap();

        match serial_struct.counter {
            0 => {
                if byte_received == 0xA0 || byte_received == 0xB0 || byte_received == 0xC0 {
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
                    cx.spawn.msg_handler().unwrap();
                }
            }
            _ => {
                serial_struct.data.push(byte_received).ok();
                serial_struct.counter += 1;

                if serial_struct.counter == serial_struct.len + 3 {
                    serial_struct.counter = 0;
                    cx.spawn.msg_handler().unwrap();
                }
            }
        }
    }

    /// Message handler callback function
    #[task(priority=2, resources=[LED, TIMER, PWM, LCD, SERIAL_STRUCT, LED_FREQ])]
    fn msg_handler(mut cx: msg_handler::Context) {
        // Get local access to shared resources
        let led: &mut PA5<Output<PushPull>> = cx.resources.LED;
        let timer: &mut CountDownTimer<TIM1> = cx.resources.TIMER;
        let pwm: &mut Pwm<
            TIM4,
            Tim4NoRemap,
            (C1, C2, C3),
            (
                PB6<Alternate<PushPull>>,
                PB7<Alternate<PushPull>>,
                PB8<Alternate<PushPull>>,
            ),
        > = cx.resources.PWM;
        let lcd: &mut LCD = cx.resources.LCD;
        let led_freq: &mut u8 = cx.resources.LED_FREQ;

        cx.resources
            .SERIAL_STRUCT
            .lock(|serial_struct: &mut SerialStruct| {
                match serial_struct.app {
                    // RGB LED commands
                    0xA0 => {
                        // Get max duty cycle and divide it by steps of 255 for the color range
                        let step = pwm.get_max_duty() / 255;

                        match serial_struct.cmd {
                            0x00 => {
                                // Set 3 colors intensities

                                let red = serial_struct.data[0];
                                let green = serial_struct.data[1];
                                let blue = serial_struct.data[2];

                                pwm.set_duty(Channel::C1, step * red as u16);
                                pwm.set_duty(Channel::C2, step * green as u16);
                                pwm.set_duty(Channel::C3, step * blue as u16);
                            }
                            0x01 => {
                                // Set red color intensity

                                let red = serial_struct.data[0];

                                pwm.set_duty(Channel::C1, step * red as u16);
                            }
                            0x02 => {
                                // Set green color intensity

                                let green = serial_struct.data[0];

                                pwm.set_duty(Channel::C2, step * green as u16);
                            }
                            0x03 => {
                                // Set blue color intensity

                                let blue = serial_struct.data[0];

                                pwm.set_duty(Channel::C3, step * blue as u16);
                            }
                            _ => {}
                        }
                    }
                    // Board LED commands
                    0xB0 => match serial_struct.cmd {
                        0x01 => {
                            // Set new LED blink frequency

                            let new_led_freq = serial_struct.data[0];

                            timer.start((new_led_freq as u32).hz());
                            *led_freq = new_led_freq;

                            // Enable timer countdown interruption
                            timer.listen(timer::Event::Update);
                        }
                        0x02 => {
                            // Turn LED off

                            // Unable timer countdown interruption
                            timer.unlisten(timer::Event::Update);
                            led.set_low().ok();
                        }
                        0x03 => {
                            // Turn LED on

                            // Enable timer countdown interruption
                            timer.listen(timer::Event::Update);
                        }
                        _ => {}
                    },
                    // LCD commands
                    0xC0 => match serial_struct.cmd {
                        0x01 => {
                            // Send command to LCD

                            let cmd = serial_struct.data[0];

                            lcd.send_cmd(cmd);
                        }
                        0x02 => {
                            // Send data to LCD

                            for n in 0..(serial_struct.len) {
                                let data = serial_struct.data[n as usize];

                                lcd.send_data(data);
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                }

                serial_struct.data.clear();
            });
    }

    extern "C" {
        fn TAMPER();
    }
};
