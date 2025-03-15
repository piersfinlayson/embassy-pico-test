// Copyright (c) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT licensed - see https://opensource.org/license/MIT

// memory.x handling drived from embassy-rs examples.

#![allow(dead_code)]
#![allow(unused_imports)]
#![no_std]
#![no_main]

use {defmt_rtt as _, panic_probe as _};

use core::arch::asm;
use defmt::{error, info, warn};
use embassy_executor::Spawner;
use embassy_futures::yield_now;
use embassy_rp::gpio::{self, Drive, Input, Level, Output, Pin, Pull};
use embassy_rp::peripherals;
use embassy_time::{Delay, Duration, Instant, Timer};
use embedded_hal::delay::DelayNs;

// RP2040 SIO base address
const SIO_BASE: u32 = 0xd0000000;
// GPIO output set register (writing 1 sets the pin)
const GPIO_OUT: u32 = SIO_BASE + 0x010;

#[cfg(feature = "pico")]
const BOARD: &str = "Pico";
#[cfg(feature = "pico")]
const IS_PICO2: bool = false;
#[cfg(feature = "pico2")]
const BOARD: &str = "Pico 2";
#[cfg(feature = "pico2")]
const IS_PICO2: bool = true;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Get test type and number
    let test_num = TestNum::get();
    let test_type = TestType::get();

    info!("embassy-pico-test");

    match test_type {
        TestType::SingleGpio => Test::single_gpio(test_num).await,
    }
}

macro_rules! single_gpio {
    ($desc:expr, $pause:block, $pin:expr) => {
        {
            info!(": {}", $desc);
            info!(": Starting");
            loop {
                $pin.set_high();
                $pause
                $pin.set_low();
                $pause
            }
        }
    };
}

struct Test {}

