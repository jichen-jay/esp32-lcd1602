#![no_std]
#![no_main]

use esp_hal::{
    clock::ClockControl,
    delay::Delay,
    dma::{Dma, DmaPriority},
    dma_buffers,
    gpio::{Io, Level, Output},
    i2s::{DataFormat, I2s, I2sWrite, Standard},
    peripherals::Peripherals,
    prelude::*,
    system::SystemControl,
};
use esp_println::println;
use esp_backtrace as _;
use esp_hal::entry;


#[entry]
fn main() -> ! {
}
