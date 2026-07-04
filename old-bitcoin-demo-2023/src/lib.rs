#![no_std]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use alloc_cortex_m::CortexMHeap;
use rp2040_hal::pac::Peripherals;

pub mod hash;
pub mod rng;

pub fn peripherals() -> Peripherals {
    Peripherals::take().unwrap()
}

#[macro_export]
macro_rules! pins {
    ($pac:expr) => {{
        // The single-cycle I/O block controls our GPIO pins
        let sio = hal::Sio::new($pac.SIO);

        // Set the pins up according to their function on this particular board
        let pins = rp_pico::Pins::new(
            $pac.IO_BANK0,
            $pac.PADS_BANK0,
            sio.gpio_bank0,
            &mut $pac.RESETS,
        );
        pins
    }};
}

#[macro_export]
macro_rules! timer {
    ($clocks:expr, $pac:expr) => {{
        let timer = rp2040_hal::Timer::new($pac.TIMER, &mut $pac.RESETS, $clocks);
        timer
    }};
}

#[macro_export]
macro_rules! clocks {
    ($pac:expr) => {{
        // Set up the watchdog driver - needed by the clock setup code
        let mut watchdog = hal::Watchdog::new($pac.WATCHDOG);

        // Configure the clocks
        //
        // The default is to generate a 125 MHz system clock
        let clocks = hal::clocks::init_clocks_and_plls(
            rp_pico::XOSC_CRYSTAL_FREQ,
            $pac.XOSC,
            $pac.CLOCKS,
            $pac.PLL_SYS,
            $pac.PLL_USB,
            &mut $pac.RESETS,
            &mut watchdog,
        )
        .ok()
        .unwrap();
        clocks
    }};
}

#[macro_export]
macro_rules! uart {
    ($pins:expr, $pac:expr, $clocks:expr, $baudrate:expr) => {{
        use fugit::RateExtU32;
        use rp2040_hal::clocks::ClocksManager;
        use rp2040_hal::gpio::bank0::{Gpio0, Gpio1};
        use rp2040_hal::gpio::{FunctionNull, FunctionUart, Pin, PullDown, PullNone};
        use rp2040_hal::pac::UART0;
        use rp2040_hal::uart::{DataBits, Enabled, StopBits, UartConfig, UartPeripheral};
        use rp2040_hal::{pac, Clock, Timer};
        use rp_pico::{hal, Pins};

        let uart_pins = (
            // UART TX (characters sent from RP2040) on pin 1 (GPIO0)
            $pins.gpio0.reconfigure::<_, PullNone>(),
            // UART RX (characters received by RP2040) on pin 2 (GPIO1)
            $pins.gpio1.reconfigure::<_, PullNone>(),
        );

        // Make a UART on the given pins
        let uart = hal::uart::UartPeripheral::new($pac.UART0, uart_pins, &mut $pac.RESETS)
            .enable(
                UartConfig::new($baudrate.Hz(), DataBits::Eight, None, StopBits::One),
                $clocks.peripheral_clock.freq(),
            )
            .unwrap();
        uart
    }};
}

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

pub fn set_up_allocator() {
    use core::mem::MaybeUninit;
    const HEAP_SIZE: usize = 1024;
    static mut HEAP: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
    unsafe { ALLOCATOR.init(HEAP.as_ptr() as usize, HEAP_SIZE) }
}