impl Test {
    async fn single_gpio(test_num: TestNum) -> ! {
        let p = embassy_rp::init(Default::default());

        let speed = embassy_rp::clocks::clk_sys_freq();
        info!("{} clock speed: {} Hz", BOARD, speed);
        info!("Single GPIO Timing test #{}", test_num as i32);
        info!(": Using GPIO 2");

        let mut output = Output::new(p.PIN_2, Level::Low);

        match test_num {
            TestNum::T1 => single_gpio!(
                "~200us period using yielding Timer::after_micros",
                { Timer::after_micros(100).await },
                output
            ),
            TestNum::T2 => single_gpio!(
                "~20us period using yielding Timer::after_micros",
                { Timer::after_micros(10).await },
                output
            ),
            TestNum::T3 => single_gpio!(
                "~2us period using yielding Timer::after_micros",
                { Timer::after_micros(1).await },
                output
            ),
            TestNum::T4 => single_gpio!(
                "200us period using blocking Delay.delay_us",
                { Delay.delay_us(100) },
                output
            ),
            TestNum::T5 => single_gpio!(
                "20us period using blocking Delay.delay_us",
                { Delay.delay_us(10) },
                output
            ),
            TestNum::T6 => single_gpio!(
                "4us period using blocking Delay.delay_us",
                { Delay.delay_us(2) },
                output
            ),
            TestNum::T7 => single_gpio!(
                "2us period using blocking Delay.delay_us",
                { Delay.delay_us(1) },
                output
            ),
            TestNum::T8 => single_gpio!(
                "not near 200ns period using blocking Delay.delay_ns",
                { Delay.delay_ns(100) },
                output
            ),
            TestNum::T9 => single_gpio!(
                "~200us period using blocking Delay.delay_us then yield_now()",
                {
                    Delay.delay_us(100);
                    yield_now().await
                },
                output
            ),
            TestNum::T10 => single_gpio!(
                "~20us period using blocking Delay.delay_us then yield_now()",
                {
                    Delay.delay_us(10);
                    yield_now().await
                },
                output
            ),
            TestNum::T11 => single_gpio!(
                "~2us period using blocking Delay.delay_us then yield_now()",
                {
                    Delay.delay_us(1);
                    yield_now().await
                },
                output
            ),
            TestNum::T12 => single_gpio!(
                "\"2 cycle\" delay using blocking cortex_m::asm::delay()",
                { cortex_m::asm::delay(2) },
                output
            ),
            TestNum::T13 => {
                single_gpio!(
                    "As fast as possible with no delay and embassy GPIO functions",
                    {},
                    output
                );
            }
            TestNum::T14 => {
                info!(": Using same assembly for both Pico and Pico 2");
                if !IS_PICO2 {
                    info!(": 200ns period using asm (Pico)    <== selected");
                    info!(": 100ns period using asm (Pico 2)");
                } else {
                    info!(": 200ns period using asm (Pico)");
                    info!(": 100ns period using asm (Pico 2)  <== selected");
                }
                info!(": Starting");
                Self::asm_toggle_gpio2_period_200ns_pico();
            }
            TestNum::T15 => {
                info!(": Using Pico and Pico 2 specific assembly");
                info!(": 200ns period using asm on both Pico and Pico 2");
                info!(": Starting");
                Self::asm_toggle_gpio2_period_200ns();
            }
            TestNum::T16 => {
                info!(": Using Pico and Pico 2 specific assembly");
                info!(": 80ns period using asm on both Pico and Pico 2");
                info!(": Low drive strength (2mA)");
                info!(": Starting");
                output.set_drive_strength(Drive::_2mA);
                Self::asm_toggle_gpio2_period_80ns();
            }
            TestNum::T17 => {
                info!(": Using same assembly for both Pico and Pico 2");
                if !IS_PICO2 {
                    info!(": 48ns period using asm (Pico)    <== selected");
                    info!(": 34ns period using asm (Pico 2)");
                } else {
                    info!(": 48ns period using asm (Pico)");
                    info!(": 34ns period using asm (Pico 2)  <== selected");
                }
                info!(": Low drive strength (2mA)");
                info!(": Starting");
                output.set_drive_strength(Drive::_2mA);
                Self::asm_toggle_gpio2_period_min();
            }
            TestNum::T18 => {
                info!(": Using same assembly for both Pico and Pico 2");
                if !IS_PICO2 {
                    info!(": 48ns period using asm (Pico)    <== selected");
                    info!(": 34ns period using asm (Pico 2)");
                } else {
                    info!(": 48ns period using asm (Pico)");
                    info!(": 34ns period using asm (Pico 2)  <== selected");
                }
                info!(": High drive strength (12mA)");
                info!(": Starting");
                output.set_drive_strength(Drive::_12mA);
                Self::asm_toggle_gpio2_period_min();
            }
            TestNum::T19 => {
                info!(": Using Pico and Pico 2 specific assembly");
                info!(": 20us period using asm on both Pico and Pico 2");
                info!(": Uses Timer::at()");
                info!(": Starting");
                let mut expires = Instant::now();
                let _10us = Duration::from_micros(10);
                loop {
                    output.set_high();
                    expires += _10us;
                    Timer::at(expires).await;
                    output.set_low();
                    expires += _10us;
                    Timer::at(expires).await;
                }
            }
            _ => unimplemented!("Test {} not implemented", test_num as i32),
        }
    }

    // This function takes exactly 200ns to toggle GPIO 2 twice.  It does 8
    // cycles of work, and 17 cycles of no-ops, for a total of 25 clock
    // cycles.  The Pico does 125MHz, so 25 cycles is 200ns.
    //
    // However, this takes 100ns on the Pico 2, as it used only 15 cycles, at
    // 150Mhz.  This is because the branch takes 1 fewer cycle, and some of
    // the other instructions (likely nops) are being executed in parallel.
    fn asm_toggle_gpio2_period_200ns_pico() -> ! {
        // Load register r0 with the GPIO_OUT register address
        Self::asm_load_gpio_out_addr();

        // Loop around, setting GPIO 2 high, pausing 10 clock cycles, then
        // setting GPIO 2 low, pausing 9 clock cycles.
        loop {
            Self::set_gpio2_high();
            Self::asm_10_cycles_nop();
            Self::set_gpio2_low();
            Self::asm_9_cycles_nop();
        }
    }

    // This function takes exactly 200ns to toggle GPIO 2 twice on both the
    // Pico and Pico 2.  It does this by:
    // - using different no-op instructions - insteads adds to a register
    // - adding the additional, required, number of cycles in the Pico 2 case,
    //   to account for 1 fewer cycle in the branch instruction, and the 5
    //   extra required due to its faster speed.
    fn asm_toggle_gpio2_period_200ns() -> ! {
        // Load register r0 with the GPIO_OUT register address
        Self::asm_load_gpio_out_addr();

        // Loop around, setting GPIO 2 high, pausing 10 clock cycles, then
        // setting GPIO 2 low, pausing 9 clock cycles.
        loop {
            Self::set_gpio2_high();
            Self::asm_10_cycles_add_r2();
            #[cfg(feature = "pico2")]
            Self::asm_3_cycles_add_r2();
            Self::set_gpio2_low();
            Self::asm_9_cycles_add_r2();
            #[cfg(feature = "pico2")]
            Self::asm_3_cycles_add_r2();
        }
    }

    // Toggles GPIO 2 using a period of 80ns across both Pico and Pico 2.
    // 80ns is 10 cycles on the Pico and 12 on the Pico 2.
    fn asm_toggle_gpio2_period_80ns() -> ! {
        // Load register r0 with the GPIO_OUT register address
        Self::asm_load_gpio_out_addr();

        loop {
            Self::set_gpio2_high(); // 2 cycles
            #[cfg(feature = "pico")]
            Self::asm_2_cycles_add_r2();
            #[cfg(feature = "pico2")]
            Self::asm_3_cycles_add_r2();
            Self::set_gpio2_low(); // 2 cycles
            Self::asm_2_cycles_add_r2();
            #[cfg(feature = "pico2")]
            Self::asm_2_cycles_add_r2();
        }
    }

    // Toggles GPIO 2 using minimum period possible.
    fn asm_toggle_gpio2_period_min() -> ! {
        // Load register r0 with the GPIO_OUT register address
        Self::asm_load_gpio_out_addr();

        loop {
            Self::set_gpio2_high(); // 2 cycles
            Self::set_gpio2_low(); // 2 cycles
        }
    }

    // Loads the GPIO_OUT register address into register r0, and returns it.
    #[inline(always)]
    fn asm_load_gpio_out_addr() {
        // SIO base is 0xd0000000
        // GPIO_OUT register is SIO_base + 0x10
        unsafe {
            asm!(
                "movs r1, #0xd0",
                "lsls r1, r1, #24", // Shift left 3 bytes, 24 bits
                "movs r2, #0x10",
                "adds r0, r1, r2",  // Add SIO based and GPIO_OUT offset
                out("r0") _,  // Tell compiler what registers we used
                out("r1") _,
                out("r2") _,
            );
        }
    }

    // Assumes r0 is loaded with GPIO_OUT, and sets (only) GPIO 2 high.
    #[inline(always)]
    fn set_gpio2_high() {
        unsafe {
            asm!(
                "movs r1, #4",    // Set r1 to 4 (bit 2 for GPIO2)
                "str r1, [r0]",   // Store r1 to the address in r0 (sets GPIO2 high)
                out("r1") _,
            );
        }
    }

    // Assumes r0 is loaded with GPIO_OUT, and sets GPIO 2 low (plus all
    // other GPIOs).
    #[inline(always)]
    fn set_gpio2_low() {
        unsafe {
            asm!(
                "movs r1, #0",    // Set r1 to 0
                "str r1, [r0]",   // Store r1 to the address in r0 (sets GPIO2 low)
                out("r1") _,
            );
        }
    }

    // 1 cycle nop
    #[inline(always)]
    fn asm_1_cycle_r2() {
        unsafe {
            asm!("movs r2, #1");
        }
    }

    // 2 cycles
    #[inline(always)]
    fn asm_2_cycles_add_r2() {
        unsafe {
            asm!("movs r2, #1");
            asm!("adds r2, r2, #1");
        }
    }

    // 3 cycles
    #[inline(always)]
    fn asm_3_cycles_add_r2() {
        unsafe {
            asm!("movs r2, #1");
            asm!("adds r2, r2, #1");
            asm!("adds r2, r2, #1");
        }
    }

    // 5 cycles
    #[inline(always)]
    fn asm_5_cycles_r2() {
        unsafe {
            asm!("movs r2, #1");
            asm!("adds r2, r2, #1");
            asm!("adds r2, r2, #1");
            asm!("adds r2, r2, #1");
            asm!("adds r2, r2, #1");
        }
    }

    // 9 cycles = 72ms on the Pico
    #[inline(always)]
    fn asm_9_cycles_add_r2() {
        unsafe {
            asm!("movs r2, #1");
            asm!("adds r2, r2, #1");
            asm!("adds r2, r2, #1");
            asm!("adds r2, r2, #1");
            asm!("adds r2, r2, #1");
            asm!("adds r2, r2, #1");
            asm!("adds r2, r2, #1");
            asm!("adds r2, r2, #1");
            asm!("adds r2, r2, #1");
        }
    }

    // 10 cycles - 80ns on the Pico
    #[inline(always)]
    fn asm_10_cycles_add_r2() {
        unsafe {
            asm!("movs r2, #1");
            asm!("adds r2, r2, #1");
            asm!("adds r2, r2, #1");
            asm!("adds r2, r2, #1");
            asm!("adds r2, r2, #1");
            asm!("adds r2, r2, #1");
            asm!("adds r2, r2, #1");
            asm!("adds r2, r2, #1");
            asm!("adds r2, r2, #1");
            asm!("adds r2, r2, #1");
        }
    }

    // 9 nops = 72ms on the Pico
    #[inline(always)]
    fn asm_9_cycles_nop() {
        unsafe {
            asm!("nop");
            asm!("nop");
            asm!("nop");
            asm!("nop");
            asm!("nop");
            asm!("nop");
            asm!("nop");
            asm!("nop");
            asm!("nop");
        }
    }

    // 10 nops - 80ns on the Pico
    #[inline(always)]
    fn asm_10_cycles_nop() {
        unsafe {
            asm!("nop");
            asm!("nop");
            asm!("nop");
            asm!("nop");
            asm!("nop");
            asm!("nop");
            asm!("nop");
            asm!("nop");
            asm!("nop");
            asm!("nop");
        }
    }
}

// Helper routines to get test type and number
enum TestType {
    SingleGpio,
}

impl TestType {
    fn get() -> Self {
        #[cfg(feature = "single-gpio")]
        return TestType::SingleGpio;
    }
}

#[derive(Clone, Copy)]
#[repr(i32)]
enum TestNum {
    T1 = 1,
    T2,
    T3,
    T4,
    T5,
    T6,
    T7,
    T8,
    T9,
    T10,
    T11,
    T12,
    T13,
    T14,
    T15,
    T16,
    T17,
    T18,
    T19,
    T20,
    T21,
    T22,
    T23,
    T24,
    T25,
}

impl TestNum {
    fn get() -> Self {
        #[cfg(feature = "1")]
        return TestNum::T1;
        #[cfg(feature = "2")]
        return TestNum::T2;
        #[cfg(feature = "3")]
        return TestNum::T3;
        #[cfg(feature = "4")]
        return TestNum::T4;
        #[cfg(feature = "5")]
        return TestNum::T5;
        #[cfg(feature = "6")]
        return TestNum::T6;
        #[cfg(feature = "7")]
        return TestNum::T7;
        #[cfg(feature = "8")]
        return TestNum::T8;
        #[cfg(feature = "9")]
        return TestNum::T9;
        #[cfg(feature = "10")]
        return TestNum::T10;
        #[cfg(feature = "11")]
        return TestNum::T11;
        #[cfg(feature = "12")]
        return TestNum::T12;
        #[cfg(feature = "13")]
        return TestNum::T13;
        #[cfg(feature = "14")]
        return TestNum::T14;
        #[cfg(feature = "15")]
        return TestNum::T15;
        #[cfg(feature = "16")]
        return TestNum::T16;
        #[cfg(feature = "17")]
        return TestNum::T17;
        #[cfg(feature = "18")]
        return TestNum::T18;
        #[cfg(feature = "19")]
        return TestNum::T19;
        #[cfg(feature = "20")]
        return TestNum::T20;
        #[cfg(feature = "21")]
        return TestNum::T21;
        #[cfg(feature = "22")]
        return TestNum::T22;
        #[cfg(feature = "23")]
        return TestNum::T23;
        #[cfg(feature = "24")]
        return TestNum::T24;
        #[cfg(feature = "25")]
        return TestNum::T25;
    }
}
